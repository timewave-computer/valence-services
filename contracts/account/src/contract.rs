#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{
    BankMsg, Binary, CosmosMsg, Deps, DepsMut, Env, IbcMsg, MessageInfo, Reply, Response,
    StdResult, SubMsg, WasmMsg,
};
use cw2::set_contract_version;
use valence_package::helpers::{forward_to_services_manager, sender_is_a_service, verify_admin};
use valence_package::msgs::core_execute::{AccountBaseExecuteMsg, ServicesManagerExecuteMsg};
use valence_package::states::{ADMIN, SERVICES_MANAGER};

use crate::error::ContractError;
use crate::msg::{InstantiateMsg, MigrateMsg, QueryMsg};

const CONTRACT_NAME: &str = "crates.io:valence-account";
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

const EXECUTE_BY_SERVICE_REPLY_ID: u64 = 0;

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    msg: InstantiateMsg,
) -> Result<Response, ContractError> {
    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;

    // Set sender as admin, only admin can execute messages
    ADMIN.save(deps.storage, &info.sender)?;

    SERVICES_MANAGER.save(
        deps.storage,
        &deps.api.addr_validate(&msg.services_manager)?,
    )?;

    Ok(Response::default().add_attribute("method", "instantiate"))
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: AccountBaseExecuteMsg,
) -> Result<Response, ContractError> {
    match msg {
        // Register to a service and pass it the data
        AccountBaseExecuteMsg::RegisterToService { service_name, data } => {
            verify_admin(deps.as_ref(), &info)?;
            let services_manager_addr = SERVICES_MANAGER.load(deps.storage)?;
            Ok(forward_to_services_manager(
                services_manager_addr.to_string(),
                ServicesManagerExecuteMsg::RegisterToService { service_name, data },
            )?)
        }
        // unregister from a service
        AccountBaseExecuteMsg::DeregisterFromService { service_name } => {
            verify_admin(deps.as_ref(), &info)?;
            let services_manager_addr = SERVICES_MANAGER.load(deps.storage)?;
            Ok(forward_to_services_manager(
                services_manager_addr.to_string(),
                ServicesManagerExecuteMsg::DeregisterFromService { service_name },
            )?)
        }
        // Update the config for this service
        AccountBaseExecuteMsg::UpdateService { service_name, data } => {
            verify_admin(deps.as_ref(), &info)?;
            let services_manager_addr = SERVICES_MANAGER.load(deps.storage)?;
            Ok(forward_to_services_manager(
                services_manager_addr.to_string(),
                ServicesManagerExecuteMsg::UpdateService { service_name, data },
            )?)
        }
        // Pause the service
        AccountBaseExecuteMsg::PauseService { service_name } => {
            verify_admin(deps.as_ref(), &info)?;
            let services_manager_addr = SERVICES_MANAGER.load(deps.storage)?;
            Ok(forward_to_services_manager(
                services_manager_addr.to_string(),
                ServicesManagerExecuteMsg::PauseService {
                    service_name,
                    pause_for: env.contract.address.to_string(),
                },
            )?)
        }
        // Resume service
        AccountBaseExecuteMsg::ResumeService { service_name } => {
            verify_admin(deps.as_ref(), &info)?;
            let services_manager_addr = SERVICES_MANAGER.load(deps.storage)?;
            Ok(forward_to_services_manager(
                services_manager_addr.to_string(),
                ServicesManagerExecuteMsg::ResumeService {
                    service_name,
                    resume_for: env.contract.address.to_string(),
                },
            )?)
        }
        // Messages to be exected by the service, with sending funds.
        AccountBaseExecuteMsg::SendFundsByService { msgs, atomic } => {
            let services_manager_addr = SERVICES_MANAGER.load(deps.storage)?;
            sender_is_a_service(deps, &info, services_manager_addr.to_string())?;
            verify_cosmos_msg_with_funds(&msgs)?;

            // By default msgs are atomic, if 1 fails all fails
            // but services can explicitly set atomic to false
            // to allow the msgs to fail without failing the rest of the messages
            let msgs: Vec<SubMsg> = msgs
                .into_iter()
                .map(|msg| {
                    if atomic {
                        SubMsg::new(msg)
                    } else {
                        SubMsg::reply_on_error(msg, EXECUTE_BY_SERVICE_REPLY_ID)
                    }
                })
                .collect();

            Ok(Response::default().add_submessages(msgs))
        }
        // Messages to be exected by the service, without sending funds.
        AccountBaseExecuteMsg::ExecuteByService { msgs, atomic } => {
            let services_manager_addr = SERVICES_MANAGER.load(deps.storage)?;
            sender_is_a_service(deps, &info, services_manager_addr.to_string())?;
            verify_cosmos_msg(&msgs)?;

            let msgs: Vec<SubMsg> = msgs
                .into_iter()
                .map(|msg| {
                    if atomic {
                      SubMsg::new(msg)
                    } else {
                      SubMsg::reply_on_error(msg, EXECUTE_BY_SERVICE_REPLY_ID)
                    }
                })
                .collect();

            Ok(Response::default().add_submessages(msgs))
        }
        // Message to be executed by the admin of this account
        AccountBaseExecuteMsg::ExecuteByAdmin { msgs } => {
            verify_admin(deps.as_ref(), &info)?;
            Ok(Response::default().add_messages(msgs))
        }
    }
}

