use std::collections::HashMap;

use auction_package::Pair;
use cosmwasm_schema::{cw_serde, serde, QueryResponses};
use cosmwasm_std::{to_json_binary, Addr, Coin, Empty, StdError};
use cw_multi_test::{App, AppResponse, Executor};
use rebalancer::{
    contract::DEFAULT_CYCLE_PERIOD,
    msg::{ManagersAddrsResponse, WhitelistsResponse},
};
use valence_package::services::{
    rebalancer::{BaseDenom, PauseData, RebalancerConfig, ServiceFeeConfig, SystemRebalanceStatus},
    ValenceServices,
};

use super::{instantiates::AccountInstantiate, suite_builder::SuiteBuilder};

pub const ATOM: &str = "uatom";
pub const NTRN: &str = "ibc/untrn";
pub const OSMO: &str = "uosmo";

pub const ADMIN: &str = "admin";
pub const ACC_OWNER: &str = "owner";
pub const TRUSTEE: &str = "trustee";
pub const MM: &str = "market_maker";

// PID defaults
pub const DEFAULT_P: &str = "0.5";
pub const DEFAULT_I: &str = "0.005";
pub const DEFAULT_D: &str = "0.01";

pub const DEFAULT_BLOCK_TIME: u64 = 3;
pub const DAY: u64 = 86400;
pub const HALF_DAY: u64 = DAY / 2;

pub const DEFAULT_NTRN_PRICE_BPS: u64 = 15000;
pub const DEFAULT_OSMO_PRICE_BPS: u64 = 25000;

pub(crate) struct Suite {
    pub app: App,
    pub admin: Addr,
    pub owner: Addr,
    pub trustee: Addr,
    pub mm: Addr,
    pub auctions_manager_addr: Addr,
    pub oracle_addr: Addr,
    pub manager_addr: Addr,
    pub rebalancer_addr: Addr,
    pub account_addrs: Vec<Addr>,
    pub auction_addrs: HashMap<(String, String), Addr>,
    /// Used mainly for auction tests, a default pair of (ATOM, NTRN)
    pub pair: Pair,

    // code ids for future use
    pub account_code_id: u64,
}

impl Default for Suite {
    fn default() -> Self {
        SuiteBuilder::default().build_default()
    }
}

// Block helpers
impl Suite {
    pub fn update_block(&mut self, blocks: u64) -> &mut Self {
        self.app.update_block(|b| {
            b.time = b.time.plus_seconds(blocks * DEFAULT_BLOCK_TIME);
            b.height += blocks;
        });
        self
    }

    pub fn update_block_cycle(&mut self) -> &mut Self {
        self.update_block(DEFAULT_CYCLE_PERIOD / DEFAULT_BLOCK_TIME)
    }

    pub fn add_block(&mut self) -> &mut Self {
        self.update_block(1);
        self
    }
}

// account helpers
impl Suite {
    pub fn get_account_addr(&self, position: u64) -> Addr {
        self.account_addrs[position as usize].clone()
    }

    pub fn set_balance(&mut self, account_position: u64, amount: Coin) -> &mut Self {
        let account_addr = self.get_account_addr(account_position);

        self.app.init_modules(|router, _, storage| {
            router
                .bank
                .init_balance(storage, &account_addr, vec![amount])
                .unwrap();
        });
        self
    }

    pub fn add_to_balance(&mut self, account_position: u64, amount: Coin) -> &mut Self {
        let account_addr = self.get_account_addr(account_position);
        let curr_balances = self.get_all_balances(account_position);
        let update_balances = curr_balances
            .into_iter()
            .map(|mut b| {
                if b.denom == amount.denom {
                    b.amount += amount.amount
                };
                b
            })
            .collect();

        self.app.init_modules(|router, _, storage| {
            router
                .bank
                .init_balance(storage, &account_addr, update_balances)
                .unwrap();
        });
        self
    }

