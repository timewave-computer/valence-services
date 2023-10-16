use auction_package::Pair;
use cosmwasm_std::{testing::mock_env, BlockInfo, Decimal, Timestamp, Uint128};
use rebalancer::{contract::CYCLE_PERIOD, state::SystemRebalanceStatus};
use valence_package::{
    helpers::start_of_day,
    services::{rebalancer::Target, ValenceServices},
};

use crate::suite::{
    suite::{ATOM, DEFAULT_BLOCK_TIME, DEFAULT_NTRN_PRICE_BPS, DEFAULT_OSMO_PRICE_BPS, NTRN, OSMO},
    suite_builder::SuiteBuilder,
};

#[test]
fn test_rebalancer_system() {
    let mut suite = SuiteBuilder::default().with_accounts(2).build_default();

    // First confirm status is not started
    let status = suite.query_rebalancer_system_status().unwrap();
    assert_eq!(
        status,
        SystemRebalanceStatus::NotStarted {
            cycle_start: mock_env().block.time
        }
    );

    // Do a rebalance with limit of 1 to go into the processing status
    suite.rebalance_with_update_block(Some(1)).unwrap();

    // check status matches what we expect
    let status = suite.query_rebalancer_system_status().unwrap();
    assert_eq!(
        status,
        SystemRebalanceStatus::Processing {
            cycle_started: start_of_day(suite.app.block_info().time),
            start_from: suite.get_account_addr(0),
            prices: vec![
                (
                    Pair::from((ATOM.to_string(), NTRN.to_string())),
                    Decimal::bps(DEFAULT_NTRN_PRICE_BPS)
                ),
                (
                    Pair::from((ATOM.to_string(), OSMO.to_string())),
                    Decimal::bps(DEFAULT_OSMO_PRICE_BPS)
                ),
                (
                    Pair::from((NTRN.to_string(), ATOM.to_string())),
                    Decimal::one() / Decimal::bps(DEFAULT_NTRN_PRICE_BPS)
                ),
                (
                    Pair::from((NTRN.to_string(), OSMO.to_string())),
                    Decimal::bps(DEFAULT_OSMO_PRICE_BPS) / Decimal::bps(DEFAULT_NTRN_PRICE_BPS)
                )
            ]
        }
    );

    // confirm our rebalancer ran over the first account only
    let config_1 = suite
        .query_rebalancer_config(suite.get_account_addr(0))
        .unwrap();
    let config_2 = suite
        .query_rebalancer_config(suite.get_account_addr(1))
        .unwrap();

    assert_eq!(config_1.last_rebalance, suite.app.block_info().time);
    assert_eq!(config_2.last_rebalance, Timestamp::from_seconds(0));

    // Do a final Rebalance
    suite.add_block();
    suite.rebalance(Some(2)).unwrap();

    // Check status is Finished
    let status = suite.query_rebalancer_system_status().unwrap();
    assert_eq!(
        status,
        SystemRebalanceStatus::Finished {
            next_cycle: start_of_day(suite.app.block_info().time).plus_seconds(CYCLE_PERIOD)
        }
    );

    let config_1 = suite
        .query_rebalancer_config(suite.get_account_addr(0))
        .unwrap();
    let config_2 = suite
        .query_rebalancer_config(suite.get_account_addr(1))
        .unwrap();

    assert_eq!(
        config_1.last_rebalance,
        suite
            .app
            .block_info()
            .time
            .minus_seconds(DEFAULT_BLOCK_TIME)
    );
    assert_eq!(config_2.last_rebalance, suite.app.block_info().time);

    // Try to do another rebalance today, should fail
    suite.add_block();
    let err: rebalancer::error::ContractError =
        suite.rebalance(Some(2)).unwrap_err().downcast().unwrap();

    assert_eq!(
        err,
        rebalancer::error::ContractError::CycleNotStartedYet(
            start_of_day(suite.app.block_info().time.plus_seconds(CYCLE_PERIOD)).seconds()
        )
    )
}

