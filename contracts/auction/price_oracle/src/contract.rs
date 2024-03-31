use std::collections::VecDeque;

use auction_package::helpers::{
    approve_admin_change, cancel_admin_change, start_admin_change, verify_admin,
};
use auction_package::states::{ADMIN, PAIRS, PRICES, TWAP_PRICES};
use auction_package::Price;
#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{to_json_binary, Binary, Decimal, Deps, DepsMut, Env, MessageInfo, Response};
use cw2::set_contract_version;

use crate::error::ContractError;
use crate::msg::{ExecuteMsg, InstantiateMsg, MigrateMsg, QueryMsg};
use crate::state::{Config, PriceStep, ASTRO_PRICE_PATHS, CONFIG, CONFIG_V0};

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
            seconds_allow_manual_change: msg.seconds_allow_manual_change,
            seconds_auction_prices_fresh: msg.seconds_auction_prices_fresh,
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
        ExecuteMsg::UpdatePrice { pair } => {
            pair.verify()?;

            let config = CONFIG.load(deps.storage)?;

            let auction_addr = PAIRS
                .query(
                    &deps.querier,
                    config.auction_manager_addr.clone(),
                    pair.clone(),
                )?
                .ok_or(ContractError::PairAuctionNotFound)?;
            let twap_prices = TWAP_PRICES.query(&deps.querier, auction_addr)?;

            let price = if can_update_price_from_auction(&config, &env, &twap_prices) {
                get_avg_price(twap_prices)
            } else {
                let steps = ASTRO_PRICE_PATHS
                    .load(deps.storage, pair.clone())
                    .map_err(|_| ContractError::NoAstroPath(pair.clone()))?;
                get_price_from_astroport(deps.as_ref(), &env, steps)?
            };

            // Save price
            PRICES.save(deps.storage, pair, &price)?;

            Ok(Response::default().add_attribute("price", price.to_string()))
        }
        ExecuteMsg::ManualPriceUpdate { pair, price } => {
            let config = CONFIG.load(deps.storage)?;
            verify_admin(deps.as_ref(), &info)?;

            pair.verify()?;

            // sanity check
            if price.is_zero() {
                return Err(ContractError::PriceIsZero);
            }

            // Get the time last update happened
            match PRICES.load(deps.storage, pair.clone()) {
                Ok(Price {
                    time: last_updated, ..
                }) => {
                    // Verify enough time has passed since last update to allow manual update
                    // 'enough time' is defined in the config
                    if env.block.time.seconds()
                        < last_updated.seconds() + config.seconds_allow_manual_change
                    {
                        Err(ContractError::NoTermsForManualUpdate)
                    } else {
                        Ok(())
                    }
                }
                Err(_) => Ok(()),
            }?;

            // Save price
            PRICES.save(
                deps.storage,
                pair,
                &Price {
                    price,
                    time: env.block.time,
                },
            )?;

            Ok(Response::default().add_attribute("price", price.to_string()))
        }
        ExecuteMsg::AddAstroPath { pair, path } => {
            verify_admin(deps.as_ref(), &info)?;

            pair.verify()?;

            if ASTRO_PRICE_PATHS.has(deps.storage, pair.clone()) {
                return Err(ContractError::PricePathAlreadyExists);
            }

            if path.is_empty() {
                return Err(ContractError::PricePathIsEmpty);
            }

            if path[0].denom1 != pair.0 || path[path.len() - 1].denom2 != pair.1 {
                return Err(ContractError::PricePathIsWrong);
            }

            ASTRO_PRICE_PATHS.save(deps.storage, pair, &path)?;

            Ok(Response::default())
        }
        ExecuteMsg::UpdateAstroPath { pair, path } => {
            verify_admin(deps.as_ref(), &info)?;

            pair.verify()?;

            if !ASTRO_PRICE_PATHS.has(deps.storage, pair.clone()) {
                return Err(ContractError::PricePathNotFound);
            }

            if path.is_empty() {
                return Err(ContractError::PricePathIsEmpty);
            }

            if path[0].denom1 != pair.0 || path[path.len() - 1].denom2 != pair.1 {
                return Err(ContractError::PricePathIsWrong);
            }

            ASTRO_PRICE_PATHS.save(deps.storage, pair, &path)?;

            Ok(Response::default())
        }
        ExecuteMsg::StartAdminChange { addr, expiration } => {
            Ok(start_admin_change(deps, &info, &addr, expiration)?)
        }
        ExecuteMsg::CancelAdminChange {} => Ok(cancel_admin_change(deps, &info)?),
        ExecuteMsg::ApproveAdminChange {} => Ok(approve_admin_change(deps, &env, &info)?),
    }
}

