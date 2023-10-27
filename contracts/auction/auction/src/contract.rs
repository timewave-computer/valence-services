use std::collections::VecDeque;

use auction_package::helpers::{verify_admin, AuctionConfig, ChainHaltConfig, GetPriceResponse};
use auction_package::states::{ADMIN, MIN_AUCTION_AMOUNT, TWAP_PRICES};
use auction_package::Price;
#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{
    coin, to_binary, Addr, BankMsg, Binary, BlockInfo, Coin, CosmosMsg, Decimal, Deps, DepsMut,
    Env, MessageInfo, Reply, Response, StdResult, Uint128,
};
use cw2::set_contract_version;
use cw_storage_plus::Bound;
use cw_utils::must_pay;

use crate::error::ContractError;
use crate::msg::{
    ExecuteMsg, GetFundsAmountResponse, InstantiateMsg, MigrateMsg, NewAuctionParams, QueryMsg,
};
use crate::state::{
    ActiveAuction, ActiveAuctionStatus, AuctionIds, ACTIVE_AUCTION, AUCTION_CONFIG, AUCTION_FUNDS,
    AUCTION_FUNDS_SUM, AUCTION_IDS, AUCTION_STRATEGY,
};

const CONTRACT_NAME: &str = "crates.io:auction";
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

const TWAP_PRICE_LIMIT: u64 = 10;

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
            auction_funds(deps, &info, sender)
        }
        ExecuteMsg::WithdrawFundsManager { sender } => {
            verify_admin(deps.as_ref(), &info)?;
            withdraw_funds(deps, sender)
        }
        ExecuteMsg::AuctionFunds => auction_funds(deps, &info, info.sender.clone()),
        ExecuteMsg::WithdrawFunds => withdraw_funds(deps, info.sender),
        ExecuteMsg::Admin(admin_msg) => admin::handle_msg(deps, env, info, admin_msg),
        ExecuteMsg::Bid => do_bid(deps, &info, &env),
        ExecuteMsg::FinishAuction { limit } => finish_auction(deps, &env, limit),
        ExecuteMsg::CleanAfterAuction => clean_auction(deps),
    }
}

pub(crate) fn auction_funds(
    deps: DepsMut,
    info: &MessageInfo,
    sender: Addr,
) -> Result<Response, ContractError> {
    let config = AUCTION_CONFIG.load(deps.storage)?;
    let admin = ADMIN.load(deps.storage)?;

    if config.is_paused {
        return Err(ContractError::AuctionIsPaused);
    }

    let funds = must_pay(info, &config.pair.0)?;
    let min_amount = match MIN_AUCTION_AMOUNT.query(&deps.querier, admin, config.pair.0)? {
        Some(amount) => Ok(amount),
        None => Err(ContractError::NoTokenMinAmount),
    }?;

    if funds < min_amount {
        return Err(ContractError::AuctionAmountTooLow(min_amount));
    }

    let next_auction_id: u64 = AUCTION_IDS.load(deps.storage)?.next;

    // Update funds of the sender for next auction
    AUCTION_FUNDS.update(
        deps.storage,
        (next_auction_id, sender),
        |amount| -> Result<Uint128, ContractError> {
            match amount {
                Some(amount) => Ok(amount.checked_add(funds)?),
                None => Ok(funds),
            }
        },
    )?;

    // update the sum of the next auction
    AUCTION_FUNDS_SUM.update(
        deps.storage,
        next_auction_id,
        |amount| -> Result<Uint128, ContractError> {
            match amount {
                Some(amount) => Ok(amount.checked_add(funds)?),
                None => Ok(funds),
            }
        },
    )?;

    Ok(Response::default())
}

