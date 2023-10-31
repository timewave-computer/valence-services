use std::collections::HashMap;

use auction::msg::NewAuctionParams;
use auction_package::{helpers::GetPriceResponse, Pair};
use cosmwasm_schema::serde;
use cosmwasm_std::{coin, to_binary, Addr, Coin, Decimal, Empty, StdError, Uint128};
use cw_multi_test::{App, AppResponse, Executor};
use rebalancer::{contract::CYCLE_PERIOD, state::SystemRebalanceStatus};
use valence_package::{
    services::{rebalancer::RebalancerConfig, ValenceServices},
    signed_decimal::SignedDecimal,
};

use super::suite_builder::SuiteBuilder;

pub const ATOM: &str = "uatom";
pub const NTRN: &str = "untrn";
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
    pub _auction_addrs: HashMap<(String, String), String>,
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
        self.update_block(CYCLE_PERIOD / DEFAULT_BLOCK_TIME)
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
                data: Some(to_binary(register_data).unwrap()),
            },
            &[],
        )
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
        self.admin.clone(),
        self.rebalancer_addr.clone(),
    &valence_package::services::rebalancer::RebalancerExecuteMsg::<Empty,Empty>::SystemRebalance {
          limit,
        },
        &[],
      )
    }

    pub fn rebalance_with_update_block(
        &mut self,
        limit: Option<u64>,
    ) -> Result<AppResponse, anyhow::Error> {
        self.update_block_cycle();

        self.rebalance(limit)
    }

    pub fn start_auction(
        &mut self,
        pair: Pair,
        start_block: Option<u64>,
        end_block: u64,
    ) -> Result<AppResponse, anyhow::Error> {
        self.app.execute_contract(
            self.admin.clone(),
            self.auctions_manager_addr.clone(),
            &auctions_manager::msg::ExecuteMsg::Admin(
                auctions_manager::msg::AdminMsgs::OpenAuction {
                    pair,
                    params: NewAuctionParams {
                        start_block,
                        end_block,
                    },
                },
            ),
            &[],
        )
    }

    pub fn do_bid(&mut self, pair: Pair, amount: Coin) -> &mut Self {
        let auction_addr = self
            .app
            .wrap()
            .query_wasm_smart::<Addr>(
                self.auctions_manager_addr.clone(),
                &auction_package::msgs::AuctionsManagerQueryMsg::GetPairAddr { pair },
            )
            .unwrap();

        let res = self
            .app
            .execute_contract(
                self.mm.clone(),
                auction_addr,
                &auction::msg::ExecuteMsg::Bid,
                &[amount],
            )
            .unwrap();
        println!("do_bid: {res:?}");

        self
    }

    pub fn close_auction(&mut self, pair: Pair, limit: Option<u64>) -> &mut Self {
        let auction_addr = self
            .app
            .wrap()
            .query_wasm_smart::<Addr>(
                self.auctions_manager_addr.clone(),
                &auction_package::msgs::AuctionsManagerQueryMsg::GetPairAddr { pair },
            )
            .unwrap();

        let _auction = self
            .app
            .wrap()
            .query_wasm_smart::<auction::state::ActiveAuction>(
                auction_addr.clone(),
                &auction::msg::QueryMsg::GetAuction,
            )
            .unwrap();

        let res = self
            .app
            .execute_contract(
                self.admin.clone(),
                auction_addr,
                &auction::msg::ExecuteMsg::FinishAuction {
                    limit: limit.unwrap_or(5),
                },
                &[],
            )
            .unwrap();
        println!("close_auction: {res:?}");

        self
    }

    pub fn resolve_cycle(&mut self) -> &mut Self {
        let pair1 = Pair::from((ATOM.to_string(), NTRN.to_string()));
        let pair2 = Pair::from((NTRN.to_string(), ATOM.to_string()));

        self.rebalance(None).unwrap();

        self.update_price_from_auction(&pair1, None);
        self.update_price_from_auction(&pair2, None);

        let auction1_started = self
            .start_auction(
                pair1.clone(),
                None,
                self.app.block_info().height + (DAY / DEFAULT_BLOCK_TIME),
            )
            .is_ok();

        let auction2_started = self
            .start_auction(
                pair2.clone(),
                None,
                self.app.block_info().height + (DAY / DEFAULT_BLOCK_TIME),
            )
            .is_ok();

        self.update_block(HALF_DAY / DEFAULT_BLOCK_TIME);

        if auction1_started {
            self.do_bid(pair1.clone(), coin(100000_u128, pair1.clone().1));
        }

        if auction2_started {
            self.do_bid(pair2.clone(), coin(100000_u128, pair2.clone().1));
        }

        self.update_block(HALF_DAY / DEFAULT_BLOCK_TIME);

        if auction1_started {
            self.close_auction(pair1, None);
        }

        if auction2_started {
            self.close_auction(pair2, None);
        }

        self
    }
}

