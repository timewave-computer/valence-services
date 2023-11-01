use auction_package::error::AuctionError;
use cosmwasm_std::StdError;
use cw_utils::{ParseReplyError, PaymentError};
use thiserror::Error;

#[derive(Error, Debug, PartialEq)]
pub enum ContractError {
    #[error("{0}")]
    Std(#[from] StdError),

    #[error(transparent)]
    ParseReplyError(#[from] ParseReplyError),

    #[error(transparent)]
    PaymentError(#[from] PaymentError),

    #[error(transparent)]
    AuctionError(#[from] AuctionError),

    #[error("Uknown reply id: {0}")]
    UnknownReplyId(u64),

    #[error("Oracle address is missing")]
    OracleAddrMissing,

    #[error("Minimum amount for the token: {0} is missing")]
    MustSetMinAuctionAmount(String),
}

impl From<ContractError> for StdError {
    fn from(value: ContractError) -> Self {
        Self::generic_err(value.to_string())
    }
}
