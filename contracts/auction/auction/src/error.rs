use auction_package::error::AuctionError;
use cosmwasm_std::{CheckedFromRatioError, DecimalRangeExceeded, OverflowError, StdError, Uint128};
use cw_utils::PaymentError;
use thiserror::Error;

#[derive(Error, Debug, PartialEq)]
pub enum ContractError {
    #[error("{0}")]
    Std(#[from] StdError),

    #[error(transparent)]
    PaymentError(#[from] PaymentError),

    #[error(transparent)]
    AuctionError(#[from] AuctionError),

    #[error(transparent)]
    OverflowError(#[from] OverflowError),

    #[error(transparent)]
    DecimalRangeExceeded(#[from] DecimalRangeExceeded),

    #[error(transparent)]
    CheckedFromRatioError(#[from] CheckedFromRatioError),

    #[error("Sender is not admin")]
    NotAdmin,

    #[error("Auction amount is too low, minimum: {0}")]
    AuctionAmountTooLow(Uint128),

    #[error("Auction is paused")]
    AuctionIsPaused,

    #[error("Current auction is finished")]
    AuctionFinished,

    #[error("Auction is not closed yet")]
    AuctionNotClosed,

    #[error("Auction is already closed")]
    AuctionClosed,

    #[error("Auction is finished and we resolved everything")]
    AuctionFinishedAndResolved,

    #[error("This address is not authorized to bid")]
    UnauthorizedToBid,

    #[error("A bid must be called as part of a transaction")]
    BidMustBeCalledFromTx,

    #[error("Auction not started yet, starts at block: {0}")]
    AuctionNotStarted(u64),

    #[error("Bids of the auction are not resolved yet, please resolve all bids first")]
    BidsNotResolved,

    #[error("Auction is still going")]
    AuctionStillGoing,

    #[error("Cannot start auction because no funds available")]
    NoFundsForAuction,

    #[error("No funds to withdraw from auction")]
    NoFundsToWithdraw,

    #[error("Chain has halted, cannot bid")]
    ChainHalted,

    #[error("Price is older than 4 days")]
    PriceTooOld,

    #[error("Couldn't get a minimum amount for auction")]
    NoTokenMinAmount,

    #[error("End block is smaller or equal to the start block")]
    InvalidAuctionEndBlock,
}

impl From<ContractError> for StdError {
    fn from(value: ContractError) -> Self {
        Self::generic_err(value.to_string())
    }
}
