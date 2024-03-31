use auction_package::{helpers::GetPriceResponse, Pair};
use cosmwasm_schema::{cw_serde, QueryResponses};
use cosmwasm_std::{Addr, Decimal};
use cw_utils::Expiration;

use crate::state::{Config, PriceStep};

#[cw_serde]
pub struct InstantiateMsg {
    pub auctions_manager_addr: String,
    pub seconds_allow_manual_change: u64,
    pub seconds_auction_prices_fresh: u64,
}

#[cw_serde]
pub enum ExecuteMsg {
    ManualPriceUpdate {
        pair: Pair,
        price: Decimal,
    },
    UpdatePrice {
        pair: Pair,
    },
    AddAstroPath {
        pair: Pair,
        path: Vec<PriceStep>,
    },
    UpdateAstroPath {
        pair: Pair,
        path: Vec<PriceStep>,
    },
    StartAdminChange {
        addr: String,
        expiration: Expiration,
    },
    CancelAdminChange {},
    ApproveAdminChange {},
}

#[cw_serde]
#[derive(QueryResponses)]
pub enum QueryMsg {
    /// Get the minimum amount users can auction
    #[returns(GetPriceResponse)]
    GetPrice { pair: Pair },
    #[returns(Config)]
    GetConfig,
    #[returns(Addr)]
    GetAdmin,
}

#[cw_serde]
pub enum MigrateMsg {
    NoStateChange {},
}
