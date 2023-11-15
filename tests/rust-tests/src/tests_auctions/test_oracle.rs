use cosmwasm_std::{coins, Addr, Decimal};
use cw_multi_test::Executor;

use crate::suite::{
    suite::{Suite, DAY, DEFAULT_BLOCK_TIME},
    suite_builder::SuiteBuilder,
};

#[test]
fn test_update_price_manually() {
    let mut suite = SuiteBuilder::default().build_basic();

    let price = Decimal::bps(5000);
    suite.update_price(suite.pair.clone(), Some(price)).unwrap();

    let price_res = suite.query_oracle_price(suite.pair.clone());
    assert_eq!(price_res.price, price);
}

#[test]
fn test_update_price_from_auctions() {
    let mut suite = Suite::default();
    let funds = coins(100_u128, suite.pair.0.clone());

    // do 3 auctions
    suite.finalize_auction(&funds);
    suite.finalize_auction(&funds);
    suite.finalize_auction(&funds);

    // Update the price from twap
    suite.update_price(suite.pair.clone(), None).unwrap();

    // Get the price which should be an average of 1.5
    let price_res = suite.query_oracle_price(suite.pair.clone());
    let rounded_price =
        (price_res.price * Decimal::from_atomics(100_u128, 0).unwrap()).to_uint_floor();
    assert_eq!(rounded_price.u128(), 150_u128); // 150 / 100 = 1.50
}

#[test]
fn test_twap_less_then_3_auctions() {
    let mut suite = Suite::default();
    let funds = coins(1000_u128, suite.pair.0.clone());

    // do auction
    suite.finalize_auction(&funds);

    let err = suite.update_price_err(suite.pair.clone(), None);
    assert_eq!(err, price_oracle::error::ContractError::NotEnoughTwaps)
}

#[test]
fn test_twap_no_recent_auction() {
    let mut suite = Suite::default();
    let funds = coins(100_u128, suite.pair.0.clone());

    // do 3 auctions
    suite.finalize_auction(&funds);
    suite.finalize_auction(&funds);
    suite.finalize_auction(&funds);

    // Move chain 6 days ahead
    suite.update_block(DAY * 4 / DEFAULT_BLOCK_TIME);

    let err = suite.update_price_err(suite.pair.clone(), None);
    assert_eq!(
        err,
        price_oracle::error::ContractError::NoAuctionInLast3Days
    )
}

#[test]
fn test_not_admin() {
    let mut suite = Suite::default();
    let funds = coins(100_u128, suite.pair.0.clone());

    // do 3 auctions
    suite.finalize_auction(&funds);
    suite.finalize_auction(&funds);
    suite.finalize_auction(&funds);

    let err: price_oracle::error::ContractError = suite
        .app
        .execute_contract(
            Addr::unchecked("not_admin"),
            suite.oracle_addr,
            &price_oracle::msg::ExecuteMsg::UpdatePrice {
                pair: suite.pair,
                price: None,
            },
            &[],
        )
        .unwrap_err()
        .downcast()
        .unwrap();

    assert_eq!(err, price_oracle::error::ContractError::NotAdmin)
}

#[test]
fn test_config() {
    let suite = Suite::default();
    let config = suite.query_oracle_config();
    assert_eq!(
        config,
        price_oracle::state::Config {
            admin: suite.admin,
            auction_manager_addr: suite.auctions_manager_addr,
        }
    )
}

#[test]
fn test_update_price_0() {
    let mut suite = SuiteBuilder::default().build_basic();

    let price: Decimal = Decimal::zero();
    let err = suite.update_price_err(suite.pair.clone(), Some(price));

    assert_eq!(err, price_oracle::error::ContractError::PriceIsZero)
}
