use std::fmt;

use cosmwasm_schema::cw_serde;
use cosmwasm_std::{to_json_binary, Binary, Coin, CosmosMsg, Event, SubMsg};

#[cw_serde]
pub enum EventIndex {
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
impl fmt::Display for EventIndex {
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

impl From<EventIndex> for Event {
    fn from(value: EventIndex) -> Self {
        Event::new("valence")
            .add_attribute("action", value.to_string())
            .add_attribute("data", to_json_binary(&value).unwrap().to_string())
    }
}
