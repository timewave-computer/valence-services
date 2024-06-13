use std::str::FromStr;

use cosmwasm_std::{testing::mock_dependencies, Decimal, Uint128};
use rebalancer::{helpers::TargetHelper, rebalance::verify_targets};
use valence_package::{
    services::rebalancer::{ParsedTarget, TargetOverrideStrategy},
    signed_decimal::SignedDecimal,
};

use crate::suite::{
    suite::{ATOM, NTRN, OSMO},
    suite_builder::SuiteBuilder,
};

#[test]
fn test_verify_target_2_denoms() {
    let deps = mock_dependencies();
    let config = SuiteBuilder::get_default_rebalancer_register_data()
        .to_config(&deps.api)
        .unwrap();
    let mut target_helpers = vec![
        TargetHelper {
            target: ParsedTarget {
                denom: ATOM.to_string(),
                percentage: Decimal::bps(7500),
                min_balance: Some(40_u128.into()),
                last_input: None,
                last_i: SignedDecimal::zero(),
            },
            price: Decimal::from_str("1").unwrap(),
            balance_amount: Uint128::from_str("100").unwrap(),
            balance_value: Decimal::from_str("100").unwrap(),
            value_to_trade: Decimal::zero(),
            auction_min_amount: Decimal::zero(),
        },
        TargetHelper {
            target: ParsedTarget {
                denom: NTRN.to_string(),
                percentage: Decimal::bps(2500),
                min_balance: None,
                last_input: None,
                last_i: SignedDecimal::zero(),
            },
            price: Decimal::from_str("0.1").unwrap(),
            balance_amount: Uint128::zero(),
            balance_value: Decimal::zero(),
            value_to_trade: Decimal::zero(),
            auction_min_amount: Decimal::zero(),
        },
    ];

    let res = verify_targets(
        &config,
        Decimal::from_str("100").unwrap(),
        target_helpers.clone(),
    )
    .unwrap();
    assert_eq!(res[0].target.percentage, Decimal::bps(7500));

    // Change min_balance to 75, should still mean our target is 75%
    target_helpers[0].target.min_balance = Some(75_u128.into());

    let res = verify_targets(
        &config,
        Decimal::from_str("100").unwrap(),
        target_helpers.clone(),
    )
    .unwrap();
    assert_eq!(res[0].target.percentage, Decimal::bps(7500));

    // Change min_Balance to 80, should change our target perc to 80% and ntrn to 20%
    target_helpers[0].target.min_balance = Some(80_u128.into());

    let res = verify_targets(
        &config,
        Decimal::from_str("100").unwrap(),
        target_helpers.clone(),
    )
    .unwrap();

    assert_eq!(res[0].target.percentage, Decimal::bps(8000));
    assert_eq!(res[1].target.percentage, Decimal::bps(2000));

    // Change min_Balance to 80, should change our target perc to 80% and ntrn to 20%
    target_helpers[0].target.min_balance = Some(90_u128.into());

    let res = verify_targets(
        &config,
        Decimal::from_str("100").unwrap(),
        target_helpers.clone(),
    )
    .unwrap();

    assert_eq!(res[0].target.percentage, Decimal::bps(9000));
    assert_eq!(res[1].target.percentage, Decimal::bps(1000));
}

