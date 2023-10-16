use std::str::FromStr;

use cosmwasm_std::{Addr, Decimal, Timestamp};
use valence_package::{
    services::{
        rebalancer::{
            ParsedPID, ParsedTarget, RebalancerConfig, RebalancerUpdateData, Target,
            TargetOverrideStrategy, PID,
        },
        ValenceServices,
    },
    signed_decimal::SignedDecimal,
};

use crate::suite::{
    suite::{Suite, ATOM, DEFAULT_D, DEFAULT_I, DEFAULT_P, NTRN},
    suite_builder::SuiteBuilder,
};

#[test]
fn test_add_service() {
    let mut suite = SuiteBuilder::default().build_basic();

    suite
        .add_service_to_manager(
            suite.admin.clone(),
            suite.manager_addr.clone(),
            ValenceServices::Rebalancer,
            suite.rebalancer_addr.to_string(),
        )
        .unwrap();

    // test adding service with same name
    let err: services_manager::error::ContractError = suite
        .add_service_to_manager(
            suite.admin.clone(),
            suite.manager_addr.clone(),
            ValenceServices::Rebalancer,
            suite.rebalancer_addr.to_string(),
        )
        .unwrap_err()
        .downcast()
        .unwrap();

    assert_eq!(
        err,
        services_manager::error::ContractError::ServiceAlreadyExists(
            ValenceServices::Rebalancer.to_string()
        )
    );

    // test adding different service name with same address
    let err: services_manager::error::ContractError = suite
        .add_service_to_manager(
            suite.admin.clone(),
            suite.manager_addr.clone(),
            ValenceServices::Test,
            suite.rebalancer_addr.to_string(),
        )
        .unwrap_err()
        .downcast()
        .unwrap();

    assert_eq!(
        err,
        services_manager::error::ContractError::ServiceAddressAlreadyExists(
            suite.rebalancer_addr.to_string()
        )
    );
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

    // Test the default config
    suite
        .register_to_service(
            suite.owner.clone(),
            0,
            ValenceServices::Rebalancer,
            SuiteBuilder::get_default_rebalancer_register_data(),
        )
        .unwrap();
    suite.assert_rebalancer_config(
        0,
        RebalancerConfig {
            is_paused: None,
            trustee: None,
            base_denom: ATOM.to_string(),
            targets: vec![
                ParsedTarget {
                    denom: ATOM.to_string(),
                    percentage: Decimal::bps(7500),
                    min_balance: None,
                    last_input: None,
                    last_i: SignedDecimal::zero(),
                },
                ParsedTarget {
                    denom: NTRN.to_string(),
                    percentage: Decimal::bps(2500),
                    min_balance: None,
                    last_input: None,
                    last_i: SignedDecimal::zero(),
                },
            ],
            pid: ParsedPID {
                p: Decimal::from_str(DEFAULT_P).unwrap(),
                i: Decimal::from_str(DEFAULT_I).unwrap(),
                d: Decimal::from_str(DEFAULT_D).unwrap(),
            },
            max_limit: Decimal::one(),
            last_rebalance: Timestamp::from_seconds(0),
            has_min_balance: false,
            target_override_strategy: TargetOverrideStrategy::Proportional,
        },
    );

    // Test a config with changed numbers
    let mut register_data_1 = SuiteBuilder::get_default_rebalancer_register_data();
    register_data_1.trustee = Some(suite.trustee.to_string());
    register_data_1.max_limit = Some(1000);

    suite
        .register_to_service(
            suite.owner.clone(),
            1,
            ValenceServices::Rebalancer,
            register_data_1.clone(),
        )
        .unwrap();
    suite.assert_rebalancer_config(
        1,
        RebalancerConfig {
            is_paused: None,
            trustee: register_data_1.trustee,
            base_denom: ATOM.to_string(),
            targets: vec![
                ParsedTarget {
                    denom: ATOM.to_string(),
                    percentage: Decimal::bps(7500),
                    min_balance: None,
                    last_input: None,
                    last_i: SignedDecimal::zero(),
                },
                ParsedTarget {
                    denom: NTRN.to_string(),
                    percentage: Decimal::bps(2500),
                    min_balance: None,
                    last_input: None,
                    last_i: SignedDecimal::zero(),
                },
            ],
            pid: ParsedPID {
                p: Decimal::from_str(DEFAULT_P).unwrap(),
                i: Decimal::from_str(DEFAULT_I).unwrap(),
                d: Decimal::from_str(DEFAULT_D).unwrap(),
            },
            max_limit: Decimal::bps(1000),
            last_rebalance: Timestamp::from_seconds(0),
            has_min_balance: false,
            target_override_strategy: TargetOverrideStrategy::Proportional,
        },
    );

    // Try to register to a service that doesn't exists
    let err: services_manager::error::ContractError = suite
        .register_to_service(
            suite.owner.clone(),
            0,
            ValenceServices::Test,
            SuiteBuilder::get_default_rebalancer_register_data(),
        )
        .unwrap_err()
        .downcast()
        .unwrap();

    assert_eq!(
        err,
        services_manager::error::ContractError::ServiceDoesntExists(
            ValenceServices::Test.to_string()
        )
    )
}

