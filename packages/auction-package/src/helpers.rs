use cosmwasm_schema::cw_serde;
use cosmwasm_std::{Decimal, Deps, MessageInfo, Timestamp};

use crate::{error::AuctionError, states::ADMIN, Pair, PriceFreshnessStrategy};

pub fn verify_admin(deps: Deps, info: &MessageInfo) -> Result<(), AuctionError> {
    if ADMIN.load(deps.storage)? != info.sender {
        return Err(AuctionError::NotAdmin);
    }
    Ok(())
}

#[cw_serde]
pub struct GetPriceResponse {
    pub price: Decimal,
    pub time: Timestamp,
}

#[cw_serde]
pub struct ChainHaltConfig {
    /// Time in seconds of how much of a halt we accept
    pub cap: u128,
    /// seconds each block is generated
    pub block_avg: Decimal,
}

#[cw_serde]
pub struct AuctionConfig {
    pub is_paused: bool,
    pub pair: Pair,
    pub chain_halt_config: ChainHaltConfig,
    pub price_freshness_strategy: PriceFreshnessStrategy,
}
