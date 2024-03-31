use cosmwasm_std::{coins, Addr, Decimal};
use cw_multi_test::Executor;
use cw_utils::Expiration;

use crate::suite::{
    suite::{Suite, DAY, DEFAULT_BLOCK_TIME},
    suite_builder::SuiteBuilder,
};

#[test]
fn test_update_price_manually() {
    let mut suite = SuiteBuilder::default().build_basic(false);

    let price = Decimal::bps(5000);
    suite
        .manual_update_price(suite.pair.clone(), price)
        .unwrap();

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
    suite.update_price(suite.pair.clone()).unwrap();

    // Get the price which should be an average of 1.5
    let price_res = suite.query_oracle_price(suite.pair.clone());
    let rounded_price =
        (price_res.price * Decimal::from_atomics(100_u128, 0).unwrap()).to_uint_floor();
    assert_eq!(rounded_price.u128(), 150_u128); // 150 / 100 = 1.50
}

// TODO: Should fallback to astroport and not error, remove once astroport test is added
#[test]
fn test_twap_less_then_3_auctions() {
    let mut suite = Suite::default();
    let funds = coins(1000_u128, suite.pair.0.clone());

    // do auction
    suite.finalize_auction(&funds);

    let _ = suite.update_price_err(suite.pair.clone());
    // assert_eq!(err, price_oracle::error::ContractError::NotEnoughTwaps)
}

// NOTE: This doesn't actually test
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

    let err = suite.update_price_err(suite.pair.clone());
    assert_eq!(
        err,
        price_oracle::error::ContractError::NoAstroPath(suite.pair.clone())
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

    // Should error because we are not the admin
    suite
        .app
        .execute_contract(
            Addr::unchecked("not_admin"),
            suite.oracle_addr,
            &price_oracle::msg::ExecuteMsg::ManualPriceUpdate {
                pair: suite.pair,
                price: Decimal::one(),
            },
            &[],
        )
        .unwrap_err();
}

#[test]
fn test_config() {
    let suite = Suite::default();
    let config = suite.query_oracle_config();
    assert_eq!(
        config,
        price_oracle::state::Config {
            auction_manager_addr: suite.auctions_manager_addr,
            seconds_allow_manual_change: 60 * 60 * 24 * 2,
            seconds_auction_prices_fresh: 60 * 60 * 24 * 3,
        }
    )
}

#[test]
fn test_update_price_0() {
    let mut suite = SuiteBuilder::default().build_basic(true);

    let price: Decimal = Decimal::zero();
    let err = suite.manual_update_price_err(suite.pair.clone(), price);

    assert_eq!(err, price_oracle::error::ContractError::PriceIsZero)
}

#[test]
fn test_update_admin_start() {
    let mut suite = Suite::default();
    let new_admin = Addr::unchecked("random_addr");

    // Try to approve admin without starting a new change
    // should error
    suite
        .app
        .execute_contract(
            new_admin.clone(),
            suite.oracle_addr.clone(),
            &price_oracle::msg::ExecuteMsg::ApproveAdminChange {},
            &[],
        )
        .unwrap_err();

    suite
        .app
        .execute_contract(
            suite.admin.clone(),
            suite.oracle_addr.clone(),
            &price_oracle::msg::ExecuteMsg::StartAdminChange {
                addr: new_admin.to_string(),
                expiration: Expiration::Never {},
            },
            &[],
        )
        .unwrap();

    suite
        .app
        .execute_contract(
            new_admin.clone(),
            suite.oracle_addr.clone(),
            &price_oracle::msg::ExecuteMsg::ApproveAdminChange {},
            &[],
        )
        .unwrap();

    let admin = suite.query_admin(&suite.oracle_addr).unwrap();
    assert_eq!(admin, new_admin)
}

#[test]
fn test_update_admin_cancel() {
    let mut suite = Suite::default();
    let new_admin = Addr::unchecked("new_admin_addr");

    suite
        .app
        .execute_contract(
            suite.admin.clone(),
            suite.oracle_addr.clone(),
            &price_oracle::msg::ExecuteMsg::StartAdminChange {
                addr: new_admin.to_string(),
                expiration: Expiration::Never {},
            },
            &[],
        )
        .unwrap();

    suite
        .app
        .execute_contract(
            suite.admin.clone(),
            suite.oracle_addr.clone(),
            &price_oracle::msg::ExecuteMsg::CancelAdminChange {},
            &[],
        )
        .unwrap();

    // Should error because we cancelled the admin change
    suite
        .app
        .execute_contract(
            new_admin,
            suite.oracle_addr.clone(),
            &price_oracle::msg::ExecuteMsg::ApproveAdminChange {},
            &[],
        )
        .unwrap_err();
}

#[test]
fn test_update_admin_fails() {
    let mut suite = Suite::default();
    let new_admin = Addr::unchecked("new_admin_addr");
    let random_addr = Addr::unchecked("random_addr");

    suite
        .app
        .execute_contract(
            suite.admin.clone(),
            suite.oracle_addr.clone(),
            &price_oracle::msg::ExecuteMsg::StartAdminChange {
                addr: new_admin.to_string(),
                expiration: Expiration::AtHeight(suite.app.block_info().height + 5),
            },
            &[],
        )
        .unwrap();

    // Should fail because we are not the new admin
    suite
        .app
        .execute_contract(
            random_addr,
            suite.oracle_addr.clone(),
            &price_oracle::msg::ExecuteMsg::ApproveAdminChange {},
            &[],
        )
        .unwrap_err();

    suite.update_block_cycle();

    // Should fail because expired
    suite
        .app
        .execute_contract(
            new_admin,
            suite.oracle_addr.clone(),
            &price_oracle::msg::ExecuteMsg::ApproveAdminChange {},
            &[],
        )
        .unwrap_err();
}

#[test]
fn test_manual_price_update() {
    let mut suite = SuiteBuilder::default().build_basic(false);
    let funds = coins(10_u128, suite.pair.0.clone());

    // no auctions yet, so should be able to update
    suite
        .manual_update_price(suite.pair.clone(), Decimal::one())
        .unwrap();

    // 4 auctions passed we should not be able to update price now.
    suite.finalize_auction(&funds);
    suite.finalize_auction(&funds);
    suite.finalize_auction(&funds);
    suite.finalize_auction(&funds);

    suite.update_price(suite.pair.clone()).unwrap();

    let err = suite.manual_update_price_err(suite.pair.clone(), Decimal::one());
    assert_eq!(
        err,
        price_oracle::error::ContractError::NoTermsForManualUpdate
    );

    // 3 days passed without auction, we should be able to update price now.
    suite.update_block_cycle();
    suite.update_block_cycle();
    suite.update_block_cycle();

    suite
        .manual_update_price(suite.pair.clone(), Decimal::one())
        .unwrap();

    // an auction happened, we should not be able to update price now.
    suite.finalize_auction(&funds);

    let err = suite.manual_update_price_err(suite.pair.clone(), Decimal::one());
    assert_eq!(
        err,
        price_oracle::error::ContractError::NoTermsForManualUpdate
    );
}
