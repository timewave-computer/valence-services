use cosmwasm_schema::{cw_serde, QueryResponses};
use cosmwasm_std::Addr;

#[cw_serde]
pub struct InstantiateMsg {
    /// Address of the service manager contract.
    pub services_manager: String,
}

#[cw_serde]
#[derive(QueryResponses)]
pub enum QueryMsg {
    #[returns(Addr)]
    GetAdmin,
}

#[cw_serde]
pub enum MigrateMsg {
    NoStateChange,
}
