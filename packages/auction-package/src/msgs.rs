use cosmwasm_schema::{cw_serde, QueryResponses};
use cosmwasm_std::{Addr, Uint128};

use crate::{
    helpers::{AuctionConfig, GetPriceResponse},
    Pair,
};

#[cw_serde]
#[derive(QueryResponses)]
pub enum AuctionsManagerQueryMsg {
    /// Get the price of a specific pair
    #[returns(GetPriceResponse)]
    GetPrice { pair: Pair },

    /// Get the config of a specific auction
    #[returns(AuctionConfig)]
    GetConfig { pair: Pair },

    /// Get the pair address
    #[returns(Addr)]
    GetPairAddr { pair: Pair },

    /// Get the oracle address
    #[returns(Addr)]
    GetOracleAddr,

    #[returns(Uint128)]
    GetMinLimit { denom: String },

    #[returns(Addr)]
    GetAdmin,
}