/// List and verify all messages the can be sent by an account which includes
/// sending funds (native or IBC)
fn verify_cosmos_msg_with_funds(msgs: &[CosmosMsg]) -> Result<(), ContractError> {
    msgs.iter().try_for_each(|msg| match msg {
        CosmosMsg::Bank(BankMsg::Send { amount: funds, .. })
        | CosmosMsg::Wasm(WasmMsg::Execute { funds, .. })
        | CosmosMsg::Wasm(WasmMsg::Instantiate { funds, .. }) => {
            // Check we have something in the array
            if funds.is_empty() {
                return Err(ContractError::ExpectedFunds);
            }

            // Check the coins are not empty
            if funds.iter().any(|c| c.amount.is_zero()) {
                return Err(ContractError::ExpectedFunds);
            }
            Ok(())
        }
        CosmosMsg::Ibc(IbcMsg::Transfer { amount: funds, .. }) => {
            if funds.amount.is_zero() {
                return Err(ContractError::ExpectedFunds);
            }
            Ok(())
        }
        _ => Err(ContractError::NotSupportedMessageWithFunds(
            stringify!(msg).to_string(),
        )),
    })
}

/// List and verify messages that can be sent by an account without sending funds
fn verify_cosmos_msg(msgs: &[CosmosMsg]) -> Result<(), ContractError> {
    msgs.iter().try_for_each(|msg| match msg {
        CosmosMsg::Ibc(IbcMsg::SendPacket { .. })
        | CosmosMsg::Wasm(WasmMsg::Execute { .. })
        | CosmosMsg::Wasm(WasmMsg::Instantiate { .. })
        | CosmosMsg::Wasm(WasmMsg::Migrate { .. }) => Ok(()),
        _ => Err(ContractError::NotSupportedMessage(
            stringify!(msg).to_string(),
        )),
    })
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(_deps: Deps, _env: Env, _msg: QueryMsg) -> StdResult<Binary> {
    // match msg {
    //     QueryMsg::IsQueued { address } => todo!(),
    // }
    unimplemented!()
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn reply(_deps: DepsMut, _env: Env, msg: Reply) -> Result<Response, ContractError> {
    // We allow services to send non atomic messages to be executed by the account.
    // This needs to be handled by the service, and make sure that messages really
    // can be non-atomic, otherwise unexpected behavior can happen.
    // Example case for this is the rebalancer service,
    // the rebalancer send trade messages to be executed by the account,
    // trade1 message doesn't rely on trade2 message, so they can be non-atomic.
    if msg.id != EXECUTE_BY_SERVICE_REPLY_ID {
        Err(ContractError::UnexpectedReplyId(msg.id))
    } else {
        Ok(Response::default()
            .add_attribute("method", "reply_on_error")
            .add_attribute("error", msg.result.unwrap_err()))
    }
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn migrate(_deps: DepsMut, _env: Env, _msg: MigrateMsg) -> Result<Response, ContractError> {
    unimplemented!()
}
