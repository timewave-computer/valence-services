use cosmwasm_std::{Addr, Deps, DepsMut};
use valence_package::states::ACCOUNT_WHITELISTED_CODE_IDS;

use crate::{
    error::ContractError,
    state::{ADDR_TO_SERVICES, SERVICES_TO_ADDR},
};

pub(crate) fn get_service_addr(deps: Deps, service: String) -> Result<Addr, ContractError> {
    SERVICES_TO_ADDR
        .load(deps.storage, service.clone())
        .map_err(|_| ContractError::ServiceDoesntExist(service.to_string()))
}

/// Save given service name and address to storages
pub(crate) fn save_service(
    deps: DepsMut,
    service: String,
    addr: Addr,
) -> Result<(), ContractError> {
    SERVICES_TO_ADDR.save(deps.storage, service.clone(), &addr)?;
    ADDR_TO_SERVICES.save(deps.storage, addr, &service)?;
    Ok(())
}

/// Remove service from storages
pub(crate) fn remove_service(
    deps: DepsMut,
    service: String,
    addr: Addr,
) -> Result<(), ContractError> {
    SERVICES_TO_ADDR.remove(deps.storage, service);
    ADDR_TO_SERVICES.remove(deps.storage, addr);
    Ok(())
}

pub fn verify_account_code_id(
    deps: Deps,
    account_addr: impl Into<String>,
) -> Result<(), ContractError> {
    let sender_code_id = deps.querier.query_wasm_contract_info(account_addr)?.code_id;
    let whitelist = ACCOUNT_WHITELISTED_CODE_IDS.load(deps.storage)?;

    if !whitelist.contains(&sender_code_id) {
        return Err(ContractError::NotWhitelistedContract(sender_code_id));
    }

    Ok(())
}
