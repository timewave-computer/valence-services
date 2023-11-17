use cosmwasm_schema::cw_serde;
use cosmwasm_std::Addr;
use cw_storage_plus::Item;
use cw_utils::Expiration;

/// State to store the address of the services manager contract.
pub const SERVICES_MANAGER: Item<Addr> = Item::new("services_manager");

/// State to store the address of the admin of the contract.
pub const ADMIN: Item<Addr> = Item::new("admin");

/// State for when an admin want to set someone else as admin.
pub const ADMIN_CHANGE: Item<AdminChange> = Item::new("admin_change");

#[cw_serde]
pub struct AdminChange {
    pub addr: Addr,
    pub expiration: Expiration,
}
