use cosmwasm_std::{coins, to_json_binary, Addr, BankMsg, WasmMsg};
use cw_multi_test::Executor;
use cw_utils::Expiration;

use crate::suite::suite::{Suite, ATOM};

#[test]
fn test_send_funds_by_service_not_atomic() {
    let mut suite = Suite::default();

    // message should not error, even tho we try to send non existing denom
    suite
        .app
        .execute_contract(
            suite.rebalancer_addr.clone(),
            suite.get_account_addr(0),
            &valence_package::msgs::core_execute::AccountBaseExecuteMsg::SendFundsByService {
                msgs: vec![BankMsg::Send {
                    to_address: "random".to_string(),
                    amount: coins(1_u128, "doesnt_exists".to_string()),
                }
                .into()],
                atomic: false,
            },
            &[],
        )
        .unwrap();

    // same message. but it should fail, because we want it to be atomic
    suite
        .app
        .execute_contract(
            suite.rebalancer_addr.clone(),
            suite.get_account_addr(0),
            &valence_package::msgs::core_execute::AccountBaseExecuteMsg::SendFundsByService {
                msgs: vec![BankMsg::Send {
                    to_address: "random".to_string(),
                    amount: coins(1_u128, "doesnt_exists".to_string()),
                }
                .into()],
                atomic: true,
            },
            &[],
        )
        .unwrap_err();
}

#[test]
fn test_non_funds_by_service() {
    let mut suite = Suite::default();

    // should failed as we try to execute a message that send funds
    // for a msg type that doesn't send funds
    suite
        .app
        .execute_contract(
            suite.rebalancer_addr.clone(),
            suite.get_account_addr(0),
            &valence_package::msgs::core_execute::AccountBaseExecuteMsg::ExecuteByService {
                msgs: vec![BankMsg::Send {
                    to_address: "random".to_string(),
                    amount: coins(1_u128, "doesnt_exists".to_string()),
                }
                .into()],
                atomic: false,
            },
            &[],
        )
        .unwrap_err();

    // Should pass because it should not be atomic
    suite
        .app
        .execute_contract(
            suite.rebalancer_addr.clone(),
            suite.get_account_addr(0),
            &valence_package::msgs::core_execute::AccountBaseExecuteMsg::ExecuteByService {
                msgs: vec![WasmMsg::Execute {
                    contract_addr: suite.rebalancer_addr.to_string(),
                    msg: to_json_binary(&"").unwrap(),
                    funds: vec![],
                }
                .into()],
                atomic: false,
            },
            &[],
        )
        .unwrap();

    // Same message as above, but it shouldfail  because it should be atomic
    suite
        .app
        .execute_contract(
            suite.rebalancer_addr.clone(),
            suite.get_account_addr(0),
            &valence_package::msgs::core_execute::AccountBaseExecuteMsg::ExecuteByService {
                msgs: vec![WasmMsg::Execute {
                    contract_addr: suite.rebalancer_addr.to_string(),
                    msg: to_json_binary(&"").unwrap(),
                    funds: vec![],
                }
                .into()],
                atomic: true,
            },
            &[],
        )
        .unwrap_err();
}

#[test]
fn test_only_admin() {
    let mut suite = Suite::default();

    // Admin, should pass
    suite
        .app
        .execute_contract(
            suite.owner.clone(),
            suite.get_account_addr(0),
            &valence_package::msgs::core_execute::AccountBaseExecuteMsg::ExecuteByAdmin {
                msgs: vec![BankMsg::Send {
                    to_address: suite.rebalancer_addr.to_string(),
                    amount: coins(1_u128, ATOM.to_string()),
                }
                .into()],
            },
            &[],
        )
        .unwrap();

    // NOT admin, should fail
    suite
        .app
        .execute_contract(
            suite.rebalancer_addr.clone(),
            suite.get_account_addr(0),
            &valence_package::msgs::core_execute::AccountBaseExecuteMsg::ExecuteByAdmin {
                msgs: vec![],
            },
            &[],
        )
        .unwrap_err();
}

#[test]
fn test_update_admin_start() {
    let mut suite = Suite::default();
    let new_admin = Addr::unchecked("random_addr");
    let account_addr = suite.get_account_addr(0);

    // Try to approve admin without starting a new change
    // should error
    suite
        .app
        .execute_contract(
            new_admin.clone(),
            account_addr.clone(),
            &price_oracle::msg::ExecuteMsg::ApproveAdminChange,
            &[],
        )
        .unwrap_err();

    suite
        .app
        .execute_contract(
            suite.owner.clone(),
            account_addr.clone(),
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
            account_addr.clone(),
            &price_oracle::msg::ExecuteMsg::ApproveAdminChange,
            &[],
        )
        .unwrap();

    let admin = suite.query_admin(&account_addr).unwrap();
    assert_eq!(admin, new_admin)
}

#[test]
fn test_update_admin_cancel() {
    let mut suite = Suite::default();
    let new_admin = Addr::unchecked("new_admin_addr");
    let account_addr = suite.get_account_addr(0);

    suite
        .app
        .execute_contract(
            suite.owner.clone(),
            account_addr.clone(),
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
            suite.owner.clone(),
            account_addr.clone(),
            &price_oracle::msg::ExecuteMsg::CancelAdminChange,
            &[],
        )
        .unwrap();

    // Should error because we cancelled the admin change
    suite
        .app
        .execute_contract(
            new_admin,
            account_addr,
            &price_oracle::msg::ExecuteMsg::ApproveAdminChange,
            &[],
        )
        .unwrap_err();
}

#[test]
fn test_update_admin_fails() {
    let mut suite = Suite::default();
    let new_admin = Addr::unchecked("new_admin_addr");
    let random_addr = Addr::unchecked("random_addr");
    let account_addr = suite.get_account_addr(0);

    suite
        .app
        .execute_contract(
            suite.owner.clone(),
            account_addr.clone(),
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
            account_addr.clone(),
            &price_oracle::msg::ExecuteMsg::ApproveAdminChange,
            &[],
        )
        .unwrap_err();

    suite.update_block_cycle();

    // Should fail because expired
    suite
        .app
        .execute_contract(
            new_admin,
            account_addr,
            &price_oracle::msg::ExecuteMsg::ApproveAdminChange,
            &[],
        )
        .unwrap_err();
}
