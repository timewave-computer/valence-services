use cosmwasm_schema::cw_serde;
use cosmwasm_std::{
    to_binary, CosmosMsg, Deps, DepsMut, MessageInfo, Response, Timestamp, WasmMsg,
};

use crate::{
    error::ValenceError,
    msgs::{core_execute::ServicesManagerExecuteMsg, core_query::ServicesManagerQueryMsg},
    states::{ADMIN, SERVICES_MANAGER},
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
        msg: to_binary(&msg)?,
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
pub fn start_of_day(time: Timestamp) -> Timestamp {
    let leftover = time.seconds() % (60 * 60 * 24); // How much leftover from the start of the day (mid night UTC)
    time.minus_seconds(leftover)
}

#[cfg(test)]
mod test {
    use cosmwasm_std::testing::mock_env;

    use super::start_of_day;

    #[test]
    fn test_start_of_day() {
        let time1 = mock_env().block.time;
        let time2 = mock_env().block.time.plus_seconds(5);

        let start_of_1 = start_of_day(time1);
        let start_of_2 = start_of_day(time2);

        assert_eq!(start_of_1, start_of_2)
    }
}
