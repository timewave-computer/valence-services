use cosmwasm_std::{testing::mock_env, Addr, Timestamp, Uint128};
use valence_package::services::{
    rebalancer::{RebalancerUpdateData, SystemRebalanceStatus},
    ValenceServices,
};

use crate::suite::{
    suite::{ATOM, NTRN, TRUSTEE},
    suite_builder::SuiteBuilder,
};

#[test]
fn test_remove_trustee() {
    let mut data = SuiteBuilder::get_default_rebalancer_register_data();
    data.trustee = Some(TRUSTEE.to_string());
    let mut suite = SuiteBuilder::default()
        .with_rebalancer_data(vec![data])
        .build_default();

    let config = suite
        .query_rebalancer_config(suite.account_addrs[0].clone())
        .unwrap();
    assert_eq!(config.trustee, Some(TRUSTEE.to_string()));

    suite
        .update_config(
            suite.owner.clone(),
            0,
            ValenceServices::Rebalancer,
            RebalancerUpdateData {
                trustee: Some(valence_package::helpers::OptionalField::Clear),
                base_denom: None,
                targets: vec![],
                pid: None,
                max_limit: None,
                target_override_strategy: None,
            },
        )
        .unwrap();

    let config = suite
        .query_rebalancer_config(suite.account_addrs[0].clone())
        .unwrap();
    assert_eq!(config.trustee, None);
}

#[test]
fn test_not_whitelisted_base_denom() {
    let not_whitelisted_base_denom = "not_whitelisted_base_denom".to_string();
    let mut suite = SuiteBuilder::default().build_default();

    let config = suite
        .query_rebalancer_config(suite.account_addrs[0].clone())
        .unwrap();
    assert_eq!(
        config.base_denom,
        SuiteBuilder::get_default_rebalancer_register_data().base_denom
    );

    let err: rebalancer::error::ContractError = suite
        .update_config(
            suite.owner.clone(),
            0,
            ValenceServices::Rebalancer,
            RebalancerUpdateData {
                trustee: None,
                base_denom: Some(not_whitelisted_base_denom.clone()),
                targets: vec![],
                pid: None,
                max_limit: None,
                target_override_strategy: None,
            },
        )
        .unwrap_err()
        .downcast()
        .unwrap();

    assert_eq!(
        err,
        rebalancer::error::ContractError::BaseDenomNotWhitelisted(not_whitelisted_base_denom)
    )
}

#[test]
fn test_multiple_min_balance_on_update() {
    let mut data = SuiteBuilder::get_default_rebalancer_register_data();

    // set min_balance to both targets
    data.targets[0].min_balance = Some(Uint128::new(100));
    data.targets[1].min_balance = Some(Uint128::new(100));

    let mut suite = SuiteBuilder::default().build_default();

    let err: rebalancer::error::ContractError = suite
        .update_config(
            suite.owner.clone(),
            0,
            ValenceServices::Rebalancer,
            RebalancerUpdateData {
                trustee: None,
                base_denom: None,
                targets: data.targets,
                pid: None,
                max_limit: None,
                target_override_strategy: None,
            },
        )
        .unwrap_err()
        .downcast()
        .unwrap();

    assert_eq!(
        err,
        rebalancer::error::ContractError::MultipleMinBalanceTargets
    )
}

#[test]
fn test_not_whitelisted_denom_on_update() {
    let not_whitelisted_denom = "not_whitelisted_denom".to_string();
    let mut data = SuiteBuilder::get_default_rebalancer_register_data();

    // set min_balance to both targets
    data.targets[0].denom = not_whitelisted_denom.to_string();

    let mut suite = SuiteBuilder::default().build_default();

    let err: rebalancer::error::ContractError = suite
        .update_config(
            suite.owner.clone(),
            0,
            ValenceServices::Rebalancer,
            RebalancerUpdateData {
                trustee: None,
                base_denom: None,
                targets: data.targets,
                pid: None,
                max_limit: None,
                target_override_strategy: None,
            },
        )
        .unwrap_err()
        .downcast()
        .unwrap();

    assert_eq!(
        err,
        rebalancer::error::ContractError::DenomNotWhitelisted(not_whitelisted_denom)
    )
}

