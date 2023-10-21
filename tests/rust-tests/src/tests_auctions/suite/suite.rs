use auction::{
    msg::{GetFundsAmountResponse, NewAuctionParams},
    state::ActiveAuction,
};
use auction_package::{
    helpers::{ChainHaltConfig, GetPriceResponse},
    AuctionStrategy, Pair, PriceFreshnessStrategy,
};
use cosmwasm_std::{coins, Addr, Coin, Decimal, Uint128};
use cw_multi_test::{App, AppResponse, Executor};

use super::suite_builder::SuiteBuilder;

pub const ATOM: &str = "uatom";
pub const NTRN: &str = "untrn";

pub const ADMIN: &str = "admin";
pub const FUNDS_PROVIDER: &str = "funds_provider";
pub const FUNDS_PROVIDER2: &str = "funds_provider2";
pub const FUNDS_PROVIDER3: &str = "funds_provider3";
pub const MM: &str = "market_maker";

pub const DEFAULT_BLOCK_TIME: u64 = 4;


pub const DAY: u64 = 60 * 60 * 24;
pub const HALF_DAY: u64 = 60 * 60 * 12;

pub(crate) struct Suite {
    pub app: App,
    pub admin: Addr,
    pub funds_provider: Addr,
    pub mm: Addr,
    pub auction_addr: Addr,
    pub manager_addr: Addr,
    pub oracle_addr: Addr,
    pub pair: Pair,
    pub _pair_ntrn: Pair,
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

    pub fn update_block_day(&mut self) -> &mut Self {
        self.app.update_block(|b| {
            b.time = b.time.plus_seconds(DAY);
            b.height += DAY / DEFAULT_BLOCK_TIME;
        });
        self
    }
}

// Balances
impl Suite {}

// Execute Auction
impl Suite {
    pub fn start_auction(&mut self, start_block: Option<u64>, end_block: u64) -> &mut Self {
        self.app
            .execute_contract(
                self.admin.clone(),
                self.manager_addr.clone(),
                &auctions_manager::msg::ExecuteMsg::Admin(
                    auctions_manager::msg::AdminMsgs::OpenAuction {
                        pair: self.pair.clone(),
                        params: NewAuctionParams {
                            start_block,
                            end_block,
                        },
                    },
                ),
                &[],
            )
            .unwrap();

        self
    }

    pub fn start_auction_day(&mut self) -> &mut Self {
        self.app
            .execute_contract(
                self.admin.clone(),
                self.manager_addr.clone(),
                &auctions_manager::msg::ExecuteMsg::Admin(
                    auctions_manager::msg::AdminMsgs::OpenAuction {
                        pair: self.pair.clone(),
                        params: NewAuctionParams {
                            start_block: None,
                            end_block: self.app.block_info().height + (DAY / DEFAULT_BLOCK_TIME),
                        },
                    },
                ),
                &[],
            )
            .unwrap();

        self
    }

    pub fn start_auction_day_err(&mut self) -> auction::error::ContractError {
        self.app
            .execute_contract(
                self.admin.clone(),
                self.manager_addr.clone(),
                &auctions_manager::msg::ExecuteMsg::Admin(
                    auctions_manager::msg::AdminMsgs::OpenAuction {
                        pair: self.pair.clone(),
                        params: NewAuctionParams {
                            start_block: None,
                            end_block: self.app.block_info().height + (DAY / DEFAULT_BLOCK_TIME),
                        },
                    },
                ),
                &[],
            )
            .unwrap_err()
            .downcast()
            .unwrap()
    }

    pub fn close_auction(&mut self, limit: Option<u64>) -> &mut Self {
        self.app
            .execute_contract(
                self.admin.clone(),
                self.auction_addr.clone(),
                &auction::msg::ExecuteMsg::FinishAuction {
                    limit: limit.unwrap_or(5),
                },
                &[],
            )
            .unwrap();

        self
    }

    pub fn close_auction_err(&mut self, limit: Option<u64>) -> auction::error::ContractError {
        self.app
            .execute_contract(
                self.admin.clone(),
                self.auction_addr.clone(),
                &auction::msg::ExecuteMsg::FinishAuction {
                    limit: limit.unwrap_or(5),
                },
                &[],
            )
            .unwrap_err()
            .downcast()
            .unwrap()
    }

