use std::{borrow::BorrowMut, str::FromStr};

use auction_package::{states::ADMIN, Pair, PriceFreshnessStrategy};
use cosmwasm_std::Decimal;

use crate::suite::{
    instantiates::{AuctionInstantiate, AuctionsManagerInstantiate, OracleInstantiate},
    suite::{ATOM, NTRN},
    suite_builder::SuiteBuilder,
};

#[test]
fn test_instantiate_auction_manager() {
    let mut builder = SuiteBuilder::default();
    let mut app = builder.set_app();
    builder.upload_contracts(&mut app);

    let manager_addr = builder.init_auctions_manager(
        app.borrow_mut(),
        AuctionsManagerInstantiate::default(builder.auction_code_id).into(),
    );

    let admin_addr = SuiteBuilder::query_wasm_raw_item(&app, manager_addr.clone(), ADMIN);
    assert_eq!(admin_addr, builder.admin);

    let auction_code_id = SuiteBuilder::query_wasm_raw_item(
        &app,
        manager_addr,
        auctions_manager::state::AUCTION_CODE_ID,
    );
    assert_eq!(auction_code_id, builder.auction_code_id);
}

#[test]
fn test_instantiate_oracle() {
    let mut builder = SuiteBuilder::default();
    let mut app = builder.set_app();
    builder.upload_contracts(&mut app);

    let manager_addr = builder.init_auctions_manager(
        app.borrow_mut(),
        AuctionsManagerInstantiate::default(builder.auction_code_id).into(),
    );

    let oracle_addr = builder.init_oracle(
        app.borrow_mut(),
        OracleInstantiate::default(manager_addr.clone()).into(),
    );

    let config = SuiteBuilder::query_wasm_raw_item(&app, oracle_addr, price_oracle::state::CONFIG);
    assert_eq!(
        config,
        price_oracle::state::Config {
            auction_manager_addr: manager_addr,
        }
    )
}

#[test]
fn test_instantiate_auction() {
    let mut suite = SuiteBuilder::default().build_basic();

    let init_msg: auction::msg::InstantiateMsg = AuctionInstantiate::default().into();
    suite.init_auction(suite.pair.clone(), init_msg.clone(), None);

    let admin =
        SuiteBuilder::query_wasm_raw_item(&suite.app, suite.get_default_auction_addr(), ADMIN);
    assert_eq!(admin, suite.auctions_manager_addr);

    let config = suite.query_auction_config(suite.get_default_auction_addr());
    assert_eq!(
        config,
        auction_package::helpers::AuctionConfig {
            is_paused: false,
            pair: init_msg.pair,
            chain_halt_config: init_msg.chain_halt_config,
            price_freshness_strategy: PriceFreshnessStrategy {
                limit: Decimal::from_str("3").unwrap(),
                multipliers: vec![
                    // If older than 2 days, multiplier is 2x the strategy multiplier
                    (
                        Decimal::from_str("2").unwrap(),
                        Decimal::from_str("2").unwrap(),
                    ),
                    // If older than 1 days, multiplier is 1.5x the strategy multiplier
                    (Decimal::one(), Decimal::from_str("1.5").unwrap()),
                ],
            },
        }
    )
}

#[test]
fn test_instantiate_auction_err() {
    let mut suite = SuiteBuilder::default().build_basic();

    let mut init_msg = AuctionInstantiate::default();

    // Empty denom in a pair
    init_msg.change_pair(Pair("".to_string(), NTRN.to_string()));
    let err = suite
        .init_auction_err(init_msg.clone().into(), None)
        .root_cause()
        .to_string();
    assert!(err.contains(&auction_package::error::AuctionError::InvalidPair.to_string()));

    init_msg.change_pair(Pair(ATOM.to_string(), "".to_string()));
    let err = suite
        .init_auction_err(init_msg.clone().into(), None)
        .root_cause()
        .to_string();
    assert!(err.contains(&auction_package::error::AuctionError::InvalidPair.to_string()));

    init_msg.change_pair(Pair("".to_string(), "".to_string()));
    let err = suite
        .init_auction_err(init_msg.clone().into(), None)
        .root_cause()
        .to_string();
    assert!(err.contains(&auction_package::error::AuctionError::InvalidPair.to_string()));

    // Same denom in a pair
    init_msg.change_pair(Pair(ATOM.to_string(), ATOM.to_string()));
    let err = suite
        .init_auction_err(init_msg.into(), None)
        .root_cause()
        .to_string();
    assert!(err.contains(&auction_package::error::AuctionError::InvalidPair.to_string()));
}

#[test]
fn test_auction_strategy() {
    let mut suite = SuiteBuilder::default().build_basic();

    // Try with start price 0
    let mut init_msg: auction::msg::InstantiateMsg = AuctionInstantiate::default().into();
    init_msg.auction_strategy.start_price_perc = 0;

    let err = suite.init_auction_err(init_msg, None);

    assert!(err.root_cause().to_string().contains(
        &auction_package::error::AuctionError::InvalidAuctionStrategyStartPrice.to_string()
    ));

    // Try with end price 0
    let mut init_msg: auction::msg::InstantiateMsg = AuctionInstantiate::default().into();
    init_msg.auction_strategy.end_price_perc = 0;

    let err = suite.init_auction_err(init_msg, None);

    assert!(err.root_cause().to_string().contains(
        &auction_package::error::AuctionError::InvalidAuctionStrategyEndPrice.to_string()
    ));

    // Try with end price over 10000
    let mut init_msg: auction::msg::InstantiateMsg = AuctionInstantiate::default().into();
    init_msg.auction_strategy.end_price_perc = 10001;

    let err = suite.init_auction_err(init_msg, None);

    assert!(err.root_cause().to_string().contains(
        &auction_package::error::AuctionError::InvalidAuctionStrategyEndPrice.to_string()
    ));
}
