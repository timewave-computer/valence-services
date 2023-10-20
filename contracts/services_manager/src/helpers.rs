use cosmwasm_std::{Addr, Deps, DepsMut};

use crate::{
    error::ContractError,
    state::{ADDR_TO_SERVICES, SERVICES_TO_ADDR},
};

pub(crate) fn get_service_addr(deps: Deps, service: String) -> Result<Addr, ContractError> {
    SERVICES_TO_ADDR
        .load(deps.storage, service.clone())
        .map_err(|_| ContractError::ServiceDoesntExists(service.to_string()))
}

pub(crate) fn save_service(
    deps: DepsMut,
    service: String,
    addr: Addr,
) -> Result<(), ContractError> {
    SERVICES_TO_ADDR.save(deps.storage, service.clone(), &addr)?;
    ADDR_TO_SERVICES.save(deps.storage, addr, &service)?;
    Ok(())
}

pub(crate) fn remove_service(
    deps: DepsMut,
    service: String,
    addr: Addr,
) -> Result<(), ContractError> {
    SERVICES_TO_ADDR.remove(deps.storage, service);
    ADDR_TO_SERVICES.remove(deps.storage, addr);
    Ok(())
}