// Auction specific functions
impl Suite {
    pub fn get_price(&self, pair: &Pair) -> Decimal {
        self.app
            .wrap()
            .query_wasm_smart::<GetPriceResponse>(
                self.auctions_manager_addr.clone(),
                &auction_package::msgs::AuctionsManagerQueryMsg::GetPrice { pair: pair.clone() },
            )
            .unwrap()
            .price
    }

    // price_change in percentage
    pub fn change_price_perc(&mut self, pair: &Pair, price_change: SignedDecimal) {
        let price = self.get_price(pair);
        let new_price = if price_change.is_pos() {
            price + price * price_change.0
        } else {
            price - price * price_change.0
        };

        self.change_price(pair, Some(new_price))
    }

    pub fn change_price(&mut self, pair: &Pair, price: Option<Decimal>) {
        self.app
            .execute_contract(
                self.admin.clone(),
                self.oracle_addr.clone(),
                &price_oracle::msg::ExecuteMsg::UpdatePrice {
                    pair: pair.clone(),
                    price,
                },
                &[],
            )
            .unwrap();
    }

    pub fn update_price_from_auction(&mut self, pair: &Pair, price: Option<Decimal>) {
        let _ = self.app.execute_contract(
            self.admin.clone(),
            self.oracle_addr.clone(),
            &price_oracle::msg::ExecuteMsg::UpdatePrice {
                pair: pair.clone(),
                price,
            },
            &[],
        );
    }

    pub fn get_min_limit(&mut self, denom: &str) -> Uint128 {
        self.app
            .wrap()
            .query_wasm_smart(
                self.auctions_manager_addr.clone(),
                &auction_package::msgs::AuctionsManagerQueryMsg::GetMinLimit {
                    denom: denom.to_string(),
                },
            )
            .unwrap()
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
                data: Some(to_binary(&register_data).unwrap()),
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
                data: to_binary(&update_data).unwrap(),
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
}

// Queries
impl Suite {
    pub fn query_rebalancer_config(&self, account: Addr) -> Result<RebalancerConfig, StdError> {
        self.app.wrap().query_wasm_smart::<RebalancerConfig>(
            self.rebalancer_addr.clone(),
            &rebalancer::msg::QueryMsg::GetConfig {
                addr: account.to_string(),
            },
        )
    }

    pub fn query_service_addr_from_manager(
        &self,
        service: ValenceServices,
    ) -> Result<Addr, StdError> {
        self.app.wrap().query_wasm_smart::<Addr>(
            self.manager_addr.clone(),
            &valence_package::msgs::core_query::ServicesManagerQueryMsg::GetServiceAddr { service },
        )
    }

    pub fn query_is_service_on_manager(&self, addr: &str) -> Result<bool, StdError> {
        self.app.wrap().query_wasm_smart::<bool>(
            self.manager_addr.clone(),
            &valence_package::msgs::core_query::ServicesManagerQueryMsg::IsService {
                addr: addr.to_string(),
            },
        )
    }

    pub fn query_rebalancer_system_status(&self) -> Result<SystemRebalanceStatus, StdError> {
        self.app.wrap().query_wasm_smart::<SystemRebalanceStatus>(
            self.rebalancer_addr.clone(),
            &rebalancer::msg::QueryMsg::GetSystemStatus {},
        )
    }
}

// Assertions
impl Suite {
    pub fn assert_rebalancer_config(&self, account_position: u64, config: RebalancerConfig) {
        let account_addr = self.get_account_addr(account_position);
        let query_config = self.query_rebalancer_config(account_addr).unwrap();
        assert_eq!(query_config, config)
    }

    pub fn assert_rebalancer_is_paused(&self, account_position: u64, is_paused: Option<Addr>) {
        let account_addr = self.get_account_addr(account_position);
        let query_config = self.query_rebalancer_config(account_addr).unwrap();
        assert_eq!(query_config.is_paused, is_paused)
    }
}
