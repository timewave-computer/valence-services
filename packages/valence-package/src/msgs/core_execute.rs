use crate::services::ValenceServices;
use cosmwasm_schema::cw_serde;
use cosmwasm_std::{Binary, CosmosMsg};
use valence_macros::{
    valence_account_execute_msgs, valence_rebalancer_msgs, valence_service_manager_admin_msgs,
    valence_service_manager_execute_msgs,
};

/// This is base account execute msgs,
/// it implements messages to be called on the service manager
/// as well as messages to be called by services (valence_account_execute)
#[valence_account_execute_msgs]
#[valence_rebalancer_msgs]
#[cw_serde]
pub enum AccountBaseExecuteMsg {}

/// This is services manager execute msgs,
/// implements messages to be called by accounts on the services (valence_service_execute)
#[valence_service_manager_execute_msgs]
#[valence_service_manager_admin_msgs]
#[valence_rebalancer_msgs]
#[cw_serde]
pub enum ServicesManagerExecuteMsg {}

#[cw_serde]
pub enum ServicesManagerAdminMsg {
    AddService { name: ValenceServices, addr: String },
    UpdateService { name: ValenceServices, addr: String },
    RemoveService { name: ValenceServices },
}
