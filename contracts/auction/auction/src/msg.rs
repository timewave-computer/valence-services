use auction_package::{
    helpers::{AuctionConfig, ChainHaltConfig, GetPriceResponse},
    AuctionStrategy, Pair, PriceFreshnessStrategy,
};
use cosmwasm_schema::{cw_serde, QueryResponses};
use cosmwasm_std::{Addr, Uint128};

use crate::state::ActiveAuction;

#[cw_serde]
pub struct InstantiateMsg {
    pub pair: Pair,
    pub auction_strategy: AuctionStrategy,
    pub chain_halt_config: ChainHaltConfig,
    pub price_freshness_strategy: PriceFreshnessStrategy,
}

#[cw_serde]
pub enum ExecuteMsg {
    /// Send funds to be auctioned on the next auction, can only be called by the admin/auctions manager
    AuctionFundsManager { sender: Addr },
    /// Send funds to be auctioned on the next auction
    AuctionFunds,
    /// Withdraw funds from future auction, can only be called by the admin/auctions manager
    WithdrawFundsManager { sender: Addr },
    /// Withdraw funds from future auction
    WithdrawFunds,
    /// Bid on the current auction
    Bid,
    /// Finish the current auction and send funds to the funds provider
    /// Send pair.1 according to the weight of the funds provider from the total amount
    /// If we have unsold pair.0, send to funds provider accoring to provided weight
    FinishAuction { limit: u64 },
    /// Message to clean finished auction unneeded storage
    CleanAfterAuction,
    /// Admin messages that can only be called by the auctions manager
    Admin(AdminMsgs),
}

#[cw_serde]
pub struct NewAuctionParams {
    /// Optional start block, if not provided, it will start from the current block
    pub start_block: Option<u64>,
    /// When auction should end
    pub end_block: u64,
}

/// Admin messages that can only be called by the auctions manager
#[cw_serde]
pub enum AdminMsgs {
    /// Pause auction
    PauseAuction,
    /// Resume paused auction
    ResumeAuction,
    /// Update the auction strategy
    UpdateStrategy { strategy: AuctionStrategy },
    /// Start a new auction
    StartAuction(NewAuctionParams),
    /// Update the chain halt config
    UpdateChainHaltConfig(ChainHaltConfig),
    /// Update the price freshness strategy
    UpdatePriceFreshnessStrategy(PriceFreshnessStrategy),
}

#[cw_serde]
#[derive(QueryResponses)]
pub enum QueryMsg {
    /// Get the config which includes the pair and the min amount
    #[returns(AuctionConfig)]
    GetConfig,

    /// Get amount of funds provided by the given address on the current and next auction
    #[returns(GetFundsAmountResponse)]
    GetFundsAmount { addr: String },

    /// Get the current auction details
    #[returns(ActiveAuction)]
    GetAuction,

    /// Get the price of the auction on the current block
    #[returns(GetPriceResponse)]
    GetPrice,

    /// Get the strategy of the auction
    #[returns(AuctionStrategy)]
    GetStrategy,
}

#[cw_serde]
pub enum MigrateMsg {}

#[cw_serde]
pub struct GetFundsAmountResponse {
    pub curr: Uint128,
    pub next: Uint128,
}
