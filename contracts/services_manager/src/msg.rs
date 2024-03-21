use cosmwasm_schema::cw_serde;

#[cw_serde]
pub struct InstantiateMsg {
    pub whitelisted_code_ids: Vec<u64>,
}

#[cw_serde]
pub enum MigrateMsg {
    NoStateChange {},
}