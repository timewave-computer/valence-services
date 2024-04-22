use std::fmt;

use cosmwasm_schema::cw_serde;
use cosmwasm_std::{to_json_binary, to_json_string, Binary, Event};

#[cw_serde]
pub enum EventIndex {
    AccountCreation {
        /// The address of the account that was created
        address: String,
        admin: String,
    },
    RebalancerRegister {
        data: Binary,
    },
}

/// Turn a ValenceServices enum into a string
impl fmt::Display for EventIndex {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            EventIndex::AccountCreation { .. } => write!(f, "account-creation"),
            EventIndex::RebalancerRegister { .. } => write!(f, "rebalancer-register"),
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
