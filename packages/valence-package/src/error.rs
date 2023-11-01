use cosmwasm_std::{DecimalRangeExceeded, StdError};
use thiserror::Error;

#[derive(Error, Debug, PartialEq)]
pub enum ValenceError {
    #[error("{0}")]
    Std(#[from] StdError),

    #[error(transparent)]
    DecimalRangeExceeded(#[from] DecimalRangeExceeded),

    #[error("Sender is not a service!")]
    UnauthorizedService,

    #[error("Only admin can perform this action")]
    NotAdmin,

    #[error("Sender is not services manager")]
    NotServicesManager,

    #[error("This services doesn't exists: {0}")]
    InvalidService(String),

    #[error("This services expects data on register: {0}")]
    MissingRegisterData(String),

    #[error("Couldn't parse binary into: {0}")]
    RegisterDataParseError(String),

    #[error("PID values cannot be more then 1")]
    PIDErrorOver,

    #[error("max_limit_bps must be between 1-10000")]
    InvalidMaxLimitRange,
}
