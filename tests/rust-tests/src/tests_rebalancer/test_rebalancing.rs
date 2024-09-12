use std::{collections::HashSet, str::FromStr};

use auction_package::Pair;
use cosmwasm_std::{Decimal, Event, Uint128};

use valence_package::services::rebalancer::{Target, PID};

use crate::suite::{
    suite::{Suite, ATOM, NTRN, OSMO},
    suite_builder::SuiteBuilder,
};

fn calc_diff(balance: Uint128, target: Decimal, p: Decimal, limit: Uint128) -> Uint128 {
    let diff = (Decimal::from_atomics(balance, 0).unwrap() - target) * p;

    if diff.to_uint_ceil() < limit {
        return Uint128::zero();
    }

    diff.to_uint_ceil()
}

#[test]
fn test_basic_p_controller() {
    let mut config = SuiteBuilder::get_default_rebalancer_register_data();
    config.pid = PID {
        p: "0.5".to_string(),
        i: "0".to_string(),
        d: "0".to_string(),
    };

    let mut suite = SuiteBuilder::default()
        .with_rebalancer_data(vec![config.clone()])
        .build_default();

    let p_perc = Decimal::from_str(&config.pid.p).unwrap();
    let atom_limit = suite.get_send_min_limit(ATOM);

    // we check 4 here because on the 4th we already reach the limit
    for _ in 0..4 {
        let mut balance = suite.get_balance(0, ATOM);
        let ntrn_balance = suite.get_balance(0, NTRN);
        let price = suite.get_price(Pair::from((ATOM.to_string(), NTRN.to_string())));
        let total_value = Decimal::from_atomics(balance.amount, 0).unwrap()
            + (Decimal::from_atomics(ntrn_balance.amount, 0).unwrap() / price);
        let target = Decimal::bps(config.targets.iter().find(|t| t.denom == ATOM).unwrap().bps)
            * total_value;

        // Calcuate expected values
        let diff = calc_diff(balance.amount, target, p_perc, atom_limit);

        // println!("diff: {diff}, balance: {balance} | target: {target}: limit: {atom_limit}",);
        let expected_balance = balance.amount - diff;

        // do rebalance
        suite.resolve_cycle();

        //get new balance
        balance = suite.get_balance(0, ATOM);
        println!("balance: {balance}, expected: {expected_balance}",);

        // assert that the balance is as expected
        assert_eq!(balance.amount, expected_balance);
    }

    // Make sure that when we hit the limit of the amount we can send
    // we don't actually send any funds.
    // 5 cycles above should reach the limit
    let old_balance = suite.get_balance(0, ATOM);

    suite.resolve_cycle();

    let new_balance = suite.get_balance(0, ATOM);
    assert_eq!(new_balance.amount, old_balance.amount);
}

#[test]
fn test_min_balance_more_than_balance() {
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

    // Rebalancer should do nothing, because our init balance is 1000 atom
    // while the min_balance is 2000 atom
    let old_balance = suite.get_balance(0, ATOM);
    suite.resolve_cycle();
    let new_balance = suite.get_balance(0, ATOM);
    assert_eq!(old_balance, new_balance)
}

/// Make sure that we are not trying to send a message to the account when we don't have any trades
#[test]
fn test_no_msg_sent_when_no_trades() {
    let mut config = SuiteBuilder::get_default_rebalancer_register_data();
    // Set config to have min_balance for ATOM
    let mut targets = SuiteBuilder::get_default_targets();
    targets[0].bps = 9999;
    targets[1].bps = 1;

    config.targets = HashSet::from_iter(targets.iter().cloned());

    let mut suite = SuiteBuilder::default()
        .with_rebalancer_data(vec![config])
        .build_default();

    let res = suite.rebalance(None).unwrap();
    let has_event = res.has_event(
        &Event::new("wasm-valence-event").add_attribute("action", "account-send-funds-by-service"),
    );
    assert!(!has_event);
}

#[test]
fn test_targets_saved_after_rebalance() {
    let mut suite = Suite::default();

    let config = suite
        .query_rebalancer_config(suite.account_addrs.first().unwrap().clone())
        .unwrap();
    assert!(config.targets[0].last_input.is_none());

    suite.rebalance(None).unwrap();

    let config = suite
        .query_rebalancer_config(suite.account_addrs.first().unwrap().clone())
        .unwrap();
    assert!(config.targets[0].last_input.is_some());
}

#[test]
fn test_base_denom_not_in_target_list() {
    let mut config = SuiteBuilder::get_default_rebalancer_register_data();
    let mut new_targets = HashSet::with_capacity(2);

    new_targets.insert(Target {
        denom: ATOM.to_string(),
        bps: 7500,
        min_balance: None,
    });

    new_targets.insert(Target {
        denom: OSMO.to_string(),
        bps: 2500,
        min_balance: None,
    });

    config.base_denom = NTRN.to_string();

    config.targets = new_targets;

    // Register with the config where the base denom is whitelisted but not in target list
    let mut suite = SuiteBuilder::default()
        .with_rebalancer_data(vec![config])
        .build_default();

    // Do a rebalance just to make sure that it doesn't panic
    suite.rebalance(None).unwrap();
}
