use cosmwasm_std::StdError;
use thiserror::Error;
use valence_package::error::ValenceError;

#[derive(Error, Debug, PartialEq)]
pub enum ContractError {
    #[error("{0}")]
    Std(#[from] StdError),

    #[error(transparent)]
    ValenceError(#[from] ValenceError),

    #[error("Message expected to send funds but none were sent!")]
    ExpectedFunds,

    #[error("Message is not supported by this contract to send funds! message: {0}")]
    NotSupportedMessageWithFunds(String),

    #[error("Message is not supported by this contract! message: {0}")]
    NotSupportedMessage(String),

    #[error("Reply id is not recognized: {0}")]
    UnexpectedReplyId(u64),
}
