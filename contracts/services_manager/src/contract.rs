#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{
    to_binary, Addr, Binary, Deps, DepsMut, Env, MessageInfo, Reply, Response, StdError, StdResult,
};
use cw2::set_contract_version;
use valence_package::msgs::core_execute::ServicesManagerExecuteMsg;
use valence_package::msgs::core_query::ServicesManagerQueryMsg;
use valence_package::states::ADMIN;

use crate::error::ContractError;
use crate::helpers::{get_service_addr, save_service};
use crate::msg::{InstantiateMsg, MigrateMsg};
use crate::state::{ADDR_TO_SERVICES, SERVICES_TO_ADDR};

const CONTRACT_NAME: &str = "crates.io:services-manager";
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    _msg: InstantiateMsg,
) -> Result<Response, ContractError> {
    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;

    ADMIN.save(deps.storage, &info.sender)?;

    Ok(Response::default().add_attribute("method", "instantiate"))
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: ServicesManagerExecuteMsg,
) -> Result<Response, ContractError> {
    match msg {
        ServicesManagerExecuteMsg::Admin(admin_msg) => {
            admin::handle_msg(deps, env, info, admin_msg)
        }
        ServicesManagerExecuteMsg::RegisterToService { service_name, data } => {
            let service_addr = get_service_addr(deps.as_ref(), service_name.to_string())?;

            let msg =
                service_name.get_register_msg(info.sender.as_ref(), service_addr.as_ref(), data)?;

            Ok(Response::default().add_message(msg))
        }
        ServicesManagerExecuteMsg::DeregisterFromService { service_name } => {
            let service_addr = get_service_addr(deps.as_ref(), service_name.to_string())?;

            let msg =
                service_name.get_deregister_msg(info.sender.as_ref(), service_addr.as_ref())?;

            Ok(Response::default().add_message(msg))
        }
        ServicesManagerExecuteMsg::UpdateService { service_name, data } => {
            let service_addr = get_service_addr(deps.as_ref(), service_name.to_string())?;

            let msg =
                service_name.get_update_msg(info.sender.as_ref(), service_addr.as_ref(), data)?;

            Ok(Response::default().add_message(msg))
        }
        ServicesManagerExecuteMsg::PauseService {
            service_name,
            pause_for,
        } => {
            let service_addr = get_service_addr(deps.as_ref(), service_name.to_string())?;
            let msg = service_name.get_pause_msg(
                pause_for,
                info.sender.as_ref(),
                service_addr.as_ref(),
            )?;

            Ok(Response::default().add_message(msg))
        }
        ServicesManagerExecuteMsg::ResumeService {
            service_name,
            resume_for,
        } => {
            let service_addr = get_service_addr(deps.as_ref(), service_name.to_string())?;
            let msg = service_name.get_resume_msg(
                resume_for,
                info.sender.as_ref(),
                service_addr.as_ref(),
            )?;

            Ok(Response::default().add_message(msg))
        }
    }
}

mod admin {
    use valence_package::{helpers::verify_admin, msgs::core_execute::ServicesManagerAdminMsg};

    use crate::helpers::remove_service;

    use super::*;

    pub fn handle_msg(
        deps: DepsMut,
        _env: Env,
        info: MessageInfo,
        msg: ServicesManagerAdminMsg,
    ) -> Result<Response, ContractError> {
        // Verify that the sender is the admin
        verify_admin(deps.as_ref(), &info)?;

        match msg {
            ServicesManagerAdminMsg::AddService { name, addr } => {
                let addr = deps.api.addr_validate(&addr)?;

                if SERVICES_TO_ADDR.has(deps.storage, name.to_string()) {
                    return Err(ContractError::ServiceAlreadyExists(name.to_string()));
                } else if ADDR_TO_SERVICES.has(deps.storage, addr.clone()) {
                    return Err(ContractError::ServiceAddressAlreadyExists(addr.to_string()));
                } else {
                    save_service(deps, name.to_string(), addr)?;
                }

                Ok(Response::default().add_attribute("method", "add_service"))
            }
            ServicesManagerAdminMsg::UpdateService { name, addr } => {
                let addr = deps.api.addr_validate(&addr)?;

                if ADDR_TO_SERVICES.has(deps.storage, addr.clone()) {
                    return Err(ContractError::ServiceAddressAlreadyExists(addr.to_string()));
                } else if !SERVICES_TO_ADDR.has(deps.storage, name.to_string()) {
                    return Err(ContractError::ServiceDoesntExistYet(name.to_string()));
                }

                save_service(deps, name.to_string(), addr)?;

                Ok(Response::default().add_attribute("method", "update_service"))
            }
            ServicesManagerAdminMsg::RemoveService { name } => {
                let addr = get_service_addr(deps.as_ref(), name.to_string())?;
                remove_service(deps, name.to_string(), addr)?;

                Ok(Response::default().add_attribute("method", "remove_service"))
            }
        }
    }
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, _env: Env, msg: ServicesManagerQueryMsg) -> StdResult<Binary> {
    match msg {
        ServicesManagerQueryMsg::IsService { addr } => {
            let is_service = ADDR_TO_SERVICES.has(deps.storage, deps.api.addr_validate(&addr)?);
            to_binary(&is_service)
        }
        ServicesManagerQueryMsg::GetServiceAddr { service } => {
            let addr = get_service_addr(deps, service.to_string())
                .map_err(|e| StdError::GenericErr { msg: e.to_string() })?;
            to_binary(&addr)
        }
        ServicesManagerQueryMsg::GetAdmin => to_binary(&ADMIN.load(deps.storage)?),
        ServicesManagerQueryMsg::GetAllServices => {
            let services = SERVICES_TO_ADDR
                .range(deps.storage, None, None, cosmwasm_std::Order::Ascending)
                .map(|item| item.map(|(name, addr)| (name, addr)))
                .collect::<StdResult<Vec<(String, Addr)>>>()?;

            to_binary(&services)
        }
    }
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn reply(_deps: DepsMut, _env: Env, _msg: Reply) -> Result<Response, ContractError> {
    unimplemented!()
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn migrate(_deps: DepsMut, _env: Env, _msg: MigrateMsg) -> Result<Response, ContractError> {
    unimplemented!()
}