#[test]
fn test_invalid_targets_perc_on_update() {
    let mut data = SuiteBuilder::get_default_rebalancer_register_data();

    // set min_balance to both targets
    data.targets[0].bps = 5000;
    data.targets[1].bps = 6000;

    let mut suite = SuiteBuilder::default().build_default();

    let err: rebalancer::error::ContractError = suite
        .update_config(
            suite.owner.clone(),
            0,
            ValenceServices::Rebalancer,
            RebalancerUpdateData {
                trustee: None,
                base_denom: None,
                targets: data.targets,
                pid: None,
                max_limit: None,
                target_override_strategy: None,
            },
        )
        .unwrap_err()
        .downcast()
        .unwrap();

    assert_eq!(
        err,
        rebalancer::error::ContractError::InvalidTargetPercentage(1.1.to_string(),)
    )
}

#[test]
fn test_not_admin_admin() {
    let mut suite = SuiteBuilder::default().build_default();

    suite.update_rebalancer_system_status_err(
        Addr::unchecked("not_admin"),
        SystemRebalanceStatus::NotStarted {
            cycle_start: Timestamp::from_nanos(0),
        },
    );
}

#[test]
fn test_update_status() {
    let mut suite = SuiteBuilder::default().build_default();

    let status = suite.query_rebalancer_system_status().unwrap();
    assert_eq!(
        status,
        SystemRebalanceStatus::NotStarted {
            cycle_start: mock_env().block.time
        }
    );

    suite
        .update_rebalancer_system_status(
            suite.admin.clone(),
            SystemRebalanceStatus::Finished {
                next_cycle: Timestamp::from_nanos(0),
            },
        )
        .unwrap();

    let status = suite.query_rebalancer_system_status().unwrap();
    assert_eq!(
        status,
        SystemRebalanceStatus::Finished {
            next_cycle: Timestamp::from_nanos(0)
        }
    );

    // try update status to processing (Should error)
    let err = suite.update_rebalancer_system_status_err(
        suite.admin.clone(),
        SystemRebalanceStatus::Processing {
            cycle_started: Timestamp::from_nanos(0),
            start_from: Addr::unchecked("random"),
            prices: vec![],
        },
    );

    assert_eq!(
        err,
        rebalancer::error::ContractError::CantUpdateStatusToProcessing
    );
}

#[test]
fn test_update_whitelist() {
    let mut suite = SuiteBuilder::default().build_default();

    let whitelist = suite.query_rebalancer_whitelists().unwrap();

    // lets make sure the whitelist is what we expect for the tests
    assert!(whitelist.denom_whitelist.contains(&ATOM.to_string()));
    assert!(whitelist.denom_whitelist.len() == 3);
    assert!(whitelist.base_denom_whitelist.contains(&ATOM.to_string()));
    assert!(whitelist.base_denom_whitelist.len() == 2);

    // remove atom, add random
    let to_add: Vec<String> = vec!["random".to_string()];
    let to_remove = vec![ATOM.to_string(), NTRN.to_string()];

    suite
        .update_rebalancer_denom_whitelist(suite.admin.clone(), to_add, to_remove)
        .unwrap();

    let whitelist = suite.query_rebalancer_whitelists().unwrap();
    assert!(!whitelist.denom_whitelist.contains(&ATOM.to_string()));
    assert!(!whitelist.denom_whitelist.contains(&NTRN.to_string()));
    assert!(whitelist.denom_whitelist.len() == 2);

    // remove atom, add random
    let to_add: Vec<String> = vec!["random".to_string()];
    let to_remove = vec![ATOM.to_string(), NTRN.to_string()];

    suite
        .update_rebalancer_base_denom_whitelist(suite.admin.clone(), to_add, to_remove)
        .unwrap();

    let whitelist = suite.query_rebalancer_whitelists().unwrap();
    assert!(!whitelist.base_denom_whitelist.contains(&ATOM.to_string()));
    assert!(!whitelist.base_denom_whitelist.contains(&NTRN.to_string()));
    assert!(whitelist.base_denom_whitelist.len() == 1);
}

#[test]
fn test_update_addrs() {
    let mut suite = SuiteBuilder::default().build_default();

    // make sure addresses are correct first
    let addrs = suite.query_rebalancer_managers().unwrap();

    assert_eq!(addrs.services, suite.manager_addr);
    assert_eq!(addrs.auctions, suite.auctions_manager_addr);

    let random_addr = Addr::unchecked("random");
    suite
        .update_rebalancer_services_manager_address(suite.admin.clone(), random_addr.clone())
        .unwrap();
    suite
        .update_rebalancer_auctions_manager_address(suite.admin.clone(), random_addr.clone())
        .unwrap();

    let addrs = suite.query_rebalancer_managers().unwrap();

    assert_eq!(addrs.services, random_addr);
    assert_eq!(addrs.auctions, random_addr);
}
