use cosmwasm_std::{DecimalRangeExceeded, StdError};
use cw_utils::PaymentError;
use thiserror::Error;

#[derive(Error, Debug, PartialEq)]
pub enum ValenceError {
    #[error("{0}")]
    Std(#[from] StdError),

    #[error(transparent)]
    DecimalRangeExceeded(#[from] DecimalRangeExceeded),

    #[error(transparent)]
    PaymentError(#[from] PaymentError),

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

    #[error("Service: {service}, Couldn't parse binary into: {ty}")]
    DataParseError { service: String, ty: String },

    #[error("PID values cannot be more then 1")]
    PIDErrorOver,

    #[error("PID values cannot be negetive")]
    PIDErrorNegetive,

    #[error("max_limit_bps must be between 1-10000")]
    InvalidMaxLimitRange,

    #[error("No new admin change started")]
    NoAdminChangeData,

    #[error("Not the new admin")]
    NotNewAdmin,

    #[error("Change admin is expired")]
    AdminChangeExpired,

    #[error("Must pay the registration fee of: {0}{1}")]
    MustPayRegistrationFee(String, String),
}