fn can_update_price_from_auction(
    config: &Config,
    env: &Env,
    auction_prices: &VecDeque<Price>,
) -> bool {
    if auction_prices.len() < 3 {
        return false;
    }

    // Make sure last auction ran in the acceptable time frame
    // else we consider the auction prices not up to date
    if auction_prices[0].time.seconds() + config.seconds_auction_prices_fresh
        < env.block.time.seconds()
    {
        return false;
    }

    true
}

fn get_avg_price(vec: VecDeque<Price>) -> Price {
    let (total_count, prices_sum) = vec.iter().fold(
        (Decimal::zero(), Decimal::zero()),
        |(total_count, prices_sum), price| (total_count + Decimal::one(), prices_sum + price.price),
    );

    Price {
        price: prices_sum / total_count,
        time: vec[0].time,
    }
}

fn get_price_from_astroport(
    deps: Deps,
    env: &Env,
    steps: Vec<PriceStep>,
) -> Result<Price, ContractError> {
    let final_denom_amount = steps.iter().fold(
        Decimal::from_atomics(1000000_u128, 0).map_err(ContractError::DecimalRangeExceeded),
        |amount, step| {
            // Build the asset
            let offer_asset = astroport::asset::Asset {
                info: astroport::asset::AssetInfo::NativeToken {
                    denom: step.denom1.clone(),
                },
                amount: amount?.to_uint_floor(),
            };

            let res = astroport::querier::simulate(
                &deps.querier,
                step.pool_address.clone(),
                &offer_asset,
            )?;

            let price = Decimal::from_atomics(
                res.return_amount
                    .checked_add(res.commission_amount)?
                    .checked_add(res.spread_amount)?,
                0,
            )?;
            deps.api.debug(format!("res: {:?}", res).as_str());
            deps.api.debug(format!("Price step: {:?}", price).as_str());

            Ok(price)
        },
    )?;

    let _price = final_denom_amount.checked_div(Decimal::from_atomics(1000000_u128, 0)?)?;
    deps.api.debug(format!("Price: {:?}", _price).as_str());

    let price = Price {
        price: final_denom_amount.checked_div(Decimal::from_atomics(1000000_u128, 0)?)?,
        time: env.block.time,
    };

    Ok(price)
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> Result<Binary, ContractError> {
    match msg {
        QueryMsg::GetPrice { pair } => {
            let price = PRICES.load(deps.storage, pair)?;

            Ok(to_json_binary(&price)?)
        }
        QueryMsg::GetConfig => {
            let config = CONFIG.load(deps.storage)?;
            Ok(to_json_binary(&config)?)
        }
        QueryMsg::GetAdmin => Ok(to_json_binary(&ADMIN.load(deps.storage)?)?),
    }
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn migrate(deps: DepsMut, _env: Env, msg: MigrateMsg) -> Result<Response, ContractError> {
    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;

    match msg {
        MigrateMsg::NoStateChange {} => Ok(Response::default()),
        MigrateMsg::ToV1 {} => {
            let config_v0 = CONFIG_V0.load(deps.storage)?;
            let config = Config {
                auction_manager_addr: config_v0.auction_manager_addr,
                seconds_allow_manual_change: 60 * 60 * 24 * 2, // 2 days
                seconds_auction_prices_fresh: 60 * 60 * 24 * 3, // 3 days
            };

            CONFIG.save(deps.storage, &config)?;
            Ok(Response::default())
        }
    }
}