    pub fn do_bid(&mut self, amount: Coin) -> &mut Self {
        self.app
            .execute_contract(
                self.mm.clone(),
                self.auction_addr.clone(),
                &auction::msg::ExecuteMsg::Bid,
                &[amount],
            )
            .unwrap();

        self
    }

    pub fn do_bid_err(&mut self, amount: Coin) -> auction::error::ContractError {
        self.app
            .execute_contract(
                self.mm.clone(),
                self.auction_addr.clone(),
                &auction::msg::ExecuteMsg::Bid,
                &[amount],
            )
            .unwrap_err()
            .downcast()
            .unwrap()
    }

    /// Do a full bid, with the amount of token pair.0 to buy
    pub fn do_full_bid(&mut self, amount: u128) -> AppResponse {
        let block_price = self.query_auction_price().price;
        let amount = (Decimal::from_atomics(amount, 0).unwrap() * block_price).to_uint_ceil();

        self.app
            .execute_contract(
                self.mm.clone(),
                self.auction_addr.clone(),
                &auction::msg::ExecuteMsg::Bid,
                &coins(amount.u128(), self.pair.1.clone()),
            )
            .unwrap()
    }

    pub fn _do_full_bid_err(&mut self, amount: u128) -> auction::error::ContractError {
        let block_price = self.query_auction_price().price;
        let amount = (Decimal::from_atomics(amount, 0).unwrap() * block_price).to_uint_ceil();

        self.app
            .execute_contract(
                self.mm.clone(),
                self.auction_addr.clone(),
                &auction::msg::ExecuteMsg::Bid,
                &coins(amount.u128(), self.pair.1.clone()),
            )
            .unwrap_err()
            .downcast()
            .unwrap()
    }

    pub fn withdraw_funds(&mut self) -> &mut Self {
        self.app
            .execute_contract(
                self.funds_provider.clone(),
                self.auction_addr.clone(),
                &auction::msg::ExecuteMsg::WithdrawFunds,
                &[],
            )
            .unwrap();

        self
    }

    pub fn withdraw_funds_err(&mut self) -> auction::error::ContractError {
        self.app
            .execute_contract(
                self.funds_provider.clone(),
                self.auction_addr.clone(),
                &auction::msg::ExecuteMsg::WithdrawFunds,
                &[],
            )
            .unwrap_err()
            .downcast()
            .unwrap()
    }

    pub fn clean_last_auction(&mut self) -> &mut Self {
        self.app
            .execute_contract(
                self.manager_addr.clone(),
                self.auction_addr.clone(),
                &auction::msg::ExecuteMsg::CleanAfterAuction,
                &[],
            )
            .unwrap();

        self
    }

    pub fn clean_last_auction_err(&mut self) -> auction::error::ContractError {
        self.app
            .execute_contract(
                self.manager_addr.clone(),
                self.auction_addr.clone(),
                &auction::msg::ExecuteMsg::CleanAfterAuction,
                &[],
            )
            .unwrap_err()
            .downcast()
            .unwrap()
    }

    pub fn finalize_auction(&mut self, funds: &[Coin]) {
        self.auction_funds(None, funds);

        self.start_auction(
            None,
            self.app.block_info().height + (DAY / DEFAULT_BLOCK_TIME),
        );

        self.update_block(HALF_DAY / DEFAULT_BLOCK_TIME);

        self.do_full_bid(100_u128);

        self.update_block(HALF_DAY / DEFAULT_BLOCK_TIME);

        self.close_auction(None);
    }

    pub fn update_auction_strategy(&mut self, strategy: AuctionStrategy) -> &mut Self {
        self.app
            .execute_contract(
                self.admin.clone(),
                self.manager_addr.clone(),
                &auctions_manager::msg::ExecuteMsg::Admin(
                    auctions_manager::msg::AdminMsgs::UpdateStrategy {
                        pair: self.pair.clone(),
                        strategy,
                    },
                ),
                &[],
            )
            .unwrap();

        self
    }
}