pub fn withdraw_funds(deps: DepsMut, sender: Addr) -> Result<Response, ContractError> {
    let config = AUCTION_CONFIG.load(deps.storage)?;

    let mut send_funds: Coin = coin(0_u128, config.pair.0);
    let auction_ids = AUCTION_IDS.load(deps.storage)?;

    let funds_amount = AUCTION_FUNDS
        .load(deps.storage, (auction_ids.next, sender.clone()))
        .unwrap_or(Uint128::zero());

    if !funds_amount.is_zero() {
        send_funds.amount += funds_amount;
        AUCTION_FUNDS.remove(deps.storage, (auction_ids.next, sender.clone()));
        AUCTION_FUNDS_SUM.update(
            deps.storage,
            auction_ids.next,
            |sum| -> Result<Uint128, ContractError> {
                Ok(sum.unwrap_or(Uint128::zero()).checked_sub(funds_amount)?)
            },
        )?;
    }

    if send_funds.amount.is_zero() {
        return Err(ContractError::NoFundsToWithdraw);
    }

    let bank_msg = BankMsg::Send {
        to_address: sender.to_string(),
        amount: vec![send_funds],
    };

    Ok(Response::default().add_message(bank_msg))
}

/// Check the diff of blocks and time to see if we had a chain halt of around our time_cap
fn is_chain_halted(env: &Env, check_block: &BlockInfo, halt_config: &ChainHaltConfig) -> bool {
    let block_diff = Uint128::from(env.block.height - check_block.height);
    let time_diff = (env.block.time.seconds() - check_block.time.seconds()) as u128;

    let avg_time_passed = (block_diff * halt_config.block_avg).u128();

    // Chain halted for at least 4 hours
    if time_diff > avg_time_passed + halt_config.cap {
        return true;
    }
    false
}

fn do_bid(deps: DepsMut, info: &MessageInfo, env: &Env) -> Result<Response, ContractError> {
    // Verify we have an active auction, else error out
    let mut active_auction = ACTIVE_AUCTION.load(deps.storage)?;

    // Verify auction is not finished
    match active_auction.status {
        ActiveAuctionStatus::Started => Ok(()),
        _ => Err(ContractError::AuctionFinished),
    }?;

    // Verify auction started
    if active_auction.start_block > env.block.height {
        return Err(ContractError::AuctionNotStarted(active_auction.start_block));
    }

    let config = AUCTION_CONFIG.load(deps.storage)?;

    if config.is_paused {
        return Err(ContractError::AuctionIsPaused);
    }

    let sent_funds = must_pay(info, &config.pair.1)?;

    let (buy_amount, leftover_amount) = if is_chain_halted(
        env,
        &active_auction.last_checked_block,
        &config.chain_halt_config,
    ) {
        active_auction.status = ActiveAuctionStatus::Finished;
        (Uint128::zero(), sent_funds)
    } else {
        let curr_price = calc_price(&active_auction, env.block.height);
        let (buy_amount, mut send_leftover) = calc_buy_amount(curr_price, sent_funds);

        let send_amount = match active_auction.available_amount.checked_sub(buy_amount) {
            Ok(available_amount) => {
                if available_amount.is_zero() {
                    active_auction.status = ActiveAuctionStatus::Finished;
                }

                active_auction.available_amount = available_amount;
                active_auction.resolved_amount += sent_funds - send_leftover;

                Ok::<Uint128, ContractError>(buy_amount)
            }
            // If we reach here, it means that we have sold all of the available amount
            // so we can finish the auction
            Err(_) => {
                let to_refund =
                    Decimal::from_atomics(buy_amount - active_auction.available_amount, 0)?;
                let new_leftover = (to_refund * curr_price).to_uint_floor();
                send_leftover += new_leftover;

                // Set the buy_amount to be whatever amount we have left, because we know the bidder over paid
                let buy_amount = active_auction.available_amount;

                active_auction.resolved_amount += sent_funds - send_leftover;
                active_auction.available_amount = Uint128::zero();
                active_auction.status = ActiveAuctionStatus::Finished;

                Ok(buy_amount)
            }
        }?;
        (send_amount, send_leftover)
    };

    let mut send_funds: Vec<Coin> = vec![];

    if !leftover_amount.is_zero() {
        send_funds.push(coin(leftover_amount.u128(), config.pair.1.clone()));
    }

    if !buy_amount.is_zero() {
        send_funds.push(coin(buy_amount.u128(), config.pair.0));
    }

    let response = if !send_funds.is_empty() {
        let bank_msg = BankMsg::Send {
            to_address: info.sender.to_string(),
            amount: send_funds,
        };
        Response::default().add_message(bank_msg)
    } else {
        Response::default()
    };

    active_auction.last_checked_block = env.block.clone();
    ACTIVE_AUCTION.save(deps.storage, &active_auction)?;

    Ok(
        response
            .add_attribute("bought_amount", buy_amount) // pair.0 amount we sent
            .add_attribute("refunded", leftover_amount), // pair.1 amount we refunded
    )
}