#[test]
fn test_verify_target_3_denoms() {
    let deps = mock_dependencies();
    let config = SuiteBuilder::get_default_rebalancer_register_data()
        .to_config(&deps.api)
        .unwrap();
    let mut target_helpers = vec![
        TargetHelper {
            target: ParsedTarget {
                denom: ATOM.to_string(),
                percentage: Decimal::bps(5000),
                min_balance: Some(40_u128.into()),
                last_input: None,
                last_i: SignedDecimal::zero(),
            },
            price: Decimal::from_str("1").unwrap(),
            balance_amount: Uint128::new(100),
            balance_value: Decimal::from_str("100").unwrap(),
            value_to_trade: Decimal::zero(),
            auction_min_amount: Decimal::zero(),
        },
        TargetHelper {
            target: ParsedTarget {
                denom: NTRN.to_string(),
                percentage: Decimal::bps(2500),
                min_balance: None,
                last_input: None,
                last_i: SignedDecimal::zero(),
            },
            price: Decimal::from_str("0.5").unwrap(),
            balance_amount: Uint128::zero(),
            balance_value: Decimal::zero(),
            value_to_trade: Decimal::zero(),
            auction_min_amount: Decimal::zero(),
        },
        TargetHelper {
            target: ParsedTarget {
                denom: OSMO.to_string(),
                percentage: Decimal::bps(2500),
                min_balance: None,
                last_input: None,
                last_i: SignedDecimal::zero(),
            },
            price: Decimal::from_str("0.1").unwrap(),
            balance_amount: Uint128::zero(),
            balance_value: Decimal::zero(),
            value_to_trade: Decimal::zero(),
            auction_min_amount: Decimal::zero(),
        },
    ];

    let res = verify_targets(
        &config,
        Decimal::from_str("100").unwrap(),
        target_helpers.clone(),
    )
    .unwrap();
    assert_eq!(res[0].target.percentage, Decimal::bps(5000));

    // Change min_balance to 50, should keep our targets
    target_helpers[0].target.min_balance = Some(50_u128.into());

    let res = verify_targets(
        &config,
        Decimal::from_str("100").unwrap(),
        target_helpers.clone(),
    )
    .unwrap();

    assert_eq!(res[0].target.percentage, Decimal::bps(5000));
    assert_eq!(res[1].target.percentage, Decimal::bps(2500));
    assert_eq!(res[2].target.percentage, Decimal::bps(2500));

    // Change min_balance to 60, should change are targets to 20% each (proportaional)
    target_helpers[0].target.min_balance = Some(60_u128.into());

    let res = verify_targets(
        &config,
        Decimal::from_str("100").unwrap(),
        target_helpers.clone(),
    )
    .unwrap();

    assert_eq!(res[0].target.percentage, Decimal::bps(6000));
    assert_eq!(res[1].target.percentage, Decimal::bps(2000));
    assert_eq!(res[2].target.percentage, Decimal::bps(2000));

    // Change min_balance to 80, should change are targets to 10% each (proportaional)
    target_helpers[0].target.min_balance = Some(80_u128.into());

    let res = verify_targets(
        &config,
        Decimal::from_str("100").unwrap(),
        target_helpers.clone(),
    )
    .unwrap();

    assert_eq!(res[0].target.percentage, Decimal::bps(8000));
    assert_eq!(res[1].target.percentage, Decimal::bps(1000));
    assert_eq!(res[2].target.percentage, Decimal::bps(1000));

    // Change min_balance to 60, and targets to 40% and 10%, should change are targets to 20% each (proportaional)
    target_helpers[0].target.min_balance = Some(60_u128.into());
    target_helpers[1].target.percentage = Decimal::bps(4000);
    target_helpers[2].target.percentage = Decimal::bps(1000);

    let res = verify_targets(&config, Decimal::from_str("100").unwrap(), target_helpers).unwrap();

    assert_eq!(res[0].target.percentage, Decimal::bps(6000));
    assert_eq!(res[1].target.percentage, Decimal::bps(3200));
    assert_eq!(res[2].target.percentage, Decimal::bps(800));
}

