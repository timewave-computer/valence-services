use cosmwasm_schema::write_api;

use rebalancer::msg::{InstantiateMsg, QueryMsg};
use valence_package::services::rebalancer::RebalancerExecuteMsg;

fn main() {
    write_api! {
        instantiate: InstantiateMsg,
        execute: RebalancerExecuteMsg,
        query: QueryMsg,
    }
}
