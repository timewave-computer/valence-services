use cosmwasm_std::StdError;
use thiserror::Error;
use valence_package::error::ValenceError;

#[derive(Error, Debug, PartialEq)]
pub enum ContractError {
    #[error("{0}")]
    Std(#[from] StdError),

    #[error(transparent)]
    ValenceError(#[from] ValenceError),

    #[error("This services already exists: {0}")]
    ServiceAlreadyExists(String),

    #[error("This services doesn't exist yet: {0}")]
    ServiceDoesntExistYet(String),

    #[error("This address already exists: {0}")]
    ServiceAddressAlreadyExists(String),

    #[error("This services doesn't exists: {0}")]
    ServiceDoesntExist(String),

    #[error("Contract is not allowed to register to services: {0}")]
    NotWhitelistedContract(u64),
}