    pub fn create_temp_account(&mut self, balance: &[Coin]) -> (u64, Addr) {
        let account_init: valence_account::msg::InstantiateMsg =
            AccountInstantiate::new(self.manager_addr.as_str()).into();

        let account_addr = self
            .app
            .instantiate_contract(
                self.account_code_id,
                self.owner.clone(),
                &account_init,
                balance,
                "account_temp".to_string(),
                Some(self.owner.to_string()),
            )
            .unwrap();

        let position = self.account_addrs.len() as u64;
        self.account_addrs.push(account_addr.clone());

        (position, account_addr)
    }
}

// Balances
impl Suite {
    fn __query_all_balances(&self, addr: Addr) -> Vec<Coin> {
        self.app.wrap().query_all_balances(addr).unwrap()
    }

    fn __query_balance(&self, addr: Addr, denom: &str) -> Coin {
        self.app.wrap().query_balance(addr, denom).unwrap()
    }

    pub fn get_all_balances(&self, account_position: u64) -> Vec<Coin> {
        let account_addr = self.get_account_addr(account_position);
        self.__query_all_balances(account_addr)
    }

    pub fn get_balance(&self, account_position: u64, denom: &str) -> Coin {
        let account_addr = self.get_account_addr(account_position);
        self.__query_balance(account_addr, denom)
    }
}

// Rebalancer specific functions
impl Suite {
    pub fn register_to_rebalancer<D: serde::ser::Serialize>(
        &mut self,
        account_position: u64,
        register_data: &D,
    ) -> Result<AppResponse, anyhow::Error> {
        let account_addr = self.get_account_addr(account_position);
        self.app.execute_contract(
            self.owner.clone(),
            account_addr,
            &valence_package::msgs::core_execute::AccountBaseExecuteMsg::RegisterToService {
                service_name: ValenceServices::Rebalancer,
                data: Some(to_json_binary(register_data).unwrap()),
            },
            &[],
        )
    }

    pub fn register_to_rebalancer_fee_err<D: serde::ser::Serialize>(
        &mut self,
        account_position: u64,
        register_data: &D,
    ) -> StdError {
        self.register_to_rebalancer(account_position, register_data)
            .unwrap_err()
            .downcast()
            .unwrap()
    }

    pub fn register_to_rebalancer_err<D: serde::ser::Serialize>(
        &mut self,
        account_position: u64,
        register_data: &D,
    ) -> rebalancer::error::ContractError {
        self.register_to_rebalancer(account_position, register_data)
            .unwrap_err()
            .downcast()
            .unwrap()
    }

    pub fn rebalance(&mut self, limit: Option<u64>) -> Result<AppResponse, anyhow::Error> {
        self.app.execute_contract(
        Addr::unchecked("random_addr"),
        self.rebalancer_addr.clone(),
    &valence_package::services::rebalancer::RebalancerExecuteMsg::<Empty,Empty>::SystemRebalance {
          limit,
        },
        &[],
      )
    }

    pub fn rebalance_err(&mut self, limit: Option<u64>) -> rebalancer::error::ContractError {
        self.rebalance(limit).unwrap_err().downcast().unwrap()
    }

    pub fn rebalance_with_update_block(
        &mut self,
        limit: Option<u64>,
    ) -> Result<AppResponse, anyhow::Error> {
        self.update_block_cycle();

        self.rebalance(limit)
    }

    pub fn update_rebalancer_system_status(
        &mut self,
        sender: Addr,
        status: SystemRebalanceStatus,
    ) -> Result<AppResponse, anyhow::Error> {
        self.app.execute_contract(
            sender,
            self.rebalancer_addr.clone(),
            &valence_package::services::rebalancer::RebalancerExecuteMsg::<Empty, Empty>::Admin(
                valence_package::services::rebalancer::RebalancerAdminMsg::UpdateSystemStatus {
                    status,
                },
            ),
            &[],
        )
    }

    pub fn update_rebalancer_system_status_err(
        &mut self,
        sender: Addr,
        status: SystemRebalanceStatus,
    ) -> rebalancer::error::ContractError {
        self.update_rebalancer_system_status(sender, status)
            .unwrap_err()
            .downcast()
            .unwrap()
    }

