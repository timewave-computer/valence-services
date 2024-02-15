use std::collections::HashSet;

use cosmwasm_schema::{cw_serde, QueryResponses};
use cosmwasm_std::{Addr, Timestamp};
use valence_package::services::rebalancer::{BaseDenom, RebalancerConfig, SystemRebalanceStatus};

#[cw_serde]
pub struct InstantiateMsg {
    pub denom_whitelist: Vec<String>,
    pub base_denom_whitelist: Vec<BaseDenom>,
    pub services_manager_addr: String,
    pub cycle_start: Timestamp,
    pub auctions_manager_addr: String,
    pub cycle_period: Option<u64>,
}

#[cw_serde]
#[derive(QueryResponses)]
pub enum QueryMsg {
    // /// Returns true if `address` is in the queue, and false
    // /// otherwise.
    #[returns(RebalancerConfig)]
    GetConfig { addr: String },
    #[returns(SystemRebalanceStatus)]
    GetSystemStatus {},
    #[returns(WhitelistsResponse)]
    GetWhiteLists,
    #[returns(ManagersAddrsResponse)]
    GetManagersAddrs,
    #[returns(Addr)]
    GetAdmin,
}

#[cw_serde]
pub enum MigrateMsg {}

#[cw_serde]
pub struct WhitelistsResponse {
    pub denom_whitelist: HashSet<String>,
    pub base_denom_whitelist: HashSet<BaseDenom>,
}

#[cw_serde]
pub struct ManagersAddrsResponse {
    pub services: Addr,
    pub auctions: Addr,
}
