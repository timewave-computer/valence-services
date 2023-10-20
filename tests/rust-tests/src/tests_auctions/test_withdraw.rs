use cosmwasm_std::{coins, testing::mock_env, Uint128};

use crate::suite::suite::Suite;

#[test]
fn test_withdraw() {
    let mut suite = Suite::default();
    let funds = coins(1000_u128, suite.pair.0.clone());
    suite.auction_funds(None, &funds);

    let funds_amount = suite
        .query_auction_funds(suite.funds_provider.as_str())
        .next;
    assert_eq!(funds_amount, funds[0].amount);

    suite.withdraw_funds();

    let funds_amount = suite
        .query_auction_funds(suite.funds_provider.as_str())
        .next;
    assert_eq!(funds_amount, Uint128::zero());
}

#[test]
fn test_withdraw_no_funds() {
    let mut suite = Suite::default();
    let funds = coins(1000_u128, suite.pair.0.clone());
    suite.auction_funds(None, &funds);

    // Withdraw once, success
    suite.withdraw_funds();

    let err = suite.withdraw_funds_err();
    assert_eq!(err, auction::error::ContractError::NoFundsToWithdraw);
}

#[test]
fn test_withdraw_manager() {
    let mut suite = Suite::default();
    let funds = coins(1000_u128, suite.pair.0.clone());
    suite.auction_funds(None, &funds);

    let funds_amount = suite
        .query_auction_funds(suite.funds_provider.as_str())
        .next;
    assert_eq!(funds_amount, funds[0].amount);

    suite.withdraw_funds_manager(suite.funds_provider.clone());

    let funds_amount = suite
        .query_auction_funds(suite.funds_provider.as_str())
        .next;
    assert_eq!(funds_amount, Uint128::zero());
}

#[test]
fn test_withdraw_active_auction() {
    let mut suite = Suite::default();
    let funds = coins(1000_u128, suite.pair.0.clone());
    suite.auction_funds(None, &funds);

    suite.start_auction(
        Some(mock_env().block.height),
        mock_env().block.height + 1000,
    );

    let funds_amount = suite.query_auction_funds(suite.funds_provider.as_str());
    assert_eq!(funds_amount.curr, funds[0].amount);
    assert_eq!(funds_amount.next, Uint128::zero());

    let err = suite.withdraw_funds_err();
    assert_eq!(err, auction::error::ContractError::NoFundsToWithdraw);

    let funds_amount = suite.query_auction_funds(suite.funds_provider.as_str());
    assert_eq!(funds_amount.curr, funds[0].amount);
    assert_eq!(funds_amount.next, Uint128::zero());
}