    pub fn update_rebalancer_denom_whitelist(
        &mut self,
        sender: Addr,
        to_add: Vec<String>,
        to_remove: Vec<String>,
    ) -> Result<AppResponse, anyhow::Error> {
        self.app.execute_contract(
            sender,
            self.rebalancer_addr.clone(),
            &valence_package::services::rebalancer::RebalancerExecuteMsg::<Empty, Empty>::Admin(
                valence_package::services::rebalancer::RebalancerAdminMsg::UpdateDenomWhitelist {
                    to_add,
                    to_remove,
                },
            ),
            &[],
        )
    }

    pub fn update_rebalancer_base_denom_whitelist(
        &mut self,
        sender: Addr,
        to_add: Vec<BaseDenom>,
        to_remove: Vec<String>,
    ) -> Result<AppResponse, anyhow::Error> {
        self.app.execute_contract(
          sender,
          self.rebalancer_addr.clone(),
          &valence_package::services::rebalancer::RebalancerExecuteMsg::<Empty, Empty>::Admin(
              valence_package::services::rebalancer::RebalancerAdminMsg::UpdateBaseDenomWhitelist {
                  to_add,
                  to_remove,
              },
          ),
          &[]
      )
    }

    pub fn update_rebalancer_services_manager_address(
        &mut self,
        sender: Addr,
        addr: Addr,
    ) -> Result<AppResponse, anyhow::Error> {
        self.app.execute_contract(
            sender,
            self.rebalancer_addr.clone(),
            &valence_package::services::rebalancer::RebalancerExecuteMsg::<Empty, Empty>::Admin(
                valence_package::services::rebalancer::RebalancerAdminMsg::UpdateServicesManager {
                    addr: addr.to_string(),
                },
            ),
            &[],
        )
    }

    pub fn update_rebalancer_auctions_manager_address(
        &mut self,
        sender: Addr,
        addr: Addr,
    ) -> Result<AppResponse, anyhow::Error> {
        self.app.execute_contract(
            sender,
            self.rebalancer_addr.clone(),
            &valence_package::services::rebalancer::RebalancerExecuteMsg::<Empty, Empty>::Admin(
                valence_package::services::rebalancer::RebalancerAdminMsg::UpdateAuctionsManager {
                    addr: addr.to_string(),
                },
            ),
            &[],
        )
    }

    pub fn update_rebalancer_fees(
        &mut self,
        fees: ServiceFeeConfig,
    ) -> Result<AppResponse, anyhow::Error> {
        self.app.execute_contract(
            self.admin.clone(),
            self.rebalancer_addr.clone(),
            &valence_package::services::rebalancer::RebalancerExecuteMsg::<Empty, Empty>::Admin(
                valence_package::services::rebalancer::RebalancerAdminMsg::UpdateFess { fees },
            ),
            &[],
        )
    }
}

// Execute service management
impl Suite {
    pub fn add_service_to_manager(
        &mut self,
        sender: Addr,
        manager_addr: Addr,
        name: ValenceServices,
        service_addr: String,
    ) -> Result<AppResponse, anyhow::Error> {
        self.app.execute_contract(
            sender,
            manager_addr,
            &valence_package::msgs::core_execute::ServicesManagerExecuteMsg::Admin(
                valence_package::msgs::core_execute::ServicesManagerAdminMsg::AddService {
                    name,
                    addr: service_addr,
                },
            ),
            &[],
        )
    }

    pub fn update_service_on_manager(
        &mut self,
        sender: Addr,
        manager_addr: Addr,
        name: ValenceServices,
        service_addr: String,
    ) -> Result<AppResponse, anyhow::Error> {
        self.app.execute_contract(
            sender,
            manager_addr,
            &valence_package::msgs::core_execute::ServicesManagerExecuteMsg::Admin(
                valence_package::msgs::core_execute::ServicesManagerAdminMsg::UpdateService {
                    name,
                    addr: service_addr,
                },
            ),
            &[],
        )
    }