fn finish_auction(deps: DepsMut, env: &Env, limit: u64) -> Result<Response, ContractError> {
    let mut active_auction = ACTIVE_AUCTION.load(deps.storage)?;

    if active_auction.status == ActiveAuctionStatus::Started
        && active_auction.end_block > env.block.height
        && !active_auction.available_amount.is_zero()
    {
        return Err(ContractError::AuctionStillGoing);
    }

    let (start_from, mut total_sent_sold_token, mut total_sent_bought_token) = match active_auction
        .status
    {
        ActiveAuctionStatus::CloseAuction(addr, total_sent_sold_token, total_sent_bought_token) => {
            Ok((addr, total_sent_sold_token, total_sent_bought_token))
        }
        ActiveAuctionStatus::AuctionClosed => Err(ContractError::AuctionClosed),
        ActiveAuctionStatus::Finished | ActiveAuctionStatus::Started => {
            Ok((None, Uint128::zero(), Uint128::zero()))
        }
    }?;

    let config = AUCTION_CONFIG.load(deps.storage)?;
    let curr_auction_id = AUCTION_IDS.load(deps.storage)?.curr;
    let mut last_resolved = start_from.clone();
    let start_from = start_from.map(Bound::exclusive);
    let mut total_resolved = 0;

    let mut bank_msgs: Vec<CosmosMsg> = vec![];

    AUCTION_FUNDS
        .prefix(curr_auction_id)
        .range(
            deps.storage,
            start_from,
            None,
            cosmwasm_std::Order::Ascending,
        )
        .take(limit as usize)
        .try_for_each(|res| -> Result<(), ContractError> {
            total_resolved += 1;
            let (addr, amount) = res?;
            let mut send_funds: Vec<Coin> = vec![];

            if active_auction.resolved_amount.is_zero() {
                // We didn't sell anything, so refund
                send_funds.push(coin(amount.u128(), &config.pair.0));
            } else {
                // We sold something, calculate only what we sold
                let perc_of_total = Decimal::from_atomics(amount, 0)?
                    / Decimal::from_atomics(active_auction.total_amount, 0)?;
                let to_send_amount =
                    Decimal::from_atomics(active_auction.resolved_amount, 0)? * perc_of_total;

                // TODO: Verify this is correct
                let to_send_amount =
                    if to_send_amount - to_send_amount.floor() >= Decimal::bps(9999) {
                        to_send_amount.to_uint_ceil()
                    } else {
                        to_send_amount.to_uint_floor()
                    };

                total_sent_bought_token += to_send_amount;
                send_funds.push(coin(to_send_amount.u128(), &config.pair.1));

                // If we still have available amount, we refund based on the perc from total provided
                if !active_auction.available_amount.is_zero() {
                    let to_send_amount =
                        (Decimal::from_atomics(active_auction.available_amount, 0)?
                            * perc_of_total)
                            .to_uint_floor();
                    total_sent_sold_token += to_send_amount;
                    send_funds.push(coin(to_send_amount.u128(), &config.pair.0));
                }
            }

            let send_msg = BankMsg::Send {
                to_address: addr.to_string(),
                amount: send_funds,
            };
            bank_msgs.push(send_msg.into());

            last_resolved = Some(addr);
            Ok(())
        })?;

    // If we looped over less than our limit, it means we resolved everything
    let status = if total_resolved < limit {
        // calculate if we have leftover from rounding and add it to the next auction
        let leftover_sold_token = active_auction
            .available_amount
            .checked_sub(total_sent_sold_token)?;
        let leftover_bought_token = active_auction
            .resolved_amount
            .checked_sub(total_sent_bought_token)?;

        active_auction.leftovers[0] = leftover_sold_token;
        active_auction.leftovers[1] = leftover_bought_token;

        // Update twap price if we have something sold
        let sold_amount = active_auction
            .total_amount
            .checked_sub(active_auction.available_amount)?;
        if !active_auction.total_amount.is_zero() && !sold_amount.is_zero() {
            let avg_price = Decimal::from_atomics(active_auction.resolved_amount, 0)?
                .checked_div(Decimal::from_atomics(sold_amount, 0)?)?;

            let mut prices = TWAP_PRICES.load(deps.storage)?;
            prices.push_front(Price {
                price: avg_price,
                time: env.block.time,
            });

            if prices.len() > TWAP_PRICE_LIMIT as usize {
                prices.pop_back();
            }
            TWAP_PRICES.save(deps.storage, &prices)?;
        }

        ActiveAuctionStatus::AuctionClosed
    } else {
        ActiveAuctionStatus::CloseAuction(
            last_resolved,
            total_sent_sold_token,
            total_sent_bought_token,
        )
    };

    active_auction.status = status;
    ACTIVE_AUCTION.save(deps.storage, &active_auction)?;

    Ok(Response::default().add_messages(bank_msgs))
}