#[test]
fn test_verify_target_leftover_strategy() {
    let deps = mock_dependencies();
    let mut config = SuiteBuilder::get_default_rebalancer_register_data()
        .to_config(&deps.api)
        .unwrap();
    let mut target_helpers = vec![
        TargetHelper {
            target: ParsedTarget {
                denom: ATOM.to_string(),
                percentage: Decimal::bps(5000),
                min_balance: Some(40_u128.into()),
                last_input: None,
                last_i: SignedDecimal::zero(),
            },
            price: Decimal::from_str("1").unwrap(),
            balance_amount: Uint128::new(100),
            balance_value: Decimal::from_str("100").unwrap(),
            value_to_trade: Decimal::zero(),
            auction_min_amount: Decimal::zero(),
        },
        TargetHelper {
            target: ParsedTarget {
                denom: NTRN.to_string(),
                percentage: Decimal::bps(2500),
                min_balance: None,
                last_input: None,
                last_i: SignedDecimal::zero(),
            },
            price: Decimal::from_str("0.5").unwrap(),
            balance_amount: Uint128::zero(),
            balance_value: Decimal::zero(),
            value_to_trade: Decimal::zero(),
            auction_min_amount: Decimal::zero(),
        },
        TargetHelper {
            target: ParsedTarget {
                denom: OSMO.to_string(),
                percentage: Decimal::bps(2500),
                min_balance: None,
                last_input: None,
                last_i: SignedDecimal::zero(),
            },
            price: Decimal::from_str("0.1").unwrap(),
            balance_amount: Uint128::zero(),
            balance_value: Decimal::zero(),
            value_to_trade: Decimal::zero(),
            auction_min_amount: Decimal::zero(),
        },
    ];

    let res = verify_targets(
        &config,
        Decimal::from_str("100").unwrap(),
        target_helpers.clone(),
    )
    .unwrap();
    assert_eq!(res[0].target.percentage, Decimal::bps(5000));
    assert_eq!(res[1].target.percentage, Decimal::bps(2500));
    assert_eq!(res[2].target.percentage, Decimal::bps(2500));

    // Change min_balance to 63 and perc to 29 and 21
    target_helpers[0].target.min_balance = Some(63_u128.into());
    target_helpers[1].target.percentage = Decimal::bps(2900);
    target_helpers[2].target.percentage = Decimal::bps(2100);

    let res = verify_targets(
        &config,
        Decimal::from_str("100").unwrap(),
        target_helpers.clone(),
    )
    .unwrap();

    assert_eq!(res[0].target.percentage, Decimal::bps(6300));
    assert_eq!(res[1].target.percentage, Decimal::bps(2146));
    assert_eq!(res[2].target.percentage, Decimal::bps(1554));

    // Change strategy to priority, and set perc to 21 and 29
    config.target_override_strategy = TargetOverrideStrategy::Priority;
    target_helpers[1].target.percentage = Decimal::bps(2100);
    target_helpers[2].target.percentage = Decimal::bps(2900);

    let res = verify_targets(
        &config,
        Decimal::from_str("100").unwrap(),
        target_helpers.clone(),
    )
    .unwrap();

    assert_eq!(res[0].target.percentage, Decimal::bps(6300));
    assert_eq!(res[1].target.percentage, Decimal::bps(2100));
    assert_eq!(res[2].target.percentage, Decimal::bps(1600));

    // Change min_balance to 80 and perc to 25 and 25
    target_helpers[0].target.min_balance = Some(80_u128.into());
    target_helpers[1].target.percentage = Decimal::bps(2500);
    target_helpers[2].target.percentage = Decimal::bps(2500);

    let res = verify_targets(
        &config,
        Decimal::from_str("100").unwrap(),
        target_helpers.clone(),
    )
    .unwrap();

    assert_eq!(res[0].target.percentage, Decimal::bps(8000));
    assert_eq!(res[1].target.percentage, Decimal::bps(2000));
    assert_eq!(res[2].target.percentage, Decimal::bps(0));
}

#[test]
fn test_verify_target_min_balance_over_balance() {
    let deps = mock_dependencies();
    let config = SuiteBuilder::get_default_rebalancer_register_data()
        .to_config(&deps.api)
        .unwrap();
    let target_helpers = vec![
        TargetHelper {
            target: ParsedTarget {
                denom: ATOM.to_string(),
                percentage: Decimal::bps(5000),
                min_balance: Some(120_u128.into()),
                last_input: None,
                last_i: SignedDecimal::zero(),
            },
            price: Decimal::from_str("1").unwrap(),
            balance_amount: Uint128::new(100),
            balance_value: Decimal::from_str("100").unwrap(),
            value_to_trade: Decimal::zero(),
            auction_min_amount: Decimal::zero(),
        },
        TargetHelper {
            target: ParsedTarget {
                denom: NTRN.to_string(),
                percentage: Decimal::bps(2500),
                min_balance: None,
                last_input: None,
                last_i: SignedDecimal::zero(),
            },
            price: Decimal::from_str("0.5").unwrap(),
            balance_amount: Uint128::zero(),
            balance_value: Decimal::zero(),
            value_to_trade: Decimal::zero(),
            auction_min_amount: Decimal::zero(),
        },
        TargetHelper {
            target: ParsedTarget {
                denom: OSMO.to_string(),
                percentage: Decimal::bps(2500),
                min_balance: None,
                last_input: None,
                last_i: SignedDecimal::zero(),
            },
            price: Decimal::from_str("0.1").unwrap(),
            balance_amount: Uint128::zero(),
            balance_value: Decimal::zero(),
            value_to_trade: Decimal::zero(),
            auction_min_amount: Decimal::zero(),
        },
    ];

    let res = verify_targets(&config, Decimal::from_str("100").unwrap(), target_helpers).unwrap();
    assert_eq!(res[0].target.percentage, Decimal::bps(10000));
    assert_eq!(res[1].target.percentage, Decimal::bps(0));
    assert_eq!(res[2].target.percentage, Decimal::bps(0));
}