    pub fn remove_service_from_manager(
        &mut self,
        sender: Addr,
        manager_addr: Addr,
        name: ValenceServices,
    ) -> Result<AppResponse, anyhow::Error> {
        self.app.execute_contract(
            sender,
            manager_addr,
            &valence_package::msgs::core_execute::ServicesManagerExecuteMsg::Admin(
                valence_package::msgs::core_execute::ServicesManagerAdminMsg::RemoveService {
                    name,
                },
            ),
            &[],
        )
    }

    pub fn register_to_service<D: serde::ser::Serialize>(
        &mut self,
        sender: Addr,
        account_position: u64,
        service_name: ValenceServices,
        register_data: D,
    ) -> Result<AppResponse, anyhow::Error> {
        let account_addr = self.get_account_addr(account_position);
        self.app.execute_contract(
            sender,
            account_addr,
            &valence_package::msgs::core_execute::AccountBaseExecuteMsg::RegisterToService {
                service_name,
                data: Some(to_json_binary(&register_data).unwrap()),
            },
            &[],
        )
    }

    pub fn update_config<C: serde::ser::Serialize>(
        &mut self,
        sender: Addr,
        account_position: u64,
        service_name: ValenceServices,
        update_data: C,
    ) -> Result<AppResponse, anyhow::Error> {
        let account_addr = self.get_account_addr(account_position);
        self.app.execute_contract(
            sender,
            account_addr,
            &valence_package::msgs::core_execute::AccountBaseExecuteMsg::UpdateService {
                service_name,
                data: to_json_binary(&update_data).unwrap(),
            },
            &[],
        )
    }

    pub fn deregister_from_service(
        &mut self,
        sender: Addr,
        account_position: u64,
        service_name: ValenceServices,
    ) -> Result<AppResponse, anyhow::Error> {
        let account_addr = self.get_account_addr(account_position);
        self.app.execute_contract(
            sender,
            account_addr,
            &valence_package::msgs::core_execute::AccountBaseExecuteMsg::DeregisterFromService {
                service_name,
            },
            &[],
        )
    }

    pub fn pause_service(
        &mut self,
        account_position: u64,
        service_name: ValenceServices,
    ) -> Result<AppResponse, anyhow::Error> {
        let account_addr = self.get_account_addr(account_position);
        self.app.execute_contract(
            self.owner.clone(),
            account_addr,
            &valence_package::msgs::core_execute::AccountBaseExecuteMsg::PauseService {
                service_name,
                reason: Some("Some reason".to_string()),
            },
            &[],
        )
    }

    pub fn pause_service_with_sender(
        &mut self,
        sender: Addr,
        account_position: u64,
        service_name: ValenceServices,
    ) -> Result<AppResponse, anyhow::Error> {
        let account_addr = self.get_account_addr(account_position);
        self.app.execute_contract(
            sender,
            self.manager_addr.clone(),
            &valence_package::msgs::core_execute::ServicesManagerExecuteMsg::PauseService {
                service_name,
                pause_for: account_addr.to_string(),
                reason: Some("Some reason".to_string()),
            },
            &[],
        )
    }

    pub fn resume_service(
        &mut self,
        account_position: u64,
        service_name: ValenceServices,
    ) -> Result<AppResponse, anyhow::Error> {
        let account_addr = self.get_account_addr(account_position);
        self.app.execute_contract(
            self.owner.clone(),
            account_addr,
            &valence_package::msgs::core_execute::AccountBaseExecuteMsg::ResumeService {
                service_name,
            },
            &[],
        )
    }

    pub fn resume_service_err(
        &mut self,
        account_position: u64,
        service_name: ValenceServices,
    ) -> rebalancer::error::ContractError {
        self.resume_service(account_position, service_name)
            .unwrap_err()
            .downcast()
            .unwrap()
    }

    pub fn resume_service_with_sender(
        &mut self,
        sender: Addr,
        account_position: u64,
        service_name: ValenceServices,
    ) -> Result<AppResponse, anyhow::Error> {
        let account_addr = self.get_account_addr(account_position);
        self.app.execute_contract(
            sender,
            self.manager_addr.clone(),
            &valence_package::msgs::core_execute::ServicesManagerExecuteMsg::ResumeService {
                service_name,
                resume_for: account_addr.to_string(),
            },
            &[],
        )
    }