#[test]
fn test_deregister() {
    let mut suite = Suite::default();

    suite
        .deregister_from_service(suite.owner.clone(), 0, ValenceServices::Rebalancer)
        .unwrap();

    // Confirm we no longer registered
    suite
        .query_rebalancer_config(suite.get_account_addr(0))
        .unwrap_err();
}

#[test]
fn test_pause() {
    let mut suite = SuiteBuilder::default()
        .with_accounts(3)
        .with_rebalancer_data(vec![
            SuiteBuilder::get_default_rebalancer_register_data(),
            SuiteBuilder::get_rebalancer_register_data_with_trustee(),
            SuiteBuilder::get_rebalancer_register_data_with_trustee(),
        ])
        .build_default();

    let account_addr_0 = suite.get_account_addr(0);
    let account_addr_1 = suite.get_account_addr(1);
    let account_addr_2 = suite.get_account_addr(2);

    /* Try to pause as someone random (trustee that isn't set as trustee) */
    let err: rebalancer::error::ContractError = suite
        .pause_service_with_sender(suite.trustee.clone(), 0, ValenceServices::Rebalancer)
        .unwrap_err()
        .downcast()
        .unwrap();
    assert_eq!(err, rebalancer::error::ContractError::NotAuthorizedToPause);

    suite.pause_service(0, ValenceServices::Rebalancer).unwrap();
    suite.assert_rebalancer_is_paused(0, Some(account_addr_0));

    // If account paused try to pause again, it should fail
    let err: rebalancer::error::ContractError = suite
        .pause_service_with_sender(suite.owner.clone(), 0, ValenceServices::Rebalancer)
        .unwrap_err()
        .downcast()
        .unwrap();
    assert_eq!(err, rebalancer::error::ContractError::AccountAlreadyPaused);

    /* Pause as the account owner */
    suite.pause_service(1, ValenceServices::Rebalancer).unwrap();
    suite.assert_rebalancer_is_paused(1, Some(account_addr_1));

    // Trustee can't pause after main account paused
    let err: rebalancer::error::ContractError = suite
        .pause_service_with_sender(suite.trustee.clone(), 1, ValenceServices::Rebalancer)
        .unwrap_err()
        .downcast()
        .unwrap();
    assert_eq!(err, rebalancer::error::ContractError::AccountAlreadyPaused);

    /* try pausing as trustee */
    let err: rebalancer::error::ContractError = suite
        .pause_service_with_sender(
            Addr::unchecked("random_sender"),
            2,
            ValenceServices::Rebalancer,
        )
        .unwrap_err()
        .downcast()
        .unwrap();
    assert_eq!(err, rebalancer::error::ContractError::NotAuthorizedToPause);

    suite
        .pause_service_with_sender(suite.trustee.clone(), 2, ValenceServices::Rebalancer)
        .unwrap();
    suite.assert_rebalancer_is_paused(2, Some(suite.trustee.clone()));

    // try pausing as the owner after trustee paused
    suite.pause_service(2, ValenceServices::Rebalancer).unwrap();
    suite.assert_rebalancer_is_paused(2, Some(account_addr_2));
}

#[test]
fn test_resume() {
    let mut suite = SuiteBuilder::default()
        .with_accounts(4)
        .with_rebalancer_data(vec![
            SuiteBuilder::get_default_rebalancer_register_data(),
            SuiteBuilder::get_rebalancer_register_data_with_trustee(),
            SuiteBuilder::get_rebalancer_register_data_with_trustee(),
            SuiteBuilder::get_default_rebalancer_register_data(),
        ])
        .build_default();

    /* Pause as owner, and resume as owner */
    suite.pause_service(0, ValenceServices::Rebalancer).unwrap();
    suite
        .resume_service(0, ValenceServices::Rebalancer)
        .unwrap();
    suite.assert_rebalancer_is_paused(0, None);

    /* Pause as account owner, but try to resume as trustee (can't because owner paused it) */
    suite.pause_service(1, ValenceServices::Rebalancer).unwrap();
    let err: rebalancer::error::ContractError = suite
        .resume_service_with_sender(suite.trustee.clone(), 1, ValenceServices::Rebalancer)
        .unwrap_err()
        .downcast()
        .unwrap();
    assert_eq!(err, rebalancer::error::ContractError::NotAuthorizedToResume);

    /* Pause as trustee, and resume as trustee */
    suite
        .pause_service_with_sender(suite.trustee.clone(), 2, ValenceServices::Rebalancer)
        .unwrap();
    suite
        .resume_service_with_sender(suite.trustee.clone(), 2, ValenceServices::Rebalancer)
        .unwrap();

    /* Pause as trustee but resume as owner */
    suite
        .pause_service_with_sender(suite.trustee.clone(), 2, ValenceServices::Rebalancer)
        .unwrap();
    suite
        .resume_service(2, ValenceServices::Rebalancer)
        .unwrap();

    /* Try resume with random when no trustee is assigned */
    suite.pause_service(3, ValenceServices::Rebalancer).unwrap();
    let err: rebalancer::error::ContractError = suite
        .resume_service_with_sender(
            Addr::unchecked("random_sender"),
            3,
            ValenceServices::Rebalancer,
        )
        .unwrap_err()
        .downcast()
        .unwrap();
    assert_eq!(err, rebalancer::error::ContractError::NotAuthorizedToResume);

    /* Try resume not paused */
    let err: rebalancer::error::ContractError = suite
        .resume_service(0, ValenceServices::Rebalancer)
        .unwrap_err()
        .downcast()
        .unwrap();
    assert_eq!(err, rebalancer::error::ContractError::NotPaused);
}

