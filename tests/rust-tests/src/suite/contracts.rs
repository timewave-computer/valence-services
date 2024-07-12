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
    .with_reply(rebalancer::contract::reply)
    .with_migrate(rebalancer::contract::migrate);
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

pub fn astro_token_contract() -> Box<dyn Contract<Empty>> {
    Box::new(
        ContractWrapper::new(
            astroport_token::contract::execute,
            astroport_token::contract::instantiate,
            astroport_token::contract::query,
        )
        .with_migrate(astroport_token::contract::migrate),
    )
}

pub fn astro_factory_contract() -> Box<dyn Contract<Empty>> {
    Box::new(
        ContractWrapper::new(
            astroport_factory::contract::execute,
            astroport_factory::contract::instantiate,
            astroport_factory::contract::query,
        )
        .with_migrate(astroport_factory::contract::migrate)
        .with_reply(astroport_factory::contract::reply),
    )
}

pub fn astro_pair_contract() -> Box<dyn Contract<Empty>> {
    Box::new(
        ContractWrapper::new(
            astroport_pair::contract::execute,
            astroport_pair::contract::instantiate,
            astroport_pair::contract::query,
        )
        .with_reply(astroport_pair::contract::reply)
        .with_migrate(astroport_pair::contract::migrate),
    )
}

pub fn astro_pair_stable_contract() -> Box<dyn Contract<Empty>> {
    Box::new(
        ContractWrapper::new(
            astroport_pair_stable::contract::execute,
            astroport_pair_stable::contract::instantiate,
            astroport_pair_stable::contract::query,
        )
        .with_reply(astroport_pair_stable::contract::reply)
        .with_migrate(astroport_pair_stable::contract::migrate),
    )
}

pub fn astro_coin_registry_contract() -> Box<dyn Contract<Empty>> {
    let registry_contract = ContractWrapper::new(
        astroport_native_coin_registry::contract::execute,
        astroport_native_coin_registry::contract::instantiate,
        astroport_native_coin_registry::contract::query,
    )
    .with_migrate(astroport_native_coin_registry::contract::migrate);

    Box::new(registry_contract)
}