    pub fn withdraw_fees_from_manager(
        &mut self,
        denom: impl Into<String>,
    ) -> Result<AppResponse, anyhow::Error> {
        self.app.execute_contract(
            self.admin.clone(),
            self.manager_addr.clone(),
            &valence_package::msgs::core_execute::ServicesManagerExecuteMsg::Admin(
                valence_package::msgs::core_execute::ServicesManagerAdminMsg::Withdraw {
                    denom: denom.into(),
                },
            ),
            &[],
        )
    }
}

// Queries
impl Suite {
    pub fn query_rebalancer_config(&self, account: Addr) -> Result<RebalancerConfig, StdError> {
        self.app.wrap().query_wasm_smart(
            self.rebalancer_addr.clone(),
            &rebalancer::msg::QueryMsg::GetConfig {
                addr: account.to_string(),
            },
        )
    }

    pub fn query_rebalancer_paused_config(&self, account: Addr) -> Result<PauseData, StdError> {
        self.app.wrap().query_wasm_smart(
            self.rebalancer_addr.clone(),
            &rebalancer::msg::QueryMsg::GetPausedConfig {
                addr: account.to_string(),
            },
        )
    }

    pub fn query_service_addr_from_manager(
        &self,
        service: ValenceServices,
    ) -> Result<Addr, StdError> {
        self.app.wrap().query_wasm_smart(
            self.manager_addr.clone(),
            &valence_package::msgs::core_query::ServicesManagerQueryMsg::GetServiceAddr { service },
        )
    }

    pub fn query_is_service_on_manager(&self, addr: &str) -> Result<bool, StdError> {
        self.app.wrap().query_wasm_smart(
            self.manager_addr.clone(),
            &valence_package::msgs::core_query::ServicesManagerQueryMsg::IsService {
                addr: addr.to_string(),
            },
        )
    }

    pub fn query_rebalancer_system_status(&self) -> Result<SystemRebalanceStatus, StdError> {
        self.app.wrap().query_wasm_smart(
            self.rebalancer_addr.clone(),
            &rebalancer::msg::QueryMsg::GetSystemStatus {},
        )
    }

    pub fn query_rebalancer_whitelists(&self) -> Result<WhitelistsResponse, StdError> {
        self.app.wrap().query_wasm_smart(
            self.rebalancer_addr.clone(),
            &rebalancer::msg::QueryMsg::GetWhiteLists,
        )
    }

    pub fn query_rebalancer_managers(&self) -> Result<ManagersAddrsResponse, StdError> {
        self.app.wrap().query_wasm_smart(
            self.rebalancer_addr.clone(),
            &rebalancer::msg::QueryMsg::GetManagersAddrs,
        )
    }

    pub fn query_admin(&self, contract: &Addr) -> Result<Addr, StdError> {
        #[cw_serde]
        #[derive(QueryResponses)]
        enum Query {
            #[returns(Addr)]
            GetAdmin,
        }

        self.app.wrap().query_wasm_smart(contract, &Query::GetAdmin)
    }
}

// Assertions
impl Suite {
    pub fn assert_rebalancer_config(&self, account_position: u64, config: RebalancerConfig) {
        let account_addr = self.get_account_addr(account_position);
        let query_config = self.query_rebalancer_config(account_addr).unwrap();

        // Assert targets are correct
        for target in config.targets.iter() {
            assert!(query_config.targets.contains(target));
        }

        assert_eq!(query_config.trustee, config.trustee);
        assert_eq!(query_config.base_denom, config.base_denom);
        assert_eq!(query_config.pid, config.pid);
        assert_eq!(query_config.max_limit, config.max_limit);
        assert_eq!(query_config.last_rebalance, config.last_rebalance);
        assert_eq!(query_config.has_min_balance, config.has_min_balance);
        assert_eq!(
            query_config.target_override_strategy,
            config.target_override_strategy
        );
    }
}