// Execute Auctions Manager
impl Suite {
    pub fn init_auction(
        &mut self,
        init_msg: auction::msg::InstantiateMsg,
        min_amount: Option<Uint128>,
    ) -> &mut Self {
        self.app
            .execute_contract(
                self.admin.clone(),
                self.manager_addr.clone(),
                &auctions_manager::msg::ExecuteMsg::Admin(
                    auctions_manager::msg::AdminMsgs::NewAuction {
                        msg: init_msg.clone(),
                        min_amount,
                    },
                ),
                &[],
            )
            .unwrap();

        let addr: Addr = self
            .app
            .wrap()
            .query_wasm_smart(
                self.manager_addr.clone(),
                &auction_package::msgs::AuctionsManagerQueryMsg::GetPairAddr {
                    pair: init_msg.pair,
                },
            )
            .unwrap();

        self.auction_addr = addr;
        self
    }

    pub fn pause_auction(&mut self) -> &mut Self {
        self.app
            .execute_contract(
                self.admin.clone(),
                self.manager_addr.clone(),
                &auctions_manager::msg::ExecuteMsg::Admin(
                    auctions_manager::msg::AdminMsgs::PauseAuction {
                        pair: self.pair.clone(),
                    },
                ),
                &[],
            )
            .unwrap();
        self
    }

    pub fn resume_auction(&mut self) -> &mut Self {
        self.app
            .execute_contract(
                self.admin.clone(),
                self.manager_addr.clone(),
                &auctions_manager::msg::ExecuteMsg::Admin(
                    auctions_manager::msg::AdminMsgs::ResumeAuction {
                        pair: self.pair.clone(),
                    },
                ),
                &[],
            )
            .unwrap();
        self
    }

    pub fn update_oracle(&mut self, oracle_addr: &str) -> &mut Self {
        self.app
            .execute_contract(
                self.admin.clone(),
                self.manager_addr.clone(),
                &auctions_manager::msg::ExecuteMsg::Admin(
                    auctions_manager::msg::AdminMsgs::UpdateOracle {
                        oracle_addr: oracle_addr.to_string(),
                    },
                ),
                &[],
            )
            .unwrap();
        self
    }

    pub fn init_auction_err(
        &mut self,
        init_msg: auction::msg::InstantiateMsg,
        min_amount: Option<Uint128>,
    ) -> anyhow::Error {
        self.app
            .execute_contract(
                self.admin.clone(),
                self.manager_addr.clone(),
                &auctions_manager::msg::ExecuteMsg::Admin(
                    auctions_manager::msg::AdminMsgs::NewAuction {
                        msg: init_msg,
                        min_amount,
                    },
                ),
                &[],
            )
            .unwrap_err()
    }

    pub fn update_oracle_addr(&mut self, oracle_addr: Option<Addr>) -> &mut Self {
        self.app
            .execute_contract(
                self.admin.clone(),
                self.manager_addr.clone(),
                &auctions_manager::msg::ExecuteMsg::Admin(
                    auctions_manager::msg::AdminMsgs::UpdateOracle {
                        oracle_addr: oracle_addr.unwrap_or(self.oracle_addr.clone()).to_string(),
                    },
                ),
                &[],
            )
            .unwrap();

        self
    }

    pub fn auction_funds_manager(&mut self, user: Option<Addr>, amount: &[Coin]) -> &mut Self {
        self.app
            .execute_contract(
                user.unwrap_or(self.funds_provider.clone()),
                self.manager_addr.clone(),
                &auctions_manager::msg::ExecuteMsg::AuctionFunds {
                    pair: self.pair.clone(),
                },
                amount,
            )
            .unwrap();

        self
    }

    pub fn withdraw_funds_manager(&mut self, user: Addr) -> &mut Self {
        self.app
            .execute_contract(
                user,
                self.manager_addr.clone(),
                &auctions_manager::msg::ExecuteMsg::WithdrawFunds {
                    pair: self.pair.clone(),
                },
                &[],
            )
            .unwrap();

        self
    }

    pub fn update_chain_halt_config(
        &mut self,
        pair: Pair,
        halt_config: ChainHaltConfig,
    ) -> &mut Self {
        self.app
            .execute_contract(
                self.admin.clone(),
                self.manager_addr.clone(),
                &auctions_manager::msg::ExecuteMsg::Admin(
                    auctions_manager::msg::AdminMsgs::UpdateChainHaltConfig { pair, halt_config },
                ),
                &[],
            )
            .unwrap();

        self
    }

