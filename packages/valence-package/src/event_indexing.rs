use std::{collections::HashSet, fmt};

use auction_package::{
    helpers::{AuctionConfig, ChainHaltConfig},
    states::MinAmount,
    AuctionStrategy, Pair, PriceFreshnessStrategy,
};
use cosmwasm_std::{
    to_json_binary, Binary, Coin, CosmosMsg, Decimal, Empty, Event, SubMsg, Uint128,
};
use serde::Serialize;

use crate::services::rebalancer::{
    BaseDenom, RebalanceTrade, RebalancerConfig, ServiceFeeConfig, SystemRebalanceStatus,
};

pub type ValenceEventEmpty = ValenceEvent<Empty>;

#[derive(
    cosmwasm_schema::serde::Serialize,
    cosmwasm_schema::serde::Deserialize,
    std::clone::Clone,
    std::fmt::Debug,
    std::cmp::PartialEq,
    cosmwasm_schema::schemars::JsonSchema,
)]
#[serde(deny_unknown_fields, rename_all = "kebab-case")]
#[schemars(crate = "cosmwasm_schema::schemars")]
pub enum ValenceEvent<E>
where
    E: Serialize,
{
    AccountCreation {
        /// The admin address of the account
        admin: String,
        /// where the account was created from (native, 3rd-party, etc)
        referral: String,
    },
    AccountRegisterService {
        service_name: String,
        data: Option<Binary>,
    },
    AccountUpdateService {
        service_name: String,
        data: Binary,
    },
    AccountDeregisterService {
        service_name: String,
    },
    AccountPauseService {
        service_name: String,
    },
    AccountResumeService {
        service_name: String,
    },
    AccountSendFundsByService {
        service_name: String,
        msgs: Vec<SubMsg>,
        atomic: bool,
    },
    AccountExecuteByService {
        service_name: String,
        msgs: Vec<SubMsg>,
        atomic: bool,
    },
    AccountExecuteByAdmin {
        msgs: Vec<CosmosMsg>,
    },
    AccountStartAdminChange {
        admin: String,
    },
    AccountCancelAdminChange {},
    AccountApproveAdminChange {},

    // Oracle
    OracleUpdatePrice {
        pair: Pair,
        price: Decimal,
        source: String,
    },
    OracleAddPath {
        pair: Pair,
        path: Vec<E>,
    },
    OracleUpdatePath {
        pair: Pair,
        path: Vec<E>,
    },
    OracleUpdateConfig {
        config: E,
    },
    OracleStartAdminChange {
        admin: String,
    },
    OracleCancelAdminChange {},
    OracleApproveAdminChange {},

    // Auction manager
    AuctionManagerUpdateAuctionCodeId {
        code_id: u64,
    },
    AuctionManagerUpdateOracle {
        oracle_addr: String,
    },
    AuctionManagerMigrateAuction {
        pair: Pair,
        code_id: u64,
        msg: E,
    },
    AuctionManagerUpdateMinAmount {
        denom: String,
        min_amount: MinAmount,
    },
    AuctionManagerStartAdminChange {
        admin: String,
    },
    AuctionManagerCancelAdminChange {},
    AuctionManagerApproveAdminChange {},

    // Auctions
    AuctionInit {
        config: AuctionConfig,
        strategy: AuctionStrategy,
    },
    AuctionAuctionFunds {
        address: String,
        amount: Uint128,
        auction_id: u64,
    },
    AuctionWithdrawFunds {
        address: String,
        amount: Uint128,
        auction_id: u64,
    },
    AuctionDoBid {
        auction_id: u64,
        bidder: String,
        price: Decimal,
        /// How much of token.0 the bidder bought
        bought_amount: Uint128,
        /// If bidder sent too much and we couldn't "swap" all, then we refund him the rest
        refunded_amount: Uint128,
    },
    AuctionPause {},
    AuctionResume {},
    AuctionUpdateStrategy {
        strategy: AuctionStrategy,
    },
    AuctionUpdateChainHaltConfig {
        halt_config: ChainHaltConfig,
    },
    AuctionUpdatePriceFreshnessStrategy {
        strategy: PriceFreshnessStrategy,
    },
    AuctionOpen {
        auction_id: u64,
        auction: E,
    },
    AuctionOpenRefund {
        auction_id: u64,
        min_amount: Uint128,
        refund_amount: Uint128,
        total_users: u64,
    },
    AuctionClose {
        auction_id: u64,
        is_closed: bool,
        price: String,
        accounts: u64,
    },

    // Services manager
    ServicesManagerAddService {
        service_name: String,
        addr: String,
    },
    ServicesManagerUpdateService {
        service_name: String,
        addr: String,
    },
    ServicesManagerRemoveService {
        service_name: String,
    },
    ServicesManagerUpdateCodeIdWhitelist {
        to_add: Vec<u64>,
        to_remove: Vec<u64>,
    },
    ServicesManagerWithdraw {
        amount: Coin,
    },
    ServicesManagerStartAdminChange {
        admin: String,
    },
    ServicesManagerCancelAdminChange {},
    ServicesManagerApproveAdminChange {},

    // Rebalancer
    RebalancerRegister {
        account: String,
        config: RebalancerConfig,
    },
    RebalancerDeregister {
        account: String,
    },
    RebalancerUpdate {
        account: String,
        config: RebalancerConfig,
    },
    RebalancerPause {
        account: String,
        reason: String,
    },
    RebalancerResume {
        account: String,
    },
    RebalancerUpdateSystemStatus {
        status: SystemRebalanceStatus,
    },
    RebalancerUpdateDenomWhitelist {
        denoms: HashSet<String>,
    },
    RebalancerUpdateBaseDenomWhitelist {
        base_denoms: HashSet<BaseDenom>,
    },
    RebalancerUpdateServicesManager {
        addr: String,
    },
    RebalancerUpdateAuctionsManager {
        addr: String,
    },
    RebalancerUpdateCyclePeriod {
        period: u64,
    },
    RebalancerUpdateFees {
        fees: ServiceFeeConfig,
    },
    RebalancerStartAdminChange {
        admin: String,
    },
    RebalancerCancelAdminChange {},
    RebalancerApproveAdminChange {},
    RebalancerCycle {
        limit: u64,
        cycled_over: u64,
    },
    RebalancerAccountRebalance {
        account: String,
        total_value: Decimal,
        trades: Vec<RebalanceTrade>,
    },
    RebalancerAccountRebalancePause {
        account: String,
        total_value: Decimal,
    },
}

