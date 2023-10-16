use crate::services::ValenceServices;
use cosmwasm_schema::{cw_serde, QueryResponses};
use cosmwasm_std::Addr;
use valence_macros::valence_services_manager_query_msgs;

#[valence_services_manager_query_msgs]
#[cw_serde]
#[derive(QueryResponses)]
pub enum ServicesManagerQueryMsg {
    // /// Returns true if `address` is in the queue, and false
    // /// otherwise.
    // #[returns(bool)]
    // IsQueued { address: String },
}
