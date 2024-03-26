use auction_package::helpers::{approve_admin_change, GetPriceResponse};
use auction_package::msgs::AuctionsManagerQueryMsg;
use auction_package::states::{ADMIN, MIN_AUCTION_AMOUNT, ORACLE_ADDR, PAIRS};
#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{
    to_json_binary, Binary, Deps, DepsMut, Env, MessageInfo, Order, Reply, Response, StdResult,
    WasmMsg,
};
use cw2::set_contract_version;
use cw_storage_plus::Bound;
use cw_utils::{nonpayable, parse_reply_instantiate_data};

use crate::error::ContractError;
use crate::msg::{ExecuteMsg, InstantiateMsg, MigrateMsg};
use crate::state::AUCTION_CODE_ID;

const CONTRACT_NAME: &str = "crates.io:auctions-manager";
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

const INSTANTIATE_AUCTION_REPLY_ID: u64 = 1;

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    msg: InstantiateMsg,
) -> Result<Response, ContractError> {
    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;

    ADMIN.save(deps.storage, &info.sender)?;
    AUCTION_CODE_ID.save(deps.storage, &msg.auction_code_id)?;

    for min_amount in msg.min_auction_amount {
        MIN_AUCTION_AMOUNT.save(deps.storage, min_amount.0, &min_amount.1)?;
    }

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
        ExecuteMsg::AuctionFunds { pair } => {
            pair.verify()?;
            let pair_addr = PAIRS.load(deps.storage, pair)?;

            let msg = WasmMsg::Execute {
                contract_addr: pair_addr.to_string(),
                msg: to_json_binary(&auction::msg::ExecuteMsg::AuctionFundsManager {
                    sender: info.sender,
                })?,
                funds: info.funds,
            };

            Ok(Response::default().add_message(msg))
        }
        ExecuteMsg::WithdrawFunds { pair } => {
            nonpayable(&info)?;
            pair.verify()?;
            let pair_addr = PAIRS.load(deps.storage, pair)?;

            let msg = WasmMsg::Execute {
                contract_addr: pair_addr.to_string(),
                msg: to_json_binary(&auction::msg::ExecuteMsg::WithdrawFundsManager {
                    sender: info.sender,
                })?,
                funds: vec![],
            };

            Ok(Response::default().add_message(msg))
        }
        ExecuteMsg::FinishAuction { pair, limit } => {
            nonpayable(&info)?;
            pair.verify()?;
            let pair_addr = PAIRS.load(deps.storage, pair)?;

            let msg = WasmMsg::Execute {
                contract_addr: pair_addr.to_string(),
                msg: to_json_binary(&auction::msg::ExecuteMsg::FinishAuction { limit })?,
                funds: vec![],
            };

            Ok(Response::default().add_message(msg))
        }
        ExecuteMsg::Admin(admin_msg) => admin::handle_msg(deps, env, info, *admin_msg),
        ExecuteMsg::ApproveAdminChange {} => Ok(approve_admin_change(deps, &env, &info)?),
    }
}