#[test]
fn test_register() {
    let mut suite = SuiteBuilder::default().with_accounts(2).build_basic();

    // Because we have a basic setup here, we need to register the service to the manager
    suite
        .add_service_to_manager(
            suite.admin.clone(),
            suite.manager_addr.clone(),
            ValenceServices::Rebalancer,
            suite.rebalancer_addr.to_string(),
        )
        .unwrap();

    suite
        .register_to_rebalancer(0, &SuiteBuilder::get_default_rebalancer_register_data())
        .unwrap();

    // Try to register when already registered
    let err =
        suite.register_to_rebalancer_err(0, &SuiteBuilder::get_default_rebalancer_register_data());
    assert_eq!(
        err,
        rebalancer::error::ContractError::AccountAlreadyRegistered
    );

    // Try register with not whitelisted base denom
    let mut register_data = SuiteBuilder::get_default_rebalancer_register_data();
    register_data.base_denom = "not_whitelisted_denom".to_string();

    let err = suite.register_to_rebalancer_err(1, &register_data);
    assert_eq!(
        err,
        rebalancer::error::ContractError::BaseDenomNotWhitelisted(
            "not_whitelisted_denom".to_string()
        )
    );

    // Try to register with only 1 target
    register_data = SuiteBuilder::get_default_rebalancer_register_data();
    register_data.targets = vec![Target {
        denom: ATOM.to_string(),
        percentage: 10000,
        min_balance: None,
    }];

    let err = suite.register_to_rebalancer_err(1, &register_data);
    assert_eq!(err, rebalancer::error::ContractError::TwoTargetsMinimum);

    // Try to register with not whitelisted denom
    register_data = SuiteBuilder::get_default_rebalancer_register_data();
    register_data.targets = vec![
        Target {
            denom: ATOM.to_string(),
            percentage: 5000,
            min_balance: None,
        },
        Target {
            denom: "not_whitelisted_denom".to_string(),
            percentage: 5000,
            min_balance: None,
        },
    ];

    let err = suite.register_to_rebalancer_err(1, &register_data);
    assert_eq!(
        err,
        rebalancer::error::ContractError::DenomNotWhitelisted("not_whitelisted_denom".to_string())
    );

    // Try to register with wrong total percentage (must equal 10000)
    register_data = SuiteBuilder::get_default_rebalancer_register_data();
    register_data.targets = vec![
        Target {
            denom: ATOM.to_string(),
            percentage: 6000,
            min_balance: None,
        },
        Target {
            denom: NTRN.to_string(),
            percentage: 5000,
            min_balance: None,
        },
    ];

    let err = suite.register_to_rebalancer_err(1, &register_data);
    assert_eq!(
        err,
        rebalancer::error::ContractError::InvalidTargetPercentage("1.1".to_string())
    );
}

#[test]
fn test_dup_targets() {
    let mut suite = SuiteBuilder::default().with_accounts(1).build_basic();

    suite
        .add_service_to_manager(
            suite.admin.clone(),
            suite.manager_addr.clone(),
            ValenceServices::Rebalancer,
            suite.rebalancer_addr.to_string(),
        )
        .unwrap();

    let mut register_data = SuiteBuilder::get_default_rebalancer_register_data();
    register_data.targets.push(register_data.targets[0].clone());

    let err = suite.register_to_rebalancer_err(0, &register_data);
    assert_eq!(err, rebalancer::error::ContractError::TargetsMustBeUnique);
}

#[test]
fn test_set_2_min_balance() {
    let mut suite = SuiteBuilder::default().build_basic();

    suite
        .add_service_to_manager(
            suite.admin.clone(),
            suite.manager_addr.clone(),
            ValenceServices::Rebalancer,
            suite.rebalancer_addr.to_string(),
        )
        .unwrap();

    let mut register_data = SuiteBuilder::get_default_rebalancer_register_data();
    register_data.targets.push(register_data.targets[0].clone());

    // set both to have min_balance
    register_data.targets[0].min_balance = Some(Uint128::new(100));
    register_data.targets[1].min_balance = Some(Uint128::new(100));

    let err = suite.register_to_rebalancer_err(0, &register_data);
    assert_eq!(
        err,
        rebalancer::error::ContractError::MultipleMinBalanceTargets
    );
}

#[test]
fn test_no_balance() {
    let mut suite = SuiteBuilder::default().build_default();
    let acc_addr = suite.get_account_addr(0);
    suite.app.init_modules(|router, _, storage| {
        router
            .bank
            .init_balance(storage, &acc_addr, vec![])
            .unwrap();
    });

    // Should not error and basically do nothing.
    suite.resolve_cycle();
}

#[test]
fn test_rebalancer_cycle_before() {
    let mut suite = SuiteBuilder::default().build_default();

    // Set block to before mock_env.
    suite.app.set_block(BlockInfo {
        height: mock_env().block.height - 2,
        time: mock_env().block.time.minus_seconds(12),
        chain_id: "".to_string(),
    });

    let err: rebalancer::error::ContractError =
        suite.rebalance(None).unwrap_err().downcast().unwrap();
    assert_eq!(
        err,
        rebalancer::error::ContractError::CycleNotStartedYet(mock_env().block.time.seconds())
    )
}

#[test]
fn test_rebalancer_cycle_next_day_while_processing() {
    let mut suite = SuiteBuilder::default().with_accounts(2).build_default();

    // Do only rebalance for 1 account
    suite.rebalance_with_update_block(Some(1)).unwrap();

    // Make sure last rebalance for 2nd account is 0
    let config = suite
        .query_rebalancer_config(suite.get_account_addr(1))
        .unwrap();
    assert!(config.last_rebalance.seconds() == 0);

    // Do another rebalance for both accounts now.
    suite.rebalance_with_update_block(None).unwrap();

    let config1 = suite
        .query_rebalancer_config(suite.get_account_addr(1))
        .unwrap();
    let config2 = suite
        .query_rebalancer_config(suite.get_account_addr(1))
        .unwrap();
    assert_eq!(config1.last_rebalance, suite.app.block_info().time);
    assert_eq!(config2.last_rebalance, suite.app.block_info().time);
}
