use auction_package::{error::AuctionError, Pair};
use cosmwasm_std::{CheckedFromRatioError, DecimalRangeExceeded, OverflowError, StdError};
use thiserror::Error;

#[derive(Error, Debug, PartialEq)]
pub enum ContractError {
    #[error("{0}")]
    Std(#[from] StdError),

    #[error(transparent)]
    AuctionError(#[from] AuctionError),

    #[error(transparent)]
    DecimalRangeExceeded(#[from] DecimalRangeExceeded),

    #[error(transparent)]
    OverflowError(#[from] OverflowError),

    #[error(transparent)]
    CheckedFromRatioError(#[from] CheckedFromRatioError),

    #[error("Sender is not admin")]
    NotAdmin,

    #[error("Couldn't find auction for this pair")]
    PairAuctionNotFound,

    #[error("Less then 3 auctions happened so far, not enough data for a price")]
    NotEnoughTwaps,

    #[error("No auction happened in the last 3 days")]
    NoAuctionInLast3Days,

    #[error("Set price cannot be zero")]
    PriceIsZero,

    #[error("Can't manually update price, terms are not met for manual update")]
    NoTermsForManualUpdate,

    #[error("Path for this pair already exists")]
    PricePathAlreadyExists,

    #[error("Path for this pair doesn't exists yet")]
    PricePathNotFound,

    #[error("Path must not be empty")]
    PricePathIsEmpty,

    #[error("Path doesn't match pair, denom1 in first step must be the same as pair.0, and last step denom2 must match pair.1")]
    PricePathIsWrong,

    #[error("No astroport path found for pair: {0}")]
    NoAstroPath(Pair),
}
