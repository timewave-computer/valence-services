use auction_package::Pair;
use cosmwasm_schema::cw_serde;
use cosmwasm_std::{Addr, Decimal, Timestamp};
use cw_storage_plus::{Item, Map};
use valence_package::services::rebalancer::RebalancerConfig;

/// All available denom to target (denom whitelist)
pub(crate) const DENOM_WHITELIST: Item<Vec<String>> = Item::new("token_whitelist");
/// Base denom whitelist
pub(crate) const BASE_DENOM_WHITELIST: Item<Vec<String>> = Item::new("base_token_whitelist");
/// Storage to keep all configs of  all registered accounts
pub(crate) const CONFIGS: Map<Addr, RebalancerConfig> = Map::new("configs");
/// Storage to keep the current status of the system rebalance
pub(crate) const SYSTEM_REBALANCE_STATUS: Item<SystemRebalanceStatus> =
    Item::new("system_rebalance_status");

pub(crate) const AUCTIONS_MANAGER_ADDR: Item<Addr> = Item::new("auctions_manager_addr");

#[cw_serde]
pub enum SystemRebalanceStatus {
    NotStarted {
        cycle_start: Timestamp,
    },
    Processing {
        cycle_started: Timestamp,
        start_from: Addr,
        prices: Vec<(Pair, Decimal)>,
    },
    Finished {
        next_cycle: Timestamp,
    },
}
