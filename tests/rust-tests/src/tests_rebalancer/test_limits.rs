use std::collections::HashSet;

use cosmwasm_std::{coin, BankMsg, Decimal, Uint128};
use cw_multi_test::Executor;
use valence_package::services::rebalancer::PID;

use crate::suite::{
    suite::{ATOM, DEFAULT_NTRN_PRICE_BPS, NTRN},
    suite_builder::SuiteBuilder,
};

#[test]
fn test_min_balance_more_than_balance_with_coins() {
    let mut config = SuiteBuilder::get_default_rebalancer_register_data();
    config.pid = PID {
        p: "0.5".to_string(),
        i: "0".to_string(),
        d: "0".to_string(),
    };
    // Set config to have min_balance for ATOM
    let mut targets = SuiteBuilder::get_default_targets();
    targets[0].min_balance = Some(2000_u128.into());

    config.targets = HashSet::from_iter(targets.iter().cloned());

    let mut suite = SuiteBuilder::default()
        .with_rebalancer_data(vec![config])
        .build_default();

    // Send some ntrn to the account
    let amount = (Decimal::bps(DEFAULT_NTRN_PRICE_BPS)
        * Decimal::from_atomics(2000_u128, 0).unwrap())
    .to_uint_floor()
        + Uint128::new(100);
    suite
        .app
        .execute(
            suite.owner.clone(),
            BankMsg::Send {
                to_address: suite.account_addrs[0].to_string(),
                amount: vec![coin(amount.u128(), NTRN)],
            }
            .into(),
        )
        .unwrap();

    for _ in 0..10 {
        suite.resolve_cycle();
    }

    let balance_atom = suite.get_balance(0, ATOM);
    // Balance should be equal or greater then our set minimum
    assert!(balance_atom.amount >= Uint128::new(2000));
    println!("{}", balance_atom.amount);
}

#[test]
fn test_min_balance() {
    let mut config = SuiteBuilder::get_default_rebalancer_register_data();
    config.pid = PID {
        p: "0.5".to_string(),
        i: "0".to_string(),
        d: "0".to_string(),
    };
    // Set config to have min_balance for ATOM
    let mut targets = SuiteBuilder::get_default_targets();
    targets[0].min_balance = Some(950_u128.into());

    config.targets = HashSet::from_iter(targets.iter().cloned());

    let mut suite = SuiteBuilder::default()
        .with_rebalancer_data(vec![config])
        .build_default();

    let old_config = suite
        .query_rebalancer_config(suite.get_account_addr(0))
        .unwrap();

    for _ in 0..10 {
        suite.resolve_cycle();
    }

    let balance_atom = suite.get_balance(0, ATOM);
    // Balance should be equal or greater then our set minimum
    assert!(balance_atom.amount < Uint128::new(1000));
    assert!(balance_atom.amount >= Uint128::new(950));

    let new_config = suite
        .query_rebalancer_config(suite.get_account_addr(0))
        .unwrap();

    new_config.targets.iter().for_each(|new_target| {
        let target = old_config
            .targets
            .iter()
            .find(|t| t.denom == new_target.denom)
            .unwrap();

        assert!(new_target.percentage == target.percentage);
    });
}

#[test]
fn test_max_limit() {
    let mut config = SuiteBuilder::get_default_rebalancer_register_data();
    config.pid = PID {
        p: "1".to_string(),
        i: "0".to_string(),
        d: "0".to_string(),
    };
    // Set config to have min_balance for ATOM
    config.max_limit_bps = Some(100); // 1%

    let mut suite = SuiteBuilder::default()
        .with_rebalancer_data(vec![config])
        .build_default();

    suite.resolve_cycle();

    // Doing 1 rebalance, should not exceed the max limit
    // our max_limit is 1%, with 1000 atom balance, we should not exceed 10 atom per sale
    let balance_atom = suite.get_balance(0, ATOM);
    // Balance should be equal or greater then our set minimum
    assert!(balance_atom.amount == Uint128::new(990));

    // Doing another rebalance should again only sell 10 atom
    suite.resolve_cycle();

    let balance_atom = suite.get_balance(0, ATOM);
    // Because of change in total value of the protpolio, our max limit is now 10.0066 atom
    // but because we are rounding, its at 11.
    assert!(balance_atom.amount == Uint128::new(979));
}
