use cosmwasm_std::Addr;
use cw_storage_plus::Item;

pub const SERVER_ADDR: Item<Addr> = Item::new("server_addr");
pub const AUCTION_CODE_ID: Item<u64> = Item::new("auction_code_id");
