use std::collections::HashSet;

use cosmwasm_std::Addr;
use cw_storage_plus::{Item, Map};
use valence_package::services::rebalancer::{
    BaseDenom, PauseData, RebalancerConfig, SystemRebalanceStatus,
};

/// All available denom to target (denom whitelist)
pub(crate) const DENOM_WHITELIST: Item<HashSet<String>> = Item::new("token_whitelist");
/// Base denom whitelist
pub(crate) const BASE_DENOM_WHITELIST: Item<HashSet<BaseDenom>> = Item::new("base_token_whitelist");
/// Storage to keep all configs of  all registered accounts
pub const CONFIGS: Map<Addr, RebalancerConfig> = Map::new("configs");
/// Storage to keep the current status of the system rebalance
pub(crate) const SYSTEM_REBALANCE_STATUS: Item<SystemRebalanceStatus> =
    Item::new("system_rebalance_status");

pub(crate) const AUCTIONS_MANAGER_ADDR: Item<Addr> = Item::new("auctions_manager_addr");

pub(crate) const CYCLE_PERIOD: Item<u64> = Item::new("cycle_period");

pub const PAUSED_CONFIGS: Map<Addr, PauseData> = Map::new("paused_configs");
