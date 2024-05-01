use std::{collections::HashSet, fmt};

use auction_package::{
    helpers::{AuctionConfig, ChainHaltConfig},
    states::MinAmount,
    AuctionStrategy, Pair, PriceFreshnessStrategy,
};
use cosmwasm_std::{to_json_binary, Binary, Coin, CosmosMsg, Decimal, Event, SubMsg, Uint128};
use serde::Serialize;

use crate::services::rebalancer::{
    BaseDenom, RebalanceTrade, RebalancerConfig, ServiceFeeConfig, SystemRebalanceStatus,
};

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
pub enum EventIndex<E>
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
impl<E: serde::Serialize> fmt::Display for EventIndex<E> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            // Account
            EventIndex::AccountCreation { .. } => write!(f, "account-creation"),
            EventIndex::AccountRegisterService { .. } => write!(f, "account-register-service"),
            EventIndex::AccountUpdateService { .. } => write!(f, "account-update-service"),
            EventIndex::AccountDeregisterService { .. } => write!(f, "account-deregister-service"),
            EventIndex::AccountPauseService { .. } => write!(f, "account-pause-service"),
            EventIndex::AccountResumeService { .. } => write!(f, "account-resume-service"),
            EventIndex::AccountSendFundsByService { .. } => {
                write!(f, "account-send-funds-by-service")
            }
            EventIndex::AccountExecuteByService { .. } => write!(f, "account-execute-by-service"),
            EventIndex::AccountExecuteByAdmin { .. } => write!(f, "account-execute-by-admin"),
            EventIndex::AccountStartAdminChange { .. } => write!(f, "account-start-admin-change"),
            EventIndex::AccountCancelAdminChange {} => write!(f, "account-cancel-admin-change"),
            EventIndex::AccountApproveAdminChange {} => write!(f, "account-approve-admin-change"),

            // oracle
            EventIndex::OracleUpdatePrice { .. } => write!(f, "oracle-update-price"),
            EventIndex::OracleAddPath { .. } => write!(f, "oracle-add-path"),
            EventIndex::OracleUpdatePath { .. } => write!(f, "oracle-update-path"),
            EventIndex::OracleUpdateConfig { .. } => write!(f, "oracle-update-config"),
            EventIndex::OracleStartAdminChange { .. } => write!(f, "oracle-start-admin-change"),
            EventIndex::OracleCancelAdminChange {} => write!(f, "oracle-cancel-admin-change"),
            EventIndex::OracleApproveAdminChange {} => write!(f, "oracle-approve-admin-change"),

            // Auction manager
            EventIndex::AuctionManagerUpdateAuctionCodeId { .. } => {
                write!(f, "auction-manager-update-auction-code-id")
            }
            EventIndex::AuctionManagerUpdateOracle { .. } => {
                write!(f, "auction-manager-update-oracle")
            }
            EventIndex::AuctionManagerMigrateAuction { .. } => {
                write!(f, "auction-manager-migrate-auction")
            }
            EventIndex::AuctionManagerUpdateMinAmount { .. } => {
                write!(f, "auction-manager-update-min-amount")
            }
            EventIndex::AuctionManagerStartAdminChange { .. } => {
                write!(f, "auction-manager-start-admin-change")
            }
            EventIndex::AuctionManagerCancelAdminChange {} => {
                write!(f, "auction-manager-cancel-admin-change")
            }
            EventIndex::AuctionManagerApproveAdminChange {} => {
                write!(f, "auction-manager-approve-admin-change")
            }

            // auctions
            EventIndex::AuctionInit { .. } => write!(f, "auction-init"),
            EventIndex::AuctionAuctionFunds { .. } => write!(f, "auction-auction-funds"),
            EventIndex::AuctionWithdrawFunds { .. } => write!(f, "auction-withdraw-funds"),
            EventIndex::AuctionDoBid { .. } => write!(f, "auction-do-bid"),
            EventIndex::AuctionPause {} => write!(f, "auction-pause"),
            EventIndex::AuctionResume {} => write!(f, "auction-resume"),
            EventIndex::AuctionUpdateStrategy { .. } => write!(f, "auction-update-strategy"),
            EventIndex::AuctionUpdateChainHaltConfig { .. } => {
                write!(f, "auction-update-chain-halt-config")
            }
            EventIndex::AuctionUpdatePriceFreshnessStrategy { .. } => {
                write!(f, "auction-update-price-freshness-strategy")
            }
            EventIndex::AuctionOpen { .. } => write!(f, "auction-open"),
            EventIndex::AuctionOpenRefund { .. } => write!(f, "auction-open-refund"),
            EventIndex::AuctionClose { .. } => write!(f, "auction-close"),

            // Services manager
            EventIndex::ServicesManagerAddService { .. } => {
                write!(f, "services-manager-add-service")
            }
            EventIndex::ServicesManagerUpdateService { .. } => {
                write!(f, "services-manager-update-service")
            }
            EventIndex::ServicesManagerRemoveService { .. } => {
                write!(f, "services-manager-remove-service")
            }
            EventIndex::ServicesManagerUpdateCodeIdWhitelist { .. } => {
                write!(f, "services-manager-update-code-id-whitelist")
            }
            EventIndex::ServicesManagerWithdraw { .. } => write!(f, "services-manager-withdraw"),
            EventIndex::ServicesManagerStartAdminChange { .. } => {
                write!(f, "services-manager-start-admin-change")
            }
            EventIndex::ServicesManagerCancelAdminChange {} => {
                write!(f, "services-manager-cancel-admin-change")
            }
            EventIndex::ServicesManagerApproveAdminChange {} => {
                write!(f, "services-manager-approve-admin-change")
            }

            // Rebalancer
            EventIndex::RebalancerRegister { .. } => write!(f, "rebalancer-register"),
            EventIndex::RebalancerDeregister { .. } => write!(f, "rebalancer-deregister"),
            EventIndex::RebalancerUpdate { .. } => write!(f, "rebalancer-update"),
            EventIndex::RebalancerPause { .. } => write!(f, "rebalancer-pause"),
            EventIndex::RebalancerResume { .. } => write!(f, "rebalancer-resume"),
            EventIndex::RebalancerUpdateSystemStatus { .. } => {
                write!(f, "rebalancer-update-system-status")
            }
            EventIndex::RebalancerUpdateDenomWhitelist { .. } => {
                write!(f, "rebalancer-update-denom-whitelist")
            }
            EventIndex::RebalancerUpdateBaseDenomWhitelist { .. } => {
                write!(f, "rebalancer-update-base-denom-whitelist")
            }
            EventIndex::RebalancerUpdateServicesManager { .. } => {
                write!(f, "rebalancer-update-services-manager")
            }
            EventIndex::RebalancerUpdateAuctionsManager { .. } => {
                write!(f, "rebalancer-update-auctions-manager")
            }
            EventIndex::RebalancerUpdateCyclePeriod { .. } => {
                write!(f, "rebalancer-update-cycle-period")
            }
            EventIndex::RebalancerUpdateFees { .. } => write!(f, "rebalancer-update-fees"),
            EventIndex::RebalancerStartAdminChange { .. } => {
                write!(f, "rebalancer-start-admin-change")
            }
            EventIndex::RebalancerCancelAdminChange {} => {
                write!(f, "rebalancer-cancel-admin-change")
            }
            EventIndex::RebalancerApproveAdminChange {} => {
                write!(f, "rebalancer-approve-admin-change")
            }
            EventIndex::RebalancerCycle { .. } => write!(f, "rebalancer-cycle"),
            EventIndex::RebalancerAccountRebalance { .. } => {
                write!(f, "rebalancer-account-rebalance")
            }
            EventIndex::RebalancerAccountRebalancePause { .. } => {
                write!(f, "rebalancer-account-rebalance-pause")
            }
        }
    }
}

impl<E: serde::Serialize> From<EventIndex<E>> for Event {
    fn from(value: EventIndex<E>) -> Self {
        Event::new("valence-event")
            .add_attribute("action", value.to_string())
            .add_attribute("data", to_json_binary(&value).unwrap().to_string())
    }
}
