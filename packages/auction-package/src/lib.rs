use cosmwasm_schema::cw_serde;
use cosmwasm_std::{Decimal, Timestamp};

pub mod error;
pub mod helpers;
pub mod msgs;
pub mod pair;
pub mod states;

pub use pair::Pair;

pub const CLOSEST_TO_ONE_POSSIBLE: u64 = 9999;

#[cw_serde]
pub struct AuctionStrategy {
    pub start_price_perc: u64, // BPS
    pub end_price_perc: u64,   // BPS
}

/// Gives us the strategy we should use for when the data is not fresh.
/// "multiplier" list is sorted in descending order, so after we check the list,
/// if the data is fresh, the multiplier is 1.
///
/// Ex: smallest day in the list is "0.5" (12 hours), so the multiplier will be 1 if
/// the data is updated in the last 12 hours.
#[cw_serde]
pub struct PriceFreshnessStrategy {
    /// Amount of days price considered no longer fresh
    pub limit: Decimal,
    /// Multiplier per day of unfresh data (older than day, multipler)
    /// for when data is older than 2 days, we add: ("2", "1.5")
    pub multipliers: Vec<(Decimal, Decimal)>,
}

#[cw_serde]
pub struct Price {
    pub price: Decimal,
    pub time: Timestamp,
}
