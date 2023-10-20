use std::str::FromStr;

use auction_package::{
    error::AuctionError,
    helpers::{ChainHaltConfig, GetPriceResponse},
    PriceFreshnessStrategy,
};
use cosmwasm_std::{coins, Addr, Decimal};
use cw_multi_test::Executor;

use crate::suite::{instantiates::AuctionInstantiate, suite::Suite, suite_builder::SuiteBuilder};

#[test]
fn test_pause_auction() {
    let mut suite = Suite::default();

    suite.pause_auction();

    let config = suite.query_auction_config();
    assert!(config.is_paused)
}

#[test]
fn test_resume_auction() {
    let mut suite = Suite::default();

    suite.pause_auction();

    suite.resume_auction();

    let config = suite.query_auction_config();
    assert!(!config.is_paused)
}

#[test]
fn test_update_oracle_addr() {
    let mut suite = Suite::default();
    let new_oracle_addr = "new_oracle_addr";

    suite.update_oracle(new_oracle_addr);

    let addr = suite.query_oracle_addr();
    assert_eq!(addr.as_str(), new_oracle_addr);
}

#[test]
fn test_auction_funds_from_manager() {
    let mut suite = Suite::default();
    let amount = coins(100u128, suite.pair.0.clone());

    suite.auction_funds_manager(None, &amount);

    let funds_res = suite.query_auction_funds(suite.funds_provider.as_str());
    assert_eq!(funds_res.next, amount[0].amount);
}

#[test]
fn test_not_admin() {
    let mut suite = Suite::default();

    let err = suite
        .app
        .execute_contract(
            Addr::unchecked("not_admin"),
            suite.manager_addr,
            &auctions_manager::msg::ExecuteMsg::Admin(
                auctions_manager::msg::AdminMsgs::PauseAuction { pair: suite.pair },
            ),
            &[],
        )
        .unwrap_err();
    let err = err.source().unwrap().to_string();

    assert_eq!(err, AuctionError::NotAdmin.to_string());
}

#[test]
fn test_no_oracle_addr() {
    let mut suite = SuiteBuilder::default().build_basic();
    suite.init_auction(AuctionInstantiate::default().into(), None);

    let err = suite
        .app
        .wrap()
        .query_wasm_smart::<GetPriceResponse>(
            suite.manager_addr,
            &auction_package::msgs::AuctionsManagerQueryMsg::GetPrice { pair: suite.pair },
        )
        .unwrap_err();

    assert!(err
        .to_string()
        .contains(&auctions_manager::error::ContractError::OracleAddrMissing.to_string()));
}

#[test]
fn test_update_chain_halt_config() {
    let mut suite = Suite::default();

    let config = suite.query_auction_config();
    assert_eq!(
        config.chain_halt_config,
        ChainHaltConfig {
            cap: 60 * 60 * 4,
            block_avg: Decimal::from_str("3.6").unwrap(),
        }
    );

    let new_chain_halt_config = ChainHaltConfig {
        cap: 60 * 60 * 10,
        block_avg: Decimal::from_str("2").unwrap(),
    };
    suite.update_chain_halt_config(suite.pair.clone(), new_chain_halt_config.clone());

    let config = suite.query_auction_config();
    assert_eq!(config.chain_halt_config, new_chain_halt_config);
}

#[test]
fn test_update_price_freshness_strategy() {
    let mut suite = Suite::default();

    let config = suite.query_auction_config();
    assert_eq!(
        config.price_freshness_strategy,
        PriceFreshnessStrategy {
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
        }
    );

    let new_price_freshness_strategy = PriceFreshnessStrategy {
        limit: Decimal::from_str("4").unwrap(),
        multipliers: vec![
            // If older than 2 days, multiplier is 2x the strategy multiplier
            (
                Decimal::from_str("3").unwrap(),
                Decimal::from_str("2").unwrap(),
            ),
            (
                Decimal::from_str("2").unwrap(),
                Decimal::from_str("1.6").unwrap(),
            ),
            // If older than 1 days, multiplier is 1.5x the strategy multiplier
            (Decimal::one(), Decimal::from_str("1.3").unwrap()),
        ],
    };
    suite.update_price_freshness_strategy(suite.pair.clone(), new_price_freshness_strategy.clone());

    let config = suite.query_auction_config();
    assert_eq!(
        config.price_freshness_strategy,
        new_price_freshness_strategy
    );
}
