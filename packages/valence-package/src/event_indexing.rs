use std::fmt;

use auction_package::{states::MinAmount, Pair};
use cosmwasm_schema::cw_serde;
use cosmwasm_std::{to_json_binary, Binary, Coin, CosmosMsg, Decimal, Empty, Event, SubMsg};
use serde::Serialize;

#[cw_serde]
pub enum EventIndex<E = Empty>
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
        }
    }
}

impl<E: serde::Serialize> From<EventIndex<E>> for Event {
    fn from(value: EventIndex<E>) -> Self {
        Event::new("valence")
            .add_attribute("action", value.to_string())
            .add_attribute("data", to_json_binary(&value).unwrap().to_string())
    }
}
