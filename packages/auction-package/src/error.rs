use cosmwasm_std::{DecimalRangeExceeded, StdError};
use thiserror::Error;

#[derive(Error, Debug, PartialEq)]
pub enum AuctionError {
    #[error("{0}")]
    Std(#[from] StdError),

    #[error(transparent)]
    DecimalRangeExceeded(#[from] DecimalRangeExceeded),

    #[error("Pair is invalid")]
    InvalidPair,

    #[error("Sender is not admin")]
    NotAdmin,
}
