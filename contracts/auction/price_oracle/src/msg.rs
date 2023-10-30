use auction_package::{helpers::GetPriceResponse, Pair};
use cosmwasm_schema::{cw_serde, QueryResponses};
use cosmwasm_std::Decimal;

use crate::state::Config;

#[cw_serde]
pub struct InstantiateMsg {
    pub auctions_manager_addr: String,
}

#[cw_serde]
pub enum ExecuteMsg {
    UpdatePrice { pair: Pair, price: Option<Decimal> },
}

#[cw_serde]
#[derive(QueryResponses)]
pub enum QueryMsg {
    /// Get the minimum amount users can auction
    #[returns(GetPriceResponse)]
    GetPrice { pair: Pair },
    #[returns(Config)]
    GetConfig,
}

#[cw_serde]
pub enum MigrateMsg {}
