use cosmwasm_std::{coins, to_binary, BankMsg, WasmMsg};
use cw_multi_test::Executor;

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
                    msg: to_binary(&"").unwrap(),
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
                    msg: to_binary(&"").unwrap(),
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