#[test]
fn test_update() {
    let mut suite = SuiteBuilder::default().build_default();

    suite
        .update_config(
            suite.owner.clone(),
            0,
            ValenceServices::Rebalancer,
            RebalancerUpdateData {
                trustee: Some(valence_package::helpers::OptionalField::Set(
                    "random_addr".to_string(),
                )),
                base_denom: Some(NTRN.to_string()),
                targets: vec![
                    Target {
                        denom: ATOM.to_string(),
                        percentage: 5000,
                        min_balance: None,
                    },
                    Target {
                        denom: NTRN.to_string(),
                        percentage: 5000,
                        min_balance: Some(15_u128.into()),
                    },
                ],
                pid: Some(PID {
                    p: "1".to_string(),
                    i: "0.5".to_string(),
                    d: "0.5".to_string(),
                }),
                max_limit: Some(5000),
            },
        )
        .unwrap();

    suite.assert_rebalancer_config(
        0,
        RebalancerConfig {
            is_paused: None,
            trustee: Some("random_addr".to_string()),
            base_denom: NTRN.to_string(),
            targets: vec![
                ParsedTarget {
                    denom: ATOM.to_string(),
                    percentage: Decimal::bps(5000),
                    min_balance: None,
                    last_input: None,
                    last_i: SignedDecimal::zero(),
                },
                ParsedTarget {
                    denom: NTRN.to_string(),
                    percentage: Decimal::bps(5000),
                    min_balance: Some(15_u128.into()),
                    last_input: None,
                    last_i: SignedDecimal::zero(),
                },
            ],
            pid: ParsedPID {
                p: Decimal::bps(10000),
                i: Decimal::bps(5000),
                d: Decimal::bps(5000),
            },
            max_limit: Decimal::bps(5000),
            last_rebalance: Timestamp::from_seconds(0),
            has_min_balance: true,
            target_override_strategy: TargetOverrideStrategy::Proportional,
        },
    )
}

#[test]
fn test_manager_queries() {
    let suite = SuiteBuilder::default().build_default();
    let addr = suite
        .query_service_addr_from_manager(ValenceServices::Rebalancer)
        .unwrap();
    assert_eq!(addr, suite.rebalancer_addr);

    let is_service = suite
        .query_is_service_on_manager(suite.rebalancer_addr.as_str())
        .unwrap();
    assert!(is_service);
    assert!(is_service);

    suite
        .query_service_addr_from_manager(ValenceServices::Test)
        .unwrap_err();
}

#[test]
fn test_update_service() {
    let mut suite = SuiteBuilder::default().build_default();

    /* Confirm only admin can call manager admin msgs */
    let err: services_manager::error::ContractError = suite
        .update_service_on_manager(
            Addr::unchecked("random_sender"),
            suite.manager_addr.clone(),
            ValenceServices::Rebalancer,
            "some_addr".to_string(),
        )
        .unwrap_err()
        .downcast()
        .unwrap();

    assert_eq!(err, valence_package::error::ValenceError::NotAdmin.into());

    suite
        .update_service_on_manager(
            suite.admin.clone(),
            suite.manager_addr.clone(),
            ValenceServices::Rebalancer,
            "some_addr".to_string(),
        )
        .unwrap();

    let addr = suite
        .query_service_addr_from_manager(ValenceServices::Rebalancer)
        .unwrap();
    assert_eq!(addr, "some_addr".to_string())
}

#[test]
fn test_remove_service() {
    let mut suite = SuiteBuilder::default().build_default();

    suite
        .remove_service_from_manager(
            suite.admin.clone(),
            suite.manager_addr.clone(),
            ValenceServices::Rebalancer,
        )
        .unwrap();

    let is_service = suite
        .query_is_service_on_manager(suite.rebalancer_addr.as_str())
        .unwrap();
    assert!(!is_service);
}