/// Turn a ValenceServices enum into a string
impl<E: serde::Serialize> fmt::Display for ValenceEvent<E> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            // Account
            ValenceEvent::AccountCreation { .. } => write!(f, "account-creation"),
            ValenceEvent::AccountRegisterService { .. } => write!(f, "account-register-service"),
            ValenceEvent::AccountUpdateService { .. } => write!(f, "account-update-service"),
            ValenceEvent::AccountDeregisterService { .. } => {
                write!(f, "account-deregister-service")
            }
            ValenceEvent::AccountPauseService { .. } => write!(f, "account-pause-service"),
            ValenceEvent::AccountResumeService { .. } => write!(f, "account-resume-service"),
            ValenceEvent::AccountSendFundsByService { .. } => {
                write!(f, "account-send-funds-by-service")
            }
            ValenceEvent::AccountExecuteByService { .. } => write!(f, "account-execute-by-service"),
            ValenceEvent::AccountExecuteByAdmin { .. } => write!(f, "account-execute-by-admin"),
            ValenceEvent::AccountStartAdminChange { .. } => write!(f, "account-start-admin-change"),
            ValenceEvent::AccountCancelAdminChange {} => write!(f, "account-cancel-admin-change"),
            ValenceEvent::AccountApproveAdminChange {} => write!(f, "account-approve-admin-change"),

            // oracle
            ValenceEvent::OracleUpdatePrice { .. } => write!(f, "oracle-update-price"),
            ValenceEvent::OracleAddPath { .. } => write!(f, "oracle-add-path"),
            ValenceEvent::OracleUpdatePath { .. } => write!(f, "oracle-update-path"),
            ValenceEvent::OracleUpdateConfig { .. } => write!(f, "oracle-update-config"),
            ValenceEvent::OracleStartAdminChange { .. } => write!(f, "oracle-start-admin-change"),
            ValenceEvent::OracleCancelAdminChange {} => write!(f, "oracle-cancel-admin-change"),
            ValenceEvent::OracleApproveAdminChange {} => write!(f, "oracle-approve-admin-change"),

            // Auction manager
            ValenceEvent::AuctionManagerUpdateAuctionCodeId { .. } => {
                write!(f, "auction-manager-update-auction-code-id")
            }
            ValenceEvent::AuctionManagerUpdateOracle { .. } => {
                write!(f, "auction-manager-update-oracle")
            }
            ValenceEvent::AuctionManagerMigrateAuction { .. } => {
                write!(f, "auction-manager-migrate-auction")
            }
            ValenceEvent::AuctionManagerUpdateMinAmount { .. } => {
                write!(f, "auction-manager-update-min-amount")
            }
            ValenceEvent::AuctionManagerStartAdminChange { .. } => {
                write!(f, "auction-manager-start-admin-change")
            }
            ValenceEvent::AuctionManagerCancelAdminChange {} => {
                write!(f, "auction-manager-cancel-admin-change")
            }
            ValenceEvent::AuctionManagerApproveAdminChange {} => {
                write!(f, "auction-manager-approve-admin-change")
            }

            // auctions
            ValenceEvent::AuctionInit { .. } => write!(f, "auction-init"),
            ValenceEvent::AuctionAuctionFunds { .. } => write!(f, "auction-auction-funds"),
            ValenceEvent::AuctionWithdrawFunds { .. } => write!(f, "auction-withdraw-funds"),
            ValenceEvent::AuctionDoBid { .. } => write!(f, "auction-do-bid"),
            ValenceEvent::AuctionPause {} => write!(f, "auction-pause"),
            ValenceEvent::AuctionResume {} => write!(f, "auction-resume"),
            ValenceEvent::AuctionUpdateStrategy { .. } => write!(f, "auction-update-strategy"),
            ValenceEvent::AuctionUpdateChainHaltConfig { .. } => {
                write!(f, "auction-update-chain-halt-config")
            }
            ValenceEvent::AuctionUpdatePriceFreshnessStrategy { .. } => {
                write!(f, "auction-update-price-freshness-strategy")
            }
            ValenceEvent::AuctionOpen { .. } => write!(f, "auction-open"),
            ValenceEvent::AuctionOpenRefund { .. } => write!(f, "auction-open-refund"),
            ValenceEvent::AuctionClose { .. } => write!(f, "auction-close"),

            // Services manager
            ValenceEvent::ServicesManagerAddService { .. } => {
                write!(f, "services-manager-add-service")
            }
            ValenceEvent::ServicesManagerUpdateService { .. } => {
                write!(f, "services-manager-update-service")
            }
            ValenceEvent::ServicesManagerRemoveService { .. } => {
                write!(f, "services-manager-remove-service")
            }
            ValenceEvent::ServicesManagerUpdateCodeIdWhitelist { .. } => {
                write!(f, "services-manager-update-code-id-whitelist")
            }
            ValenceEvent::ServicesManagerWithdraw { .. } => write!(f, "services-manager-withdraw"),
            ValenceEvent::ServicesManagerStartAdminChange { .. } => {
                write!(f, "services-manager-start-admin-change")
            }
            ValenceEvent::ServicesManagerCancelAdminChange {} => {
                write!(f, "services-manager-cancel-admin-change")
            }
            ValenceEvent::ServicesManagerApproveAdminChange {} => {
                write!(f, "services-manager-approve-admin-change")
            }

            // Rebalancer
            ValenceEvent::RebalancerRegister { .. } => write!(f, "rebalancer-register"),
            ValenceEvent::RebalancerDeregister { .. } => write!(f, "rebalancer-deregister"),
            ValenceEvent::RebalancerUpdate { .. } => write!(f, "rebalancer-update"),
            ValenceEvent::RebalancerPause { .. } => write!(f, "rebalancer-pause"),
            ValenceEvent::RebalancerResume { .. } => write!(f, "rebalancer-resume"),
            ValenceEvent::RebalancerUpdateSystemStatus { .. } => {
                write!(f, "rebalancer-update-system-status")
            }
            ValenceEvent::RebalancerUpdateDenomWhitelist { .. } => {
                write!(f, "rebalancer-update-denom-whitelist")
            }
            ValenceEvent::RebalancerUpdateBaseDenomWhitelist { .. } => {
                write!(f, "rebalancer-update-base-denom-whitelist")
            }
            ValenceEvent::RebalancerUpdateServicesManager { .. } => {
                write!(f, "rebalancer-update-services-manager")
            }
            ValenceEvent::RebalancerUpdateAuctionsManager { .. } => {
                write!(f, "rebalancer-update-auctions-manager")
            }
            ValenceEvent::RebalancerUpdateCyclePeriod { .. } => {
                write!(f, "rebalancer-update-cycle-period")
            }
            ValenceEvent::RebalancerUpdateFees { .. } => write!(f, "rebalancer-update-fees"),
            ValenceEvent::RebalancerStartAdminChange { .. } => {
                write!(f, "rebalancer-start-admin-change")
            }
            ValenceEvent::RebalancerCancelAdminChange {} => {
                write!(f, "rebalancer-cancel-admin-change")
            }
            ValenceEvent::RebalancerApproveAdminChange {} => {
                write!(f, "rebalancer-approve-admin-change")
            }
            ValenceEvent::RebalancerCycle { .. } => write!(f, "rebalancer-cycle"),
            ValenceEvent::RebalancerAccountRebalance { .. } => {
                write!(f, "rebalancer-account-rebalance")
            }
            ValenceEvent::RebalancerAccountRebalancePause { .. } => {
                write!(f, "rebalancer-account-rebalance-pause")
            }
        }
    }
}

impl<E: serde::Serialize> From<ValenceEvent<E>> for Event {
    fn from(value: ValenceEvent<E>) -> Self {
        Event::new("valence-event")
            .add_attribute("action", value.to_string())
            .add_attribute("data", to_json_binary(&value).unwrap().to_string())
    }
}