fn clean_auction(deps: DepsMut) -> Result<Response, ContractError> {
    let active_auction = ACTIVE_AUCTION.load(deps.storage)?;

    match active_auction.status {
        ActiveAuctionStatus::AuctionClosed => Ok::<_, ContractError>(()),
        _ => return Err(ContractError::AuctionNotClosed),
    }?;

    let curr_auction_id = AUCTION_IDS.load(deps.storage)?.curr;

    // Clean the funds at the id of ended auction
    AUCTION_FUNDS
        .prefix(curr_auction_id)
        .clear(deps.storage, None);
    // Clean the funds sum
    AUCTION_FUNDS_SUM.remove(deps.storage, curr_auction_id);

    Ok(Response::default())
}

fn calc_price(terms: &ActiveAuction, curr_height: u64) -> Decimal {
    let block_diff = Decimal::from_atomics(terms.end_block - terms.start_block, 0).unwrap();
    let price_diff = terms.start_price - terms.end_price;

    let price_per_block = price_diff / block_diff;
    let block_passed = Decimal::from_atomics(curr_height - terms.start_block, 0).unwrap();

    terms.start_price - (price_per_block * block_passed)
}

/// Calc how much of pair.0 to send (bought amount) and how much pair.1 to refund (leftover)
fn calc_buy_amount(price: Decimal, amount: Uint128) -> (Uint128, Uint128) {
    let amount = Decimal::from_atomics(amount, 0).unwrap();

    let buy_amount = amount / price;
    let buy_floor = buy_amount.floor();
    let leftover = (amount - (buy_floor * price)).to_uint_floor();

    (buy_floor.to_uint_floor(), leftover)
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
            start_block: new_auction_params.start_block.unwrap_or(env.block.height),
            end_block: new_auction_params.end_block,
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

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn reply(_deps: DepsMut, _env: Env, _msg: Reply) -> Result<Response, ContractError> {
    // Tick messages are dispatched with reply ID 0 and reply on
    // error. If an error occurs, we ignore it but stop the parent
    // message from failing, so the state change which moved the tick
    // receiver to the end of the message queue gets committed. This
    // prevents an erroring tick receiver from locking the clock.
    unimplemented!()
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn migrate(_deps: DepsMut, _env: Env, _msg: MigrateMsg) -> Result<Response, ContractError> {
    unimplemented!()
}

#[cfg(test)]
mod test {
    use std::str::FromStr;

    use cosmwasm_std::{Decimal, Uint128};

    use crate::contract::calc_buy_amount;

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
