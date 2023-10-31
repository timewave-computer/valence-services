use cosmwasm_std::Uint128;
use valence_package::services::{rebalancer::RebalancerUpdateData, ValenceServices};

use crate::suite::{suite::TRUSTEE, suite_builder::SuiteBuilder};

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
    data.targets[0].percentage = 5000;
    data.targets[1].percentage = 6000;

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