    pub fn update_price_freshness_strategy(
        &mut self,
        pair: Pair,
        strategy: PriceFreshnessStrategy,
    ) -> &mut Self {
        self.app
            .execute_contract(
                self.admin.clone(),
                self.manager_addr.clone(),
                &auctions_manager::msg::ExecuteMsg::Admin(
                    auctions_manager::msg::AdminMsgs::UpdatePriceFreshnessStrategy {
                        pair,
                        strategy,
                    },
                ),
                &[],
            )
            .unwrap();

        self
    }
}

// Execute Oracle
impl Suite {
    pub fn update_oracle_price(&mut self, pair: Pair, price: Option<Decimal>) -> &mut Self {
        self.app
            .execute_contract(
                self.admin.clone(),
                self.oracle_addr.clone(),
                &price_oracle::msg::ExecuteMsg::UpdatePrice { pair, price },
                &[],
            )
            .unwrap();

        self
    }

    pub fn update_oracle_price_err(
        &mut self,
        pair: Pair,
        price: Option<Decimal>,
    ) -> price_oracle::error::ContractError {
        self.app
            .execute_contract(
                self.admin.clone(),
                self.oracle_addr.clone(),
                &price_oracle::msg::ExecuteMsg::UpdatePrice { pair, price },
                &[],
            )
            .unwrap_err()
            .downcast()
            .unwrap()
    }
}

// Queries
impl Suite {
    pub fn query_auction_config(&self) -> auction_package::helpers::AuctionConfig {
        self.app
            .wrap()
            .query_wasm_smart(
                self.auction_addr.clone(),
                &auction::msg::QueryMsg::GetConfig {},
            )
            .unwrap()
    }

    pub fn query_auction_price(&self) -> GetPriceResponse {
        self.app
            .wrap()
            .query_wasm_smart(self.auction_addr.clone(), &auction::msg::QueryMsg::GetPrice)
            .unwrap()
    }

    pub fn query_oracle_config(&self) -> price_oracle::state::Config {
        self.app
            .wrap()
            .query_wasm_smart(
                self.oracle_addr.clone(),
                &price_oracle::msg::QueryMsg::GetConfig,
            )
            .unwrap()
    }

    pub fn query_oracle_addr(&self) -> Addr {
        self.app
            .wrap()
            .query_wasm_smart(
                self.manager_addr.clone(),
                &auction_package::msgs::AuctionsManagerQueryMsg::GetOracleAddr,
            )
            .unwrap()
    }

    pub fn query_auction_funds(&self, user: &str) -> GetFundsAmountResponse {
        self.app
            .wrap()
            .query_wasm_smart(
                self.auction_addr.clone(),
                &auction::msg::QueryMsg::GetFundsAmount {
                    addr: user.to_string(),
                },
            )
            .unwrap()
    }

    pub fn query_oracle_price(&self, pair: Pair) -> GetPriceResponse {
        self.app
            .wrap()
            .query_wasm_smart(
                self.oracle_addr.clone(),
                &price_oracle::msg::QueryMsg::GetPrice { pair },
            )
            .unwrap()
    }

    pub fn query_auction_details(&self) -> ActiveAuction {
        self.app
            .wrap()
            .query_wasm_smart(
                self.auction_addr.clone(),
                &auction::msg::QueryMsg::GetAuction,
            )
            .unwrap()
    }

    pub fn query_auction_strategy(&self) -> AuctionStrategy {
        self.app
            .wrap()
            .query_wasm_smart(
                self.auction_addr.clone(),
                &auction::msg::QueryMsg::GetStrategy,
            )
            .unwrap()
    }
}

// helpers
impl Suite {
    pub fn calc_price_per_block(&self) -> Decimal {
        let auction_details = self.query_auction_details();

        let price_diff = auction_details.start_price - auction_details.end_price;
        let block_diff = auction_details.end_block - auction_details.start_block;

        price_diff / Decimal::from_atomics(block_diff, 0).unwrap()
    }

    pub fn get_attr_value(&self, res: &AppResponse, key_name: &str) -> Option<String> {
        let mut value: Option<String> = None;
        res.events.iter().for_each(|event| {
            if value.is_none() {
                value = event
                    .attributes
                    .iter()
                    .find(|attr| attr.key == key_name)
                    .map(|attr| attr.value.clone())
            }
        });
        value
    }
}

// Assertions
impl Suite {
    pub fn assert_auction_status(&self, status: auction::state::ActiveAuctionStatus) {
        let auction_details = self.query_auction_details();

        assert_eq!(auction_details.status, status);
    }
}
