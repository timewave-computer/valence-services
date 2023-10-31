use cosmwasm_schema::{cw_serde, QueryResponses};
use cosmwasm_std::Timestamp;
use valence_package::services::rebalancer::RebalancerConfig;

use crate::state::SystemRebalanceStatus;

#[cw_serde]
pub struct InstantiateMsg {
    pub denom_whitelist: Vec<String>,
    pub base_denom_whitelist: Vec<String>,
    pub services_manager_addr: String,
    pub cycle_start: Timestamp,
    pub auctions_manager_addr: String,
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
}

#[cw_serde]
pub enum MigrateMsg {}
