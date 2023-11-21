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

    #[error("No new admin change started")]
    NoAdminChangeData,

    #[error("Not the new admin")]
    NotNewAdmin,

    #[error("Change admin is expired")]
    AdminChangeExpired,

    #[error("Auction strategy start price must be greater than 0")]
    InvalidAuctionStrategyStartPrice,

    #[error("Auction strategy end price must be greater than 0 and lower than 10000")]
    InvalidAuctionStrategyEndPrice,
}
