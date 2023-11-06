use cosmwasm_std::Addr;
use cw_storage_plus::{Item, Map};

pub(crate) const SERVICES_TO_ADDR: Map<String, Addr> = Map::new("services_to_addr");
pub(crate) const ADDR_TO_SERVICES: Map<Addr, String> = Map::new("addr_to_services");

pub(crate) const WHITELISTED_CODE_IDS: Item<Vec<u64>> = Item::new("wl_code_ids");