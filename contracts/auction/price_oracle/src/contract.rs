use auction_package::helpers::{
    approve_admin_change, cancel_admin_change, start_admin_change, verify_admin,
};
use auction_package::states::{ADMIN, PAIRS, PRICES, TWAP_PRICES};
use auction_package::Price;
#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{
    to_binary, Binary, Decimal, Deps, DepsMut, Env, MessageInfo, Response, Timestamp,
};
use cw2::set_contract_version;

use crate::error::ContractError;
use crate::msg::{ExecuteMsg, InstantiateMsg, QueryMsg};
use crate::state::{Config, CONFIG};

const CONTRACT_NAME: &str = "crates.io:oracle";
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    msg: InstantiateMsg,
) -> Result<Response, ContractError> {
    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;

    ADMIN.save(deps.storage, &info.sender)?;

    // Set config
    CONFIG.save(
        deps.storage,
        &Config {
            auction_manager_addr: deps.api.addr_validate(&msg.auctions_manager_addr)?,
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
        ExecuteMsg::UpdatePrice { pair, price } => {
            pair.verify()?;

            let config = CONFIG.load(deps.storage)?;
            verify_admin(deps.as_ref(), &info)?;

            let (price, time) = match price {
                // We have a price, so set that as the price of the pair
                Some(price) => {
                    if price.is_zero() {
                        return Err(ContractError::PriceIsZero);
                    }

                    Ok::<(Decimal, Timestamp), ContractError>((price, env.block.time))
                }
                // We don't have a price, so we are trying to look for the price in the auction
                None => {
                    let auction_addr = PAIRS
                        .query(&deps.querier, config.auction_manager_addr, pair.clone())?
                        .ok_or(ContractError::PairAuctionNotFound)?;
                    let twap_prices = TWAP_PRICES.query(&deps.querier, auction_addr)?;

                    if twap_prices.len() < 3 {
                        return Err(ContractError::NotEnoughTwaps);
                    }

                    // Check if we had an auction in the last 3 days and 6 hours
                    // 6 hours is a little buffer in case our auction end time doesn't match exactly our update time
                    if twap_prices[0].time.seconds()
                        < env.block.time.seconds() - (60 * 60 * 24 * 3 + (60 * 60 * 6))
                    {
                        return Err(ContractError::NoAuctionInLast3Days);
                    }

                    let (total_count, prices_sum) = twap_prices.iter().fold(
                        (Decimal::zero(), Decimal::zero()),
                        |(total_count, prices_sum), price| {
                            (total_count + Decimal::one(), prices_sum + price.price)
                        },
                    );

                    Ok((prices_sum / total_count, twap_prices[0].time))
                }
            }?;

            // Save price
            PRICES.save(deps.storage, pair, &Price { price, time })?;

            Ok(Response::default().add_attribute("price", price.to_string()))
        }
        ExecuteMsg::StartAdminChange { addr, expiration } => {
            Ok(start_admin_change(deps, &info, &addr, expiration)?)
        }
        ExecuteMsg::CancelAdminChange => Ok(cancel_admin_change(deps, &info)?),
        ExecuteMsg::ApproveAdminChange => Ok(approve_admin_change(deps, &env, &info)?),
    }
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> Result<Binary, ContractError> {
    match msg {
        QueryMsg::GetPrice { pair } => {
            let price = PRICES.load(deps.storage, pair)?;

            Ok(to_binary(&price)?)
        }
        QueryMsg::GetConfig => {
            let config = CONFIG.load(deps.storage)?;
            Ok(to_binary(&config)?)
        }
        QueryMsg::GetAdmin => Ok(to_binary(&ADMIN.load(deps.storage)?)?),
    }
}
