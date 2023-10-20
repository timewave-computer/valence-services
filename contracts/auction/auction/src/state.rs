use auction_package::{helpers::AuctionConfig, AuctionStrategy};
use cosmwasm_schema::cw_serde;
use cosmwasm_std::{Addr, BlockInfo, Decimal, Uint128};
use cw_storage_plus::{Item, Map};

/// The config of any auction
pub const AUCTION_CONFIG: Item<AuctionConfig> = Item::new("auction_config");
/// Inner tracker of auction ids of current auction and next auction
pub const AUCTION_IDS: Item<AuctionIds> = Item::new("auction_ids");
/// track user funds sent for auction for auction id and address
pub const AUCTION_FUNDS: Map<(u64, Addr), Uint128> = Map::new("funds");
/// Sum of the funds sent for auction for auction id
pub const AUCTION_FUNDS_SUM: Map<u64, Uint128> = Map::new("funds_sum");

/// The active auction data
pub(crate) const ACTIVE_AUCTION: Item<ActiveAuction> = Item::new("active_auction");
/// The strategy we use when setting min and max prices for an auction
pub(crate) const AUCTION_STRATEGY: Item<AuctionStrategy> = Item::new("auction_strategy");

#[cw_serde]
pub struct ActiveAuction {
    /// The auction status
    pub status: ActiveAuctionStatus,
    /// The auction starting block height
    pub start_block: u64,
    /// The auction ending block height
    pub end_block: u64,
    /// The price on start_block
    pub start_price: Decimal,
    /// The price on end_block
    pub end_price: Decimal,
    /// The available amount of pair.0
    pub available_amount: Uint128,
    /// The received and resolved amount of pair.1
    pub resolved_amount: Uint128,
    /// The total funds of pair.0 that was sent to sell
    pub total_amount: Uint128,
    /// leftover funds to add to the next auction
    pub leftovers: [Uint128; 2],
    /// The last checked block for chain halts
    pub last_checked_block: BlockInfo,
}

#[cw_serde]
pub enum ActiveAuctionStatus {
    /// The auction started, and last resolved block height is (u64)
    Started,
    /// The auction is finished, if param is 0, we resolved everything,
    /// else it holds the last resovled height
    Finished,
    /// Handle closing auction, addr of the last funds provider we resolved
    /// and the total amounts of the pair we sent already
    /// (provider, total_amount_pair.0, total_amount_pair.1)
    CloseAuction(Option<Addr>, Uint128, Uint128),
    /// The auction is closed
    AuctionClosed,
}

#[cw_serde]
pub struct AuctionIds {
    pub curr: u64,
    pub next: u64,
}
