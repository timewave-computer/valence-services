use cosmwasm_schema::cw_serde;
use cosmwasm_std::{
    to_json_binary, CosmosMsg, Deps, DepsMut, Env, MessageInfo, Response, Timestamp, WasmMsg,
};
use cw_utils::Expiration;

use crate::{
    error::ValenceError,
    msgs::{core_execute::ServicesManagerExecuteMsg, core_query::ServicesManagerQueryMsg},
    states::{AdminChange, ADMIN, ADMIN_CHANGE, SERVICES_MANAGER},
};

/// An optional helper for Option, for when we need to update an optional field in storage.
/// Ex:
/// We want to update an optional field in storage: `sample: Option<String>`
/// but we also want to have it optional on the update message: `sample: Option<OptionalField<String>>`
///
/// This allows us to have 3 options:
/// 1. None: Do nothing, keep storage as is.
/// 2. Some(OptionalField::Clear): Clear the field in storage and set it to None.
/// 3. Some(OptionalField::Set(value)): Set the field in storage to Some(value)
#[cw_serde]
pub enum OptionalField<T> {
    Set(T),
    Clear,
}

/// Forward the message to the services manager contract.
pub fn forward_to_services_manager(
    manager_addr: String,
    msg: ServicesManagerExecuteMsg,
) -> Result<Response, ValenceError> {
    let msg = CosmosMsg::Wasm(WasmMsg::Execute {
        contract_addr: manager_addr,
        msg: to_json_binary(&msg)?,
        funds: vec![],
    });
    Ok(Response::default().add_message(msg))
}

/// Verify the sender address is a service
pub fn sender_is_a_service(
    deps: DepsMut,
    info: &MessageInfo,
    manager_addr: String,
) -> Result<(), ValenceError> {
    if deps.querier.query_wasm_smart::<bool>(
        manager_addr,
        &ServicesManagerQueryMsg::IsService {
            addr: info.sender.to_string(),
        },
    )? {
        Ok(())
    } else {
        Err(ValenceError::UnauthorizedService)
    }
}

/// Verify the sender is the admin of the contract
pub fn verify_admin(deps: Deps, info: &MessageInfo) -> Result<(), ValenceError> {
    if ADMIN.load(deps.storage)? != info.sender {
        return Err(ValenceError::NotAdmin {});
    }
    Ok(())
}

/// Verify the sender is the services manager
pub fn verify_services_manager(deps: Deps, info: &MessageInfo) -> Result<(), ValenceError> {
    if SERVICES_MANAGER.load(deps.storage)? != info.sender {
        return Err(ValenceError::NotServicesManager {});
    }
    Ok(())
}

/// Get the timestomt of the start of the day (00:00 midnight)
pub fn start_of_cycle(time: Timestamp, cycle: u64) -> Timestamp {
    let leftover = time.seconds() % cycle; // How much leftover from the start of the day (mid night UTC)
    time.minus_seconds(leftover)
}

pub fn start_admin_change(
    deps: DepsMut,
    info: &MessageInfo,
    addr: &str,
    expiration: Expiration,
) -> Result<Response, ValenceError> {
    verify_admin(deps.as_ref(), info)?;

    let admin_change = AdminChange {
        addr: deps.api.addr_validate(addr)?,
        expiration,
    };

    ADMIN_CHANGE.save(deps.storage, &admin_change)?;

    Ok(Response::default()
        .add_attribute("new_admin_address", admin_change.addr.to_string())
        .add_attribute("expire", expiration.to_string()))
}

pub fn cancel_admin_change(deps: DepsMut, info: &MessageInfo) -> Result<Response, ValenceError> {
    verify_admin(deps.as_ref(), info)?;

    ADMIN_CHANGE.remove(deps.storage);

    Ok(Response::default())
}

pub fn approve_admin_change(
    deps: DepsMut,
    env: &Env,
    info: &MessageInfo,
) -> Result<Response, ValenceError> {
    let admin_data = ADMIN_CHANGE
        .load(deps.storage)
        .map_err(|_| ValenceError::NoAdminChangeData)?;

    if admin_data.addr != info.sender {
        return Err(ValenceError::NotNewAdmin);
    }

    if admin_data.expiration.is_expired(&env.block) {
        return Err(ValenceError::AdminChangeExpired);
    }

    ADMIN.save(deps.storage, &admin_data.addr)?;
    ADMIN_CHANGE.remove(deps.storage);

    Ok(Response::default())
}

#[cfg(test)]
mod test {
    use cosmwasm_std::testing::mock_env;

    use super::start_of_cycle;

    #[test]
    fn test_start_of_day() {
        let time1 = mock_env().block.time;
        let time2 = mock_env().block.time.plus_seconds(5);

        let start_of_1 = start_of_cycle(time1, 60 * 60 * 24);
        let start_of_2 = start_of_cycle(time2, 60 * 60 * 24);

        assert_eq!(start_of_1, start_of_2)
    }
}
