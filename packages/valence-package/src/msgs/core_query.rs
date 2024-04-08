use crate::services::ValenceServices;
use crate::states::QueryFeeAction;
use cosmwasm_schema::{cw_serde, QueryResponses};
use cosmwasm_std::{Addr, Coin};
use valence_macros::valence_services_manager_query_msgs;

/// Services manager query messages
#[valence_services_manager_query_msgs]
#[cw_serde]
#[derive(QueryResponses)]
pub enum ServicesManagerQueryMsg {
    #[returns(Addr)]
    GetRebalancerConfig { account: String },
}
