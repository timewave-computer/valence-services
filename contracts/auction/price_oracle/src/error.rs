use auction_package::error::AuctionError;
use cosmwasm_std::StdError;
use thiserror::Error;

#[derive(Error, Debug, PartialEq)]
pub enum ContractError {
    #[error("{0}")]
    Std(#[from] StdError),

    #[error(transparent)]
    AuctionError(#[from] AuctionError),

    #[error("Sender is not admin")]
    NotAdmin,

    #[error("Price expired and no longer fresh")]
    PriceExpired,

    #[error("Couldn't find auction for this pair")]
    PairAuctionNotFound,

    #[error("Less then 3 auctions happened so far, not enough data for a price")]
    NotEnoughTwaps,

    #[error("No auction happened in the last 3 days")]
    NoAuctionInLast3Days,

    #[error("Set price cannot be zero")]
    PriceIsZero,
}
