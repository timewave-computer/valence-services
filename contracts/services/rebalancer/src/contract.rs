#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{
    to_binary, Binary, Decimal, Deps, DepsMut, Env, MessageInfo, Reply, Response, StdResult,
};
use cw2::set_contract_version;
use valence_package::helpers::{verify_services_manager, OptionalField};
use valence_package::services::rebalancer::RebalancerExecuteMsg;
use valence_package::states::{ADMIN, SERVICES_MANAGER};

use crate::error::ContractError;
use crate::helpers::has_dup;
use crate::msg::{InstantiateMsg, MigrateMsg, QueryMsg};
use crate::rebalance::execute_system_rebalance;
use crate::state::{
    SystemRebalanceStatus, AUCTIONS_MANAGER_ADDR, BASE_DENOM_WHITELIST, CONFIGS, DENOM_WHITELIST,
    SYSTEM_REBALANCE_STATUS,
};

const CONTRACT_NAME: &str = "crates.io:covenant-clock";
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

// TODO: Make cycle period configurable
pub const CYCLE_PERIOD: u64 = 60 * 60 * 24; // 24 hours
pub const DEFAULT_LIMIT: u64 = 50;

pub const REPLY_DEFAULT_REBALANCE: u64 = 0;

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    msg: InstantiateMsg,
) -> Result<Response, ContractError> {
    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;

    // Set the admin
    ADMIN.save(deps.storage, &info.sender)?;

    // Save status as not started
    SYSTEM_REBALANCE_STATUS.save(
        deps.storage,
        &SystemRebalanceStatus::NotStarted {
            cycle_start: msg.cycle_start,
        },
    )?;

    // Set the services manager
    SERVICES_MANAGER.save(
        deps.storage,
        &deps.api.addr_validate(&msg.services_manager_addr)?,
    )?;

    // Set our whitelist
    DENOM_WHITELIST.save(deps.storage, &msg.denom_whitelist)?;
    BASE_DENOM_WHITELIST.save(deps.storage, &msg.base_denom_whitelist)?;

    // save auction addr
    AUCTIONS_MANAGER_ADDR.save(
        deps.storage,
        &deps.api.addr_validate(&msg.auctions_manager_addr)?,
    )?;

    Ok(Response::default().add_attribute("method", "instantiate"))
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: RebalancerExecuteMsg,
) -> Result<Response, ContractError> {
    match msg {
        RebalancerExecuteMsg::Register { register_for, data } => {
            verify_services_manager(deps.as_ref(), &info)?;
            let data = data.expect("We must have register data");
            let registree = deps.api.addr_validate(&register_for)?;

            if CONFIGS.has(deps.storage, registree.clone()) {
                return Err(ContractError::AccountAlreadyRegistered);
            }

            // check base denom is whitelisted
            let base_denom_whitelist = BASE_DENOM_WHITELIST.load(deps.storage)?;
            if !base_denom_whitelist.contains(&data.base_denom) {
                return Err(ContractError::BaseDenomNotWhitelisted(data.base_denom));
            }

            // Verify we have at least 2 targets
            if data.targets.len() < 2 {
                return Err(ContractError::TwoTargetsMinimum);
            }

            // check target denoms are whitelisted
            let denom_whitelist = DENOM_WHITELIST.load(deps.storage)?;
            let mut total_bps = 0;
            let mut has_min_balance = false;

            // Make sure denom is unique
            if has_dup(&data.targets) {
                return Err(ContractError::TargetsMustBeUnique);
            };

            for target in data.targets.clone() {
                total_bps += target.percentage;

                // Verify we only have a single min_Balance target
                if target.min_balance.is_some() && has_min_balance {
                    return Err(ContractError::MultipleMinBalanceTargets);
                } else if target.min_balance.is_some() {
                    has_min_balance = true;
                }

                // Verify the target is whitelisted
                if !denom_whitelist.contains(&target.denom) {
                    return Err(ContractError::DenomNotWhitelisted(target.denom));
                }
            }
            if total_bps != 10000 {
                return Err(ContractError::InvalidTargetPercentage(
                    Decimal::bps(total_bps).to_string(),
                ));
            }

            // save config
            CONFIGS.save(deps.storage, registree, &data.to_config()?)?;

            Ok(Response::default())
        }
        RebalancerExecuteMsg::Deregister { deregister_for } => {
            verify_services_manager(deps.as_ref(), &info)?;
            CONFIGS.remove(deps.storage, deps.api.addr_validate(&deregister_for)?);

            Ok(Response::default())
        }
        RebalancerExecuteMsg::Update { update_for, data } => {
            verify_services_manager(deps.as_ref(), &info)?;
            let account = deps.api.addr_validate(&update_for)?;
            let mut config = CONFIGS.load(deps.storage, account.clone())?;

            if let Some(trustee) = data.trustee {
                match trustee {
                    OptionalField::Set(trustee) => config.trustee = Some(trustee),
                    OptionalField::Clear => config.trustee = None,
                };
            }

            if let Some(base_denom) = data.base_denom {
                if !BASE_DENOM_WHITELIST
                    .load(deps.storage)?
                    .contains(&base_denom)
                {
                    return Err(ContractError::BaseDenomNotWhitelisted(base_denom));
                }
                config.base_denom = base_denom;
            }

            if !data.targets.is_empty() {
                let denom_whitelist = DENOM_WHITELIST.load(deps.storage)?;
                let mut total_bps = 0;
                let mut has_min_balance = false;

                for target in data.targets.clone() {
                    total_bps += target.percentage;

                    if target.min_balance.is_some() && has_min_balance {
                        return Err(ContractError::MultipleMinBalanceTargets);
                    } else if target.min_balance.is_some() {
                        has_min_balance = true;
                    }

                    if !denom_whitelist.contains(&target.denom) {
                        return Err(ContractError::DenomNotWhitelisted(target.denom));
                    }
                }

                if total_bps != 10000 {
                    return Err(ContractError::InvalidTargetPercentage(
                        Decimal::bps(total_bps).to_string(),
                    ));
                }

                config.has_min_balance = has_min_balance;
                config.targets = data.targets.into_iter().map(|t| t.into()).collect();
            }

            if let Some(pid) = data.pid {
                config.pid = pid.into_parsed()?;
            }

            if let Some(max_limit) = data.max_limit {
                config.max_limit = Decimal::bps(max_limit);
            }

            CONFIGS.save(deps.storage, account, &config)?;

            Ok(Response::default())
        }
        RebalancerExecuteMsg::Pause { pause_for, sender } => {
            verify_services_manager(deps.as_ref(), &info)?;
            let account = deps.api.addr_validate(&pause_for)?;
            let sender = deps.api.addr_validate(&sender)?;

            let mut config = CONFIGS.load(deps.storage, account.clone())?;
            let trustee = config
                .trustee
                .clone()
                .map(|a| deps.api.addr_validate(&a))
                .transpose()?;

            if let Some(pauser) = config.is_paused {
                if let Some(trustee) = trustee {
                    // If we have trustee, and its the pauser, and the sender is the account, we change the pauser to the account
                    // else it means that the pauser is the account, so we error because rebalancer already paused.
                    if pauser == trustee && sender == account {
                        config.is_paused = Some(account.clone());
                    } else {
                        return Err(ContractError::AccountAlreadyPaused);
                    }
                } else {
                    // If we reach here, it means we don't have a trustee, but the rebalancer is paused
                    // which can only mean that the pauser is the account, so we error because rebalancer already paused.
                    return Err(ContractError::AccountAlreadyPaused);
                }
            } else {
                // If we reached here it means the rebalancer is not paused so we check if the sender is valid
                // sender can either be the trustee or the account.
                if sender == account {
                    // If we don't have a trustee, and the sender is the account, then we set him as the pauser
                    config.is_paused = Some(account.clone());
                } else if let Some(trustee) = trustee {
                    // If we have a trustee, and its the sender, then we set him as the pauser
                    if trustee == sender {
                        config.is_paused = Some(trustee);
                    } else {
                        // The sender is not the trustee, so we error
                        return Err(ContractError::NotAuthorizedToPause);
                    }
                } else {
                    // If we reach here, it means we don't have a trustee, and the sender is not the account
                    // so we error because only the account can pause the rebalancer.
                    return Err(ContractError::NotAuthorizedToPause);
                }
            }

            CONFIGS.save(deps.storage, account, &config)?;

            Ok(Response::default())
        }
        RebalancerExecuteMsg::Resume { resume_for, sender } => {
            verify_services_manager(deps.as_ref(), &info)?;
            let account = deps.api.addr_validate(&resume_for)?;
            let sender = deps.api.addr_validate(&sender)?;

            let mut config = CONFIGS.load(deps.storage, account.clone())?;
            let trustee = config
                .trustee
                .clone()
                .map(|a| deps.api.addr_validate(&a))
                .transpose()?;

            // If config is paused
            if let Some(resumer) = config.is_paused {
                // If the sender is the account, we resume
                if sender == account {
                    config.is_paused = None;
                } else if let Some(trustee) = trustee {
                    // If we have a trustee, and its the sender, we resume
                    if sender == trustee && resumer == trustee {
                        config.is_paused = None;
                    } else {
                        // We error because only the account or the trustee can resume
                        return Err(ContractError::NotAuthorizedToResume);
                    }
                } else {
                    // If we don't have a trustee and sender is not account, we error
                    return Err(ContractError::NotAuthorizedToResume);
                }
            } else {
                // config is not paused, so error out
                return Err(ContractError::NotPaused);
            }

            CONFIGS.save(deps.storage, account, &config)?;

            Ok(Response::default())
        }
        RebalancerExecuteMsg::SystemRebalance { limit } => {
            execute_system_rebalance(deps, &env, limit)
        }
    }
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::GetConfig { addr } => {
            to_binary(&CONFIGS.load(deps.storage, deps.api.addr_validate(&addr)?)?)
        }
        QueryMsg::GetSystemStatus {} => to_binary(&SYSTEM_REBALANCE_STATUS.load(deps.storage)?),
    }
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn reply(_deps: DepsMut, _env: Env, msg: Reply) -> Result<Response, ContractError> {
    // Tick messages are dispatched with reply ID 0 and reply on
    // error. If an error occurs, we ignore it but stop the parent
    // message from failing, so the state change which moved the tick
    // receiver to the end of the message queue gets committed. This
    // prevents an erroring tick receiver from locking the clock.
    match msg.id {
        REPLY_DEFAULT_REBALANCE => Ok(Response::default()
            .add_attribute("method", "reply_on_error")
            .add_attribute("error", msg.result.unwrap_err())),
        _ => Err(ContractError::UnexpectedReplyId(msg.id)),
    }
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn migrate(_deps: DepsMut, _env: Env, _msg: MigrateMsg) -> Result<Response, ContractError> {
    unimplemented!()
}
