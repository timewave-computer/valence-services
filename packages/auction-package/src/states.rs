use std::collections::VecDeque;

use cosmwasm_schema::cw_serde;
use cosmwasm_std::{Addr, Uint128};
use cw_storage_plus::{Item, Map};
use cw_utils::Expiration;

use crate::{helpers::ChainHaltConfig, Pair, Price};

/// The admin of the contract
pub const ADMIN: Item<Addr> = Item::new("admin");
/// The oracle address saved on the auction manager
pub const ORACLE_ADDR: Item<Addr> = Item::new("oracle_addr");
/// Prices storage of the oracle
pub const PRICES: Map<Pair, Price> = Map::new("prices");
/// Map from Pair to auction contract address
pub const PAIRS: Map<Pair, Addr> = Map::new("pairs");

/// TWAP prices of the auction
pub const TWAP_PRICES: Item<VecDeque<Price>> = Item::new("twap_prices");
/// Chain halt config
pub const CHAIN_HALT_CONFIG: Item<ChainHaltConfig> = Item::new("ch_config");
/// The min amount allowed to send to auction per token
pub const MIN_AUCTION_AMOUNT: Map<String, Uint128> = Map::new("min_auction_amount");

pub const ADMIN_CHANGE: Item<AdminChange> = Item::new("admin_change");

#[cw_serde]
pub struct AdminChange {
    pub addr: Addr,
    pub expiration: Expiration,
}
