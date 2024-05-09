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

pub type ValenceEvent = ValenceGenericEvent<Empty>;

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
pub enum ValenceGenericEvent<E>
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
    AuctionManagerChangeServerAddr {
        addr: String,
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
impl<E: serde::Serialize> fmt::Display for ValenceGenericEvent<E> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            // Account
            ValenceGenericEvent::AccountCreation { .. } => write!(f, "account-creation"),
            ValenceGenericEvent::AccountRegisterService { .. } => {
                write!(f, "account-register-service")
            }
            ValenceGenericEvent::AccountUpdateService { .. } => write!(f, "account-update-service"),
            ValenceGenericEvent::AccountDeregisterService { .. } => {
                write!(f, "account-deregister-service")
            }
            ValenceGenericEvent::AccountPauseService { .. } => write!(f, "account-pause-service"),
            ValenceGenericEvent::AccountResumeService { .. } => write!(f, "account-resume-service"),
            ValenceGenericEvent::AccountSendFundsByService { .. } => {
                write!(f, "account-send-funds-by-service")
            }
            ValenceGenericEvent::AccountExecuteByService { .. } => {
                write!(f, "account-execute-by-service")
            }
            ValenceGenericEvent::AccountExecuteByAdmin { .. } => {
                write!(f, "account-execute-by-admin")
            }
            ValenceGenericEvent::AccountStartAdminChange { .. } => {
                write!(f, "account-start-admin-change")
            }
            ValenceGenericEvent::AccountCancelAdminChange {} => {
                write!(f, "account-cancel-admin-change")
            }
            ValenceGenericEvent::AccountApproveAdminChange {} => {
                write!(f, "account-approve-admin-change")
            }

            // oracle
            ValenceGenericEvent::OracleUpdatePrice { .. } => write!(f, "oracle-update-price"),
            ValenceGenericEvent::OracleAddPath { .. } => write!(f, "oracle-add-path"),
            ValenceGenericEvent::OracleUpdatePath { .. } => write!(f, "oracle-update-path"),
            ValenceGenericEvent::OracleUpdateConfig { .. } => write!(f, "oracle-update-config"),
            ValenceGenericEvent::OracleStartAdminChange { .. } => {
                write!(f, "oracle-start-admin-change")
            }
            ValenceGenericEvent::OracleCancelAdminChange {} => {
                write!(f, "oracle-cancel-admin-change")
            }
            ValenceGenericEvent::OracleApproveAdminChange {} => {
                write!(f, "oracle-approve-admin-change")
            }

            // Auction manager
            ValenceGenericEvent::AuctionManagerUpdateAuctionCodeId { .. } => {
                write!(f, "auction-manager-update-auction-code-id")
            }
            ValenceGenericEvent::AuctionManagerUpdateOracle { .. } => {
                write!(f, "auction-manager-update-oracle")
            }
            ValenceGenericEvent::AuctionManagerMigrateAuction { .. } => {
                write!(f, "auction-manager-migrate-auction")
            }
            ValenceGenericEvent::AuctionManagerUpdateMinAmount { .. } => {
                write!(f, "auction-manager-update-min-amount")
            }
            ValenceGenericEvent::AuctionManagerStartAdminChange { .. } => {
                write!(f, "auction-manager-start-admin-change")
            }
            ValenceGenericEvent::AuctionManagerCancelAdminChange {} => {
                write!(f, "auction-manager-cancel-admin-change")
            }
            ValenceGenericEvent::AuctionManagerApproveAdminChange {} => {
                write!(f, "auction-manager-approve-admin-change")
            }
            ValenceEvent::AuctionManagerChangeServerAddr { .. } => {
                write!(f, "auction-manager-change-server-addr")
            }

            // auctions
            ValenceGenericEvent::AuctionInit { .. } => write!(f, "auction-init"),
            ValenceGenericEvent::AuctionAuctionFunds { .. } => write!(f, "auction-auction-funds"),
            ValenceGenericEvent::AuctionWithdrawFunds { .. } => write!(f, "auction-withdraw-funds"),
            ValenceGenericEvent::AuctionDoBid { .. } => write!(f, "auction-do-bid"),
            ValenceGenericEvent::AuctionPause {} => write!(f, "auction-pause"),
            ValenceGenericEvent::AuctionResume {} => write!(f, "auction-resume"),
            ValenceGenericEvent::AuctionUpdateStrategy { .. } => {
                write!(f, "auction-update-strategy")
            }
            ValenceGenericEvent::AuctionUpdateChainHaltConfig { .. } => {
                write!(f, "auction-update-chain-halt-config")
            }
            ValenceGenericEvent::AuctionUpdatePriceFreshnessStrategy { .. } => {
                write!(f, "auction-update-price-freshness-strategy")
            }
            ValenceGenericEvent::AuctionOpen { .. } => write!(f, "auction-open"),
            ValenceGenericEvent::AuctionOpenRefund { .. } => write!(f, "auction-open-refund"),
            ValenceGenericEvent::AuctionClose { .. } => write!(f, "auction-close"),

            // Services manager
            ValenceGenericEvent::ServicesManagerAddService { .. } => {
                write!(f, "services-manager-add-service")
            }
            ValenceGenericEvent::ServicesManagerUpdateService { .. } => {
                write!(f, "services-manager-update-service")
            }
            ValenceGenericEvent::ServicesManagerRemoveService { .. } => {
                write!(f, "services-manager-remove-service")
            }
            ValenceGenericEvent::ServicesManagerUpdateCodeIdWhitelist { .. } => {
                write!(f, "services-manager-update-code-id-whitelist")
            }
            ValenceGenericEvent::ServicesManagerWithdraw { .. } => {
                write!(f, "services-manager-withdraw")
            }
            ValenceGenericEvent::ServicesManagerStartAdminChange { .. } => {
                write!(f, "services-manager-start-admin-change")
            }
            ValenceGenericEvent::ServicesManagerCancelAdminChange {} => {
                write!(f, "services-manager-cancel-admin-change")
            }
            ValenceGenericEvent::ServicesManagerApproveAdminChange {} => {
                write!(f, "services-manager-approve-admin-change")
            }

            // Rebalancer
            ValenceGenericEvent::RebalancerRegister { .. } => write!(f, "rebalancer-register"),
            ValenceGenericEvent::RebalancerDeregister { .. } => write!(f, "rebalancer-deregister"),
            ValenceGenericEvent::RebalancerUpdate { .. } => write!(f, "rebalancer-update"),
            ValenceGenericEvent::RebalancerPause { .. } => write!(f, "rebalancer-pause"),
            ValenceGenericEvent::RebalancerResume { .. } => write!(f, "rebalancer-resume"),
            ValenceGenericEvent::RebalancerUpdateSystemStatus { .. } => {
                write!(f, "rebalancer-update-system-status")
            }
            ValenceGenericEvent::RebalancerUpdateDenomWhitelist { .. } => {
                write!(f, "rebalancer-update-denom-whitelist")
            }
            ValenceGenericEvent::RebalancerUpdateBaseDenomWhitelist { .. } => {
                write!(f, "rebalancer-update-base-denom-whitelist")
            }
            ValenceGenericEvent::RebalancerUpdateServicesManager { .. } => {
                write!(f, "rebalancer-update-services-manager")
            }
            ValenceGenericEvent::RebalancerUpdateAuctionsManager { .. } => {
                write!(f, "rebalancer-update-auctions-manager")
            }
            ValenceGenericEvent::RebalancerUpdateCyclePeriod { .. } => {
                write!(f, "rebalancer-update-cycle-period")
            }
            ValenceGenericEvent::RebalancerUpdateFees { .. } => write!(f, "rebalancer-update-fees"),
            ValenceGenericEvent::RebalancerStartAdminChange { .. } => {
                write!(f, "rebalancer-start-admin-change")
            }
            ValenceGenericEvent::RebalancerCancelAdminChange {} => {
                write!(f, "rebalancer-cancel-admin-change")
            }
            ValenceGenericEvent::RebalancerApproveAdminChange {} => {
                write!(f, "rebalancer-approve-admin-change")
            }
            ValenceGenericEvent::RebalancerCycle { .. } => write!(f, "rebalancer-cycle"),
            ValenceGenericEvent::RebalancerAccountRebalance { .. } => {
                write!(f, "rebalancer-account-rebalance")
            }
            ValenceGenericEvent::RebalancerAccountRebalancePause { .. } => {
                write!(f, "rebalancer-account-rebalance-pause")
            }
        }
    }
}

impl<E: serde::Serialize> From<ValenceGenericEvent<E>> for Event {
    fn from(value: ValenceGenericEvent<E>) -> Self {
        Event::new("valence-event")
            .add_attribute("action", value.to_string())
            .add_attribute("data", to_json_binary(&value).unwrap().to_string())
    }
}
