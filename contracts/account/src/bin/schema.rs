use cosmwasm_schema::write_api;

use valence_account::msg::{InstantiateMsg, QueryMsg};
use valence_package::msgs::core_execute::AccountBaseExecuteMsg;

fn main() {
    write_api! {
        instantiate: InstantiateMsg,
        execute: AccountBaseExecuteMsg,
        query: QueryMsg,
    }
}