// We have 100 ATOM so our total value is 100
// we
#[test]
fn test_verify_target_priority() {
    let deps = mock_dependencies();
    let mut config = SuiteBuilder::get_default_rebalancer_register_data()
        .to_config(&deps.api)
        .unwrap();
    let mut target_helpers = vec![
        TargetHelper {
            target: ParsedTarget {
                denom: ATOM.to_string(),
                percentage: Decimal::bps(5000),
                min_balance: None,
                last_input: None,
                last_i: SignedDecimal::zero(),
            },
            price: Decimal::from_str("1").unwrap(),
            balance_amount: Uint128::new(100),
            balance_value: Decimal::from_str("100").unwrap(),
            value_to_trade: Decimal::zero(),
            auction_min_amount: Decimal::zero(),
        },
        TargetHelper {
            target: ParsedTarget {
                denom: NTRN.to_string(),
                percentage: Decimal::bps(2500),
                min_balance: Some(25_u128.into()),
                last_input: None,
                last_i: SignedDecimal::zero(),
            },
            price: Decimal::from_str("0.5").unwrap(),
            balance_amount: Uint128::zero(),
            balance_value: Decimal::zero(),
            value_to_trade: Decimal::zero(),
            auction_min_amount: Decimal::zero(),
        },
        TargetHelper {
            target: ParsedTarget {
                denom: OSMO.to_string(),
                percentage: Decimal::bps(2500),
                min_balance: None,
                last_input: None,
                last_i: SignedDecimal::zero(),
            },
            price: Decimal::from_str("0.1").unwrap(),
            balance_amount: Uint128::zero(),
            balance_value: Decimal::zero(),
            value_to_trade: Decimal::zero(),
            auction_min_amount: Decimal::zero(),
        },
    ];

    // Prop
    let res = verify_targets(
        &config,
        Decimal::from_str("100").unwrap(),
        target_helpers.clone(),
    )
    .unwrap();

    assert_eq!(
        res[0].target.percentage,
        Decimal::from_str("0.333333333333333333").unwrap()
    );
    assert_eq!(res[1].target.percentage, Decimal::bps(5000));
    assert_eq!(
        res[2].target.percentage,
        Decimal::from_str("0.166666666666666666").unwrap()
    );

    // Priority
    config.target_override_strategy = TargetOverrideStrategy::Priority;

    // our total value is 100
    // min_balance is 25 (price is 0.5, which means 25 / 0.5 = 50 value)
    // so we expect the result to be 50% of target 1 and 50% target 2 (target 3 is 0)
    let res = verify_targets(
        &config,
        Decimal::from_str("100").unwrap(),
        target_helpers.clone(),
    )
    .unwrap();

    assert_eq!(res[0].target.percentage, Decimal::bps(5000));
    assert_eq!(res[1].target.percentage, Decimal::bps(5000));
    assert_eq!(res[2].target.percentage, Decimal::bps(0));

    // total value is still 100
    // we set min_balance to be 20 meaning (price is 0.5, which means 25 / 0.5 = 50 value)
    target_helpers[1].target.min_balance = Some(20_u128.into());

    let res = verify_targets(
        &config,
        Decimal::from_str("100").unwrap(),
        target_helpers.clone(),
    )
    .unwrap();

    assert_eq!(res[0].target.percentage, Decimal::bps(5000));
    assert_eq!(res[1].target.percentage, Decimal::bps(4000));
    assert_eq!(res[2].target.percentage, Decimal::bps(1000));

    target_helpers[1].target.min_balance = Some(400_u128.into());

    let res = verify_targets(
        &config,
        Decimal::from_str("100").unwrap(),
        target_helpers.clone(),
    )
    .unwrap();

    println!("{:?}", res);

    assert_eq!(res[0].target.percentage, Decimal::bps(5000));
    assert_eq!(res[1].target.percentage, Decimal::bps(1000));
    assert_eq!(res[2].target.percentage, Decimal::bps(4000));
}
