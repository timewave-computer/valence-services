use std::str::FromStr;

use auction_package::Pair;
use cosmwasm_std::{Decimal, Uint128};

use valence_package::services::rebalancer::PID;

use crate::suite::{
    suite::{ATOM, NTRN},
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
    let atom_limit = suite.get_min_limit(ATOM);

    // we check 5 here because on the 7th we already reach the limit
    for _ in 0..5 {
        let mut balance = suite.get_balance(0, ATOM);
        let ntrn_balance = suite.get_balance(0, NTRN);
        let price = suite.get_price(Pair::from((ATOM.to_string(), NTRN.to_string())));
        let total_value = Decimal::from_atomics(balance.amount, 0).unwrap()
            + (Decimal::from_atomics(ntrn_balance.amount, 0).unwrap() / price);
        let target = Decimal::bps(config.targets[0].percentage) * total_value;

        // Calcuate expected values
        let diff = calc_diff(balance.amount, target, p_perc, atom_limit);

        println!("diff: {diff}, balance: {balance} | target: {target}: limit: {atom_limit}",);
        let expected_balance = balance.amount - diff;

        // do rebalance
        suite.resolve_cycle();

        //get new balance
        balance = suite.get_balance(0, ATOM);
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
    config.targets[0].min_balance = Some(Uint128::new(2000));

    let mut suite = SuiteBuilder::default()
        .with_rebalancer_data(vec![config.clone()])
        .build_default();

    // Rebalancer should do nothing, because our init balance is 1000 atom
    // while the min_balance is 2000 atom
    let old_balance = suite.get_balance(0, ATOM);
    suite.resolve_cycle();
    let new_balance = suite.get_balance(0, ATOM);
    assert_eq!(old_balance, new_balance)
}