mod admin {
    use auction_package::helpers::{cancel_admin_change, start_admin_change, verify_admin};
    use cosmwasm_std::{to_json_binary, SubMsg, WasmMsg};

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
            AdminMsgs::NewAuction { msg, min_amount } => {
                msg.pair.verify()?;

                // Make sure we either set min_amount, or have it in storage
                match min_amount {
                    Some(min_amount) => {
                        MIN_AUCTION_AMOUNT.save(deps.storage, msg.pair.0.clone(), &min_amount)?;
                        Ok::<_, ContractError>(())
                    }
                    None => {
                        MIN_AUCTION_AMOUNT
                            .load(deps.storage, msg.pair.0.clone())
                            .map_err(|_| {
                                ContractError::MustSetMinAuctionAmount(msg.pair.0.clone())
                            })?;
                        Ok(())
                    }
                }?;

                let init_msg = SubMsg::reply_on_success(
                    WasmMsg::Instantiate {
                        admin: Some(env.contract.address.to_string()),
                        code_id: AUCTION_CODE_ID.load(deps.storage)?,
                        msg: to_json_binary(&msg)?,
                        funds: vec![],
                        label: format!("auction-{}-{}", msg.pair.0, msg.pair.1),
                    },
                    INSTANTIATE_AUCTION_REPLY_ID,
                );

                Ok(Response::default().add_submessage(init_msg))
            }
            AdminMsgs::PauseAuction { pair } => {
                let pair_addr = PAIRS.load(deps.storage, pair)?;
                let msg = WasmMsg::Execute {
                    contract_addr: pair_addr.to_string(),
                    msg: to_json_binary(&auction::msg::ExecuteMsg::Admin(Box::new(
                        auction::msg::AdminMsgs::PauseAuction,
                    )))?,
                    funds: vec![],
                };

                Ok(Response::default().add_message(msg))
            }
            AdminMsgs::ResumeAuction { pair } => {
                let pair_addr = PAIRS.load(deps.storage, pair)?;
                let msg = WasmMsg::Execute {
                    contract_addr: pair_addr.to_string(),
                    msg: to_json_binary(&auction::msg::ExecuteMsg::Admin(Box::new(
                        auction::msg::AdminMsgs::ResumeAuction,
                    )))?,
                    funds: vec![],
                };

                Ok(Response::default().add_message(msg))
            }
            AdminMsgs::OpenAuction { pair, params } => {
                let pair_addr = PAIRS.load(deps.storage, pair)?;
                let msg = WasmMsg::Execute {
                    contract_addr: pair_addr.to_string(),
                    msg: to_json_binary(&auction::msg::ExecuteMsg::Admin(Box::new(
                        auction::msg::AdminMsgs::StartAuction(params),
                    )))?,
                    funds: vec![],
                };

                Ok(Response::default().add_message(msg))
            }
            AdminMsgs::UpdateAuctionId { code_id } => {
                AUCTION_CODE_ID.save(deps.storage, &code_id)?;

                Ok(Response::default())
            }
            AdminMsgs::UpdateOracle { oracle_addr } => {
                ORACLE_ADDR.save(deps.storage, &deps.api.addr_validate(&oracle_addr)?)?;

                Ok(Response::default())
            }
            AdminMsgs::UpdateStrategy { pair, strategy } => {
                let pair_addr = PAIRS.load(deps.storage, pair)?;
                let msg = WasmMsg::Execute {
                    contract_addr: pair_addr.to_string(),
                    msg: to_json_binary(&auction::msg::ExecuteMsg::Admin(Box::new(
                        auction::msg::AdminMsgs::UpdateStrategy { strategy },
                    )))?,
                    funds: vec![],
                };

                Ok(Response::default().add_message(msg))
            }
            AdminMsgs::UpdateChainHaltConfig { pair, halt_config } => {
                let pair_addr = PAIRS.load(deps.storage, pair)?;
                let msg = WasmMsg::Execute {
                    contract_addr: pair_addr.to_string(),
                    msg: to_json_binary(&auction::msg::ExecuteMsg::Admin(Box::new(
                        auction::msg::AdminMsgs::UpdateChainHaltConfig(halt_config),
                    )))?,
                    funds: vec![],
                };

                Ok(Response::default().add_message(msg))
            }
            AdminMsgs::UpdatePriceFreshnessStrategy { pair, strategy } => {
                let pair_addr = PAIRS.load(deps.storage, pair)?;
                let msg = WasmMsg::Execute {
                    contract_addr: pair_addr.to_string(),
                    msg: to_json_binary(&auction::msg::ExecuteMsg::Admin(Box::new(
                        auction::msg::AdminMsgs::UpdatePriceFreshnessStrategy(strategy),
                    )))?,
                    funds: vec![],
                };

                Ok(Response::default().add_message(msg))
            }
            AdminMsgs::MigrateAuction { pair, code_id, msg } => {
                let pair_addr = PAIRS.load(deps.storage, pair)?;

                let migrate_msg = WasmMsg::Migrate {
                    contract_addr: pair_addr.to_string(),
                    msg: to_json_binary(&msg)?,
                    new_code_id: code_id,
                };

                Ok(Response::default().add_message(migrate_msg))
            }
            AdminMsgs::UpdateMinAmount { denom, min_amount } => {
                MIN_AUCTION_AMOUNT.save(deps.storage, denom, &min_amount)?;

                Ok(Response::default())
            }
            AdminMsgs::StartAdminChange { addr, expiration } => {
                Ok(start_admin_change(deps, &info, &addr, expiration)?)
            }
            AdminMsgs::CancelAdminChange => Ok(cancel_admin_change(deps, &info)?),
        }
    }
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, _env: Env, msg: AuctionsManagerQueryMsg) -> StdResult<Binary> {
    match msg {
        AuctionsManagerQueryMsg::GetPairs { start_after, limit } => {
            let start_after = start_after.map(Bound::exclusive);
            let pairs = PAIRS
                .range(deps.storage, start_after, None, Order::Ascending)
                .take(limit.unwrap_or(50) as usize)
                .collect::<StdResult<Vec<_>>>()?;

            to_json_binary(&pairs)
        }
        AuctionsManagerQueryMsg::GetPairAddr { pair } => {
            to_json_binary(&PAIRS.load(deps.storage, pair)?)
        }
        AuctionsManagerQueryMsg::GetPrice { pair } => {
            let oracle_addr = ORACLE_ADDR
                .load(deps.storage)
                .map_err(|_| ContractError::OracleAddrMissing)?;

            to_json_binary(&deps.querier.query_wasm_smart::<GetPriceResponse>(
                oracle_addr,
                &price_oracle::msg::QueryMsg::GetPrice { pair },
            )?)
        }
        AuctionsManagerQueryMsg::GetOracleAddr => to_json_binary(&ORACLE_ADDR.load(deps.storage)?),
        AuctionsManagerQueryMsg::GetConfig { pair } => {
            let pair_addr = PAIRS.load(deps.storage, pair)?;
            deps.querier
                .query_wasm_smart(pair_addr, &auction::msg::QueryMsg::GetConfig)
        }
        AuctionsManagerQueryMsg::GetMinLimit { denom } => {
            to_json_binary(&MIN_AUCTION_AMOUNT.load(deps.storage, denom)?)
        }
        AuctionsManagerQueryMsg::GetAdmin => to_json_binary(&ADMIN.load(deps.storage)?),
    }
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn reply(deps: DepsMut, _env: Env, msg: Reply) -> Result<Response, ContractError> {
    match msg.id {
        INSTANTIATE_AUCTION_REPLY_ID => {
            let auction_addr = deps
                .api
                .addr_validate(&parse_reply_instantiate_data(msg)?.contract_address)?;
            let auction_config: auction_package::helpers::AuctionConfig = deps
                .querier
                .query_wasm_smart(auction_addr.clone(), &auction::msg::QueryMsg::GetConfig)?;

            PAIRS.save(deps.storage, auction_config.pair, &auction_addr)?;

            Ok(Response::default())
        }
        _ => Err(ContractError::UnknownReplyId(msg.id)),
    }
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn migrate(deps: DepsMut, _env: Env, msg: MigrateMsg) -> Result<Response, ContractError> {
    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;

    match msg {
        MigrateMsg::NoStateChange {} => Ok(Response::default()),
    }
}
