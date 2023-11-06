use auction::msg::NewAuctionParams;
use auction_package::{helpers::ChainHaltConfig, AuctionStrategy, Pair, PriceFreshnessStrategy};
use cosmwasm_schema::cw_serde;
use cosmwasm_std::Uint128;

#[cw_serde]
pub struct InstantiateMsg {
    pub auction_code_id: u64,
    pub min_auction_amount: Vec<(String, Uint128)>,
}

#[cw_serde]
pub enum ExecuteMsg {
    AuctionFunds { pair: Pair },
    WithdrawFunds { pair: Pair },
    FinishAuction { pair: Pair, limit: u64 },
    Admin(AdminMsgs),
}

#[cw_serde]
pub enum MigrateMsg {
    NoStateChange,
}

#[cw_serde]
pub enum AdminMsgs {
    NewAuction {
        msg: auction::msg::InstantiateMsg,
        min_amount: Option<Uint128>,
    },
    OpenAuction {
        pair: Pair,
        params: NewAuctionParams,
    },
    PauseAuction {
        pair: Pair,
    },
    ResumeAuction {
        pair: Pair,
    },
    UpdateAuctionId {
        code_id: u64,
    },
    UpdateOracle {
        oracle_addr: String,
    },
    UpdateStrategy {
        pair: Pair,
        strategy: AuctionStrategy,
    },
    UpdateChainHaltConfig {
        pair: Pair,
        halt_config: ChainHaltConfig,
    },
    UpdatePriceFreshnessStrategy {
        pair: Pair,
        strategy: PriceFreshnessStrategy,
    },
    UpdateActiveAuction {
        pair: Pair,
        params: auction::state::ActiveAuction,
    },
    MigrateAuction {
        pair: Pair,
        code_id: u64,
        msg: auction::msg::MigrateMsg,
    },
}
