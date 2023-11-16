use cosmwasm_std::Empty;
use cw_multi_test::{Contract, ContractWrapper};

pub fn account_contract() -> Box<dyn Contract<Empty>> {
    let contract = ContractWrapper::new(
        valence_account::contract::execute,
        valence_account::contract::instantiate,
        valence_account::contract::query,
    )
    .with_reply(valence_account::contract::reply);
    Box::new(contract)
}

pub fn services_manager_contract() -> Box<dyn Contract<Empty>> {
    let contract = ContractWrapper::new(
        services_manager::contract::execute,
        services_manager::contract::instantiate,
        services_manager::contract::query,
    );
    Box::new(contract)
}

pub fn rebalancer_contract() -> Box<dyn Contract<Empty>> {
    let contract = ContractWrapper::new(
        rebalancer::contract::execute,
        rebalancer::contract::instantiate,
        rebalancer::contract::query,
    )
    .with_reply(rebalancer::contract::reply);
    Box::new(contract)
}

pub fn auction_contract() -> Box<dyn Contract<Empty>> {
    let contract = ContractWrapper::new(
        auction::contract::execute,
        auction::contract::instantiate,
        auction::contract::query,
    );
    Box::new(contract)
}

pub fn auctions_manager_contract() -> Box<dyn Contract<Empty>> {
    let contract = ContractWrapper::new(
        auctions_manager::contract::execute,
        auctions_manager::contract::instantiate,
        auctions_manager::contract::query,
    )
    .with_reply(auctions_manager::contract::reply);
    Box::new(contract)
}

pub fn oracle_contract() -> Box<dyn Contract<Empty>> {
    let contract = ContractWrapper::new(
        price_oracle::contract::execute,
        price_oracle::contract::instantiate,
        price_oracle::contract::query,
    );
    Box::new(contract)
}
