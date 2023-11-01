use cosmwasm_schema::write_api;

use services_manager::msg::InstantiateMsg;
use valence_package::msgs::core_execute::ServicesManagerExecuteMsg;
use valence_package::msgs::core_query::ServicesManagerQueryMsg;

fn main() {
    write_api! {
        instantiate: InstantiateMsg,
        execute: ServicesManagerExecuteMsg,
        query: ServicesManagerQueryMsg,
    }
}
