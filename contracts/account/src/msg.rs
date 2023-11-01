use cosmwasm_schema::{cw_serde, QueryResponses};

#[cw_serde]
pub struct InstantiateMsg {
    /// Address of the service manager contract.
    pub services_manager: String,
}

#[cw_serde]
#[derive(QueryResponses)]
pub enum QueryMsg {
    // /// Returns true if `address` is in the queue, and false
    // /// otherwise.
    // #[returns(RebalancerConfig)]
    // getConfig { address: String },
}

#[cw_serde]
pub enum MigrateMsg {}
