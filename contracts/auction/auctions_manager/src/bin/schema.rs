use auction_package::msgs::AuctionsManagerQueryMsg;
use cosmwasm_schema::write_api;

use auctions_manager::msg::{ExecuteMsg, InstantiateMsg};

fn main() {
    write_api! {
        instantiate: InstantiateMsg,
        execute: ExecuteMsg,
        query: AuctionsManagerQueryMsg,
    }
}
