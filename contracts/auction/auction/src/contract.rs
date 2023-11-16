use std::collections::VecDeque;

use auction_package::helpers::{verify_admin, AuctionConfig, GetPriceResponse};
use auction_package::states::{ADMIN, TWAP_PRICES};

#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{
    to_binary, Binary, Decimal, Deps, DepsMut, Env, MessageInfo, Response, StdResult, Uint128,
};
use cw2::set_contract_version;

use crate::error::ContractError;
use crate::execute;
use crate::helpers::calc_price;
use crate::msg::{ExecuteMsg, GetFundsAmountResponse, InstantiateMsg, NewAuctionParams, QueryMsg};
use crate::state::{
    ActiveAuction, ActiveAuctionStatus, AuctionIds, ACTIVE_AUCTION, AUCTION_CONFIG, AUCTION_FUNDS,
    AUCTION_FUNDS_SUM, AUCTION_IDS, AUCTION_STRATEGY,
};

const CONTRACT_NAME: &str = "crates.io:auction";
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

pub const TWAP_PRICE_MAX_LEN: u64 = 10;

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: InstantiateMsg,
) -> Result<Response, ContractError> {
    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;

    // Set sender as admin
    ADMIN.save(deps.storage, &info.sender)?;

    // Verify pair
    msg.pair.verify()?;

    // Sort price freshness strategy
    let mut price_freshness_strategy = msg.price_freshness_strategy;
    price_freshness_strategy
        .multipliers
        .sort_by(|p1, p2| p2.0.cmp(&p1.0));

    // Set config
    AUCTION_CONFIG.save(
        deps.storage,
        &AuctionConfig {
            is_paused: false,
            pair: msg.pair,
            chain_halt_config: msg.chain_halt_config,
            price_freshness_strategy,
        },
    )?;

    // Set the strategy for this auction
    AUCTION_STRATEGY.save(deps.storage, &msg.auction_strategy)?;

    // Set auction ids as starter
    AUCTION_IDS.save(deps.storage, &AuctionIds { curr: 0, next: 1 })?;

    // Set first auction sum to none
    AUCTION_FUNDS_SUM.save(deps.storage, 0, &Uint128::zero())?;

    // Set default twap price
    TWAP_PRICES.save(deps.storage, &VecDeque::default())?;

    // Set a default auction
    ACTIVE_AUCTION.save(
        deps.storage,
        &ActiveAuction {
            status: ActiveAuctionStatus::AuctionClosed,
            start_block: 0,
            end_block: 0,
            start_price: Decimal::zero(),
            end_price: Decimal::zero(),
            available_amount: Uint128::zero(),
            resolved_amount: Uint128::zero(),
            total_amount: Uint128::zero(),
            leftovers: [Uint128::zero(), Uint128::zero()],
            last_checked_block: env.block,
        },
    )?;

    Ok(Response::default().add_attribute("method", "instantiate"))
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> Result<Response, ContractError> {
    match msg {
        ExecuteMsg::AuctionFundsManager { sender } => {
            verify_admin(deps.as_ref(), &info)?;
            execute::auction_funds(deps, &info, sender)
        }
        ExecuteMsg::WithdrawFundsManager { sender } => {
            verify_admin(deps.as_ref(), &info)?;
            execute::withdraw_funds(deps, sender)
        }
        ExecuteMsg::AuctionFunds => execute::auction_funds(deps, &info, info.sender.clone()),
        ExecuteMsg::WithdrawFunds => execute::withdraw_funds(deps, info.sender),
        ExecuteMsg::Admin(admin_msg) => admin::handle_msg(deps, env, info, admin_msg),
        ExecuteMsg::Bid => execute::do_bid(deps, &info, &env),
        ExecuteMsg::FinishAuction { limit } => execute::finish_auction(deps, &env, limit),
        ExecuteMsg::CleanAfterAuction => execute::clean_auction(deps),
    }
}

mod admin {
    use auction_package::helpers::GetPriceResponse;

    use crate::msg::AdminMsgs;

    use super::*;

    pub fn handle_msg(
        deps: DepsMut,
        env: Env,
        info: MessageInfo,
        msg: AdminMsgs,
    ) -> Result<Response, ContractError> {
        // Verify that the sender is the admin
        verify_admin(deps.as_ref(), &info)?;

        match msg {
            AdminMsgs::PauseAuction => {
                AUCTION_CONFIG.update(
                    deps.storage,
                    |mut config| -> Result<AuctionConfig, ContractError> {
                        config.is_paused = true;
                        Ok(config)
                    },
                )?;
                Ok(Response::default())
            }
            AdminMsgs::ResumeAuction => {
                AUCTION_CONFIG.update(
                    deps.storage,
                    |mut config| -> Result<AuctionConfig, ContractError> {
                        config.is_paused = false;
                        Ok(config)
                    },
                )?;
                Ok(Response::default())
            }
            AdminMsgs::UpdateStrategy { strategy } => {
                AUCTION_STRATEGY.save(deps.storage, &strategy)?;
                Ok(Response::default())
            }
            AdminMsgs::StartAuction(new_auction) => open_auction(deps, &env, new_auction),
            AdminMsgs::UpdateChainHaltConfig(halt_config) => {
                AUCTION_CONFIG.update(
                    deps.storage,
                    |mut config| -> Result<AuctionConfig, ContractError> {
                        config.chain_halt_config = halt_config;
                        Ok(config)
                    },
                )?;
                Ok(Response::default())
            }
            AdminMsgs::UpdatePriceFreshnessStrategy(strategy) => {
                AUCTION_CONFIG.update(
                    deps.storage,
                    |mut config| -> Result<AuctionConfig, ContractError> {
                        config.price_freshness_strategy = strategy;
                        Ok(config)
                    },
                )?;
                Ok(Response::default())
            }
        }
    }

    fn open_auction(
        deps: DepsMut,
        env: &Env,
        new_auction_params: NewAuctionParams,
    ) -> Result<Response, ContractError> {
        let start_block = new_auction_params.start_block.unwrap_or(env.block.height);
        let end_block = new_auction_params.end_block;

        if end_block <= start_block {
            return Err(ContractError::InvalidAuctionEndBlock);
        }

        let config = AUCTION_CONFIG.load(deps.storage)?;

        if config.is_paused {
            return Err(ContractError::AuctionIsPaused);
        }

        let active_auction = ACTIVE_AUCTION.load(deps.storage)?;

        match active_auction.status {
            ActiveAuctionStatus::AuctionClosed => Ok::<_, ContractError>(()),
            _ => return Err(ContractError::AuctionNotClosed),
        }?;

        let mut auction_ids = AUCTION_IDS.load(deps.storage)?;
        let mut total_funds = AUCTION_FUNDS_SUM
            .load(deps.storage, auction_ids.next)
            .map_err(|_| ContractError::NoFundsForAuction)?;

        if total_funds.is_zero() {
            return Err(ContractError::NoFundsForAuction);
        }

        let (start_price, end_price) = get_strategy_prices(deps.as_ref(), &config, env)?;

        // Add leftovers from previous auction
        total_funds += active_auction.leftovers[0];

        let new_active_auction = ActiveAuction {
            status: ActiveAuctionStatus::Started,
            start_block,
            end_block,
            start_price,
            end_price,
            available_amount: total_funds,
            resolved_amount: active_auction.leftovers[1],
            total_amount: total_funds,
            leftovers: [Uint128::zero(), Uint128::zero()],
            last_checked_block: env.block.clone(),
        };

        // Update auction id
        auction_ids.curr = auction_ids.next;
        auction_ids.next += 1;

        AUCTION_IDS.save(deps.storage, &auction_ids)?;
        ACTIVE_AUCTION.save(deps.storage, &new_active_auction)?;

        Ok(Response::default())
    }

    /// Helper functions to get the starting and ending prices
    /// Factors in freshness of the price from the oracle
    /// as well as the strategy percentage
    fn get_strategy_prices(
        deps: Deps,
        config: &AuctionConfig,
        env: &Env,
    ) -> Result<(Decimal, Decimal), ContractError> {
        let manager_addr = ADMIN.load(deps.storage)?;
        let auction_strategy = AUCTION_STRATEGY.load(deps.storage)?;
        let price: GetPriceResponse = deps.querier.query_wasm_smart(
            manager_addr,
            &auction_package::msgs::AuctionsManagerQueryMsg::GetPrice {
                pair: config.pair.clone(),
            },
        )?;
        let time_diff_in_days =
            Decimal::from_atomics(env.block.time.seconds() - price.time.seconds(), 0)?
                / Decimal::from_atomics((60 * 60 * 24) as u128, 0)?;

        // Check our price is not older then 4 days
        if time_diff_in_days > config.price_freshness_strategy.limit {
            return Err(ContractError::PriceTooOld);
        }

        // We loop over all of our multipliers and find the first one that is smaller then our time diff
        // the list is sorted is the biggest is first
        // If no multiplier is found, we use the default which is 1
        let multiplier = config
            .price_freshness_strategy
            .multipliers
            .iter()
            .find(|(days, _)| &time_diff_in_days > days)
            .unwrap_or(&(Decimal::zero(), Decimal::one()))
            .1;

        // Calculate the new percentage of our strategy based on the freshness multiplier above
        // the max is 75% from the original price
        let start_price_perc =
            (Decimal::bps(auction_strategy.start_price_perc) * multiplier).min(Decimal::bps(7500));
        let end_price_perc =
            (Decimal::bps(auction_strategy.end_price_perc) * multiplier).min(Decimal::bps(7500));

        // Set prices based on strategy
        let price = price.price;
        let start_price = price + price * start_price_perc;
        let end_price = price - price * end_price_perc;
        Ok((start_price, end_price))
    }
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::GetConfig => {
            let config = AUCTION_CONFIG.load(deps.storage)?;
            Ok(to_binary(&config)?)
        }
        QueryMsg::GetFundsAmount { addr } => {
            let addr = deps.api.addr_validate(&addr)?;
            let auction_ids = AUCTION_IDS.load(deps.storage)?;
            let curr = AUCTION_FUNDS
                .load(deps.storage, (auction_ids.curr, addr.clone()))
                .unwrap_or_default();
            let next = AUCTION_FUNDS
                .load(deps.storage, (auction_ids.next, addr))
                .unwrap_or_default();

            to_binary(&GetFundsAmountResponse { curr, next })
        }
        QueryMsg::GetAuction => {
            let active_auction = ACTIVE_AUCTION.load(deps.storage)?;
            to_binary(&active_auction)
        }
        QueryMsg::GetPrice => {
            let active_auction = ACTIVE_AUCTION.load(deps.storage)?;
            let price = calc_price(&active_auction, env.block.height);
            to_binary(&GetPriceResponse {
                price,
                time: env.block.time,
            })
        }
        QueryMsg::GetStrategy => {
            let auction_strategy = AUCTION_STRATEGY.load(deps.storage)?;
            to_binary(&auction_strategy)
        }
    }
}

#[cfg(test)]
mod test {
    use std::str::FromStr;

    use cosmwasm_std::{Decimal, Uint128};

    use crate::helpers::calc_buy_amount;

    #[test]
    fn test_calc_buy_amount() {
        let price = Decimal::from_str("1.5").unwrap();
        // We send 4 token2, which should give us 2 token1 and 1 token2 refunded when price is 1.5
        let amount = Uint128::from(4_u128);

        let (buy_amount, refund_amount) = calc_buy_amount(price, amount);
        assert_eq!(buy_amount, Uint128::from(2_u128));
        assert_eq!(refund_amount, Uint128::from(1_u128));

        // We send 5 token2, which should give us 3 token1 and 0 token2 refunded.
        // because 3 token1 is 4.5 token2, we can't refund less then 1, so its rounded to 5
        let amount = Uint128::from(5_u128);

        let (buy_amount, refund_amount) = calc_buy_amount(price, amount);
        assert_eq!(buy_amount, Uint128::from(3_u128));
        assert_eq!(refund_amount, Uint128::from(0_u128));
    }
}
