use auction::{
    msg::{GetFundsAmountResponse, NewAuctionParams},
    state::ActiveAuction,
};
use auction_package::{
    helpers::{ChainHaltConfig, GetPriceResponse},
    msgs::AuctionsManagerQueryMsg,
    states::MinAmount,
    AuctionStrategy, Pair, PriceFreshnessStrategy,
};
use cosmwasm_std::{coin, coins, Addr, Coin, Decimal, Uint128};
use cw_multi_test::{AppResponse, Executor};
use valence_package::signed_decimal::SignedDecimal;

use super::suite::{Suite, ATOM, DAY, DEFAULT_BLOCK_TIME, HALF_DAY, NTRN};

// Executables
impl Suite {
    pub fn init_auction(
        &mut self,
        pair: Pair,
        init_msg: auction::msg::InstantiateMsg,
        min_amount: Option<MinAmount>,
    ) -> &mut Self {
        self.app
            .execute_contract(
                self.admin.clone(),
                self.auctions_manager_addr.clone(),
                &auctions_manager::msg::ExecuteMsg::Admin(Box::new(
                    auctions_manager::msg::AdminMsgs::NewAuction {
                        msg: init_msg.clone(),
                        label: "label".to_string(),
                        min_amount,
                    },
                )),
                &[],
            )
            .unwrap();

        let addr: Addr = self
            .app
            .wrap()
            .query_wasm_smart(
                self.auctions_manager_addr.clone(),
                &auction_package::msgs::AuctionsManagerQueryMsg::GetPairAddr {
                    pair: init_msg.pair,
                },
            )
            .unwrap();

        self.auction_addrs.insert(pair.into(), addr);
        self
    }

    pub fn init_auction_err(
        &mut self,
        init_msg: auction::msg::InstantiateMsg,
        min_amount: Option<MinAmount>,
    ) -> anyhow::Error {
        self.app
            .execute_contract(
                self.admin.clone(),
                self.auctions_manager_addr.clone(),
                &auctions_manager::msg::ExecuteMsg::Admin(Box::new(
                    auctions_manager::msg::AdminMsgs::NewAuction {
                        msg: init_msg,
                        label: "label".to_string(),
                        min_amount,
                    },
                )),
                &[],
            )
            .unwrap_err()
    }

    pub fn auction_funds(&mut self, user: Addr, auction_addr: Addr, amount: &[Coin]) -> &mut Self {
        self.app
            .execute_contract(
                user,
                auction_addr,
                &auction::msg::ExecuteMsg::AuctionFunds {},
                amount,
            )
            .unwrap();

        self
    }

    pub fn auction_funds_err(
        &mut self,
        user: Addr,
        auction_addr: Addr,
        amount: &[Coin],
    ) -> auction::error::ContractError {
        self.app
            .execute_contract(
                user,
                auction_addr,
                &auction::msg::ExecuteMsg::AuctionFunds {},
                amount,
            )
            .unwrap_err()
            .downcast()
            .unwrap()
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
            &auctions_manager::msg::ExecuteMsg::Admin(Box::new(
                auctions_manager::msg::AdminMsgs::OpenAuction {
                    pair,
                    params: NewAuctionParams {
                        start_block,
                        end_block,
                    },
                },
            )),
            &[],
        )
    }

    pub fn start_auction_day(&mut self, pair: Pair) -> Result<AppResponse, anyhow::Error> {
        self.start_auction(
            pair,
            None,
            self.app.block_info().height + (DAY / DEFAULT_BLOCK_TIME),
        )
    }

    pub fn start_auction_day_err(&mut self, pair: Pair) -> auction::error::ContractError {
        self.start_auction(
            pair,
            None,
            self.app.block_info().height + (DAY / DEFAULT_BLOCK_TIME),
        )
        .unwrap_err()
        .downcast()
        .unwrap()
    }

    pub fn do_bid(&mut self, pair: Pair, amount: Coin) -> Result<AppResponse, anyhow::Error> {
        let auction_addr = self
            .app
            .wrap()
            .query_wasm_smart::<Addr>(
                self.auctions_manager_addr.clone(),
                &auction_package::msgs::AuctionsManagerQueryMsg::GetPairAddr { pair },
            )
            .unwrap();

        self.app.execute_contract(
            self.mm.clone(),
            auction_addr,
            &auction::msg::ExecuteMsg::Bid {},
            &[amount],
        )
    }

    pub fn do_bid_err(&mut self, pair: Pair, amount: Coin) -> auction::error::ContractError {
        let auction_addr = self
            .app
            .wrap()
            .query_wasm_smart::<Addr>(
                self.auctions_manager_addr.clone(),
                &auction_package::msgs::AuctionsManagerQueryMsg::GetPairAddr { pair },
            )
            .unwrap();

        self.app
            .execute_contract(
                self.mm.clone(),
                auction_addr,
                &auction::msg::ExecuteMsg::Bid {},
                &[amount],
            )
            .unwrap_err()
            .downcast()
            .unwrap()
    }

    pub fn close_auction(
        &mut self,
        pair: Pair,
        limit: Option<u64>,
    ) -> Result<AppResponse, anyhow::Error> {
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

        self.app.execute_contract(
            self.admin.clone(),
            auction_addr,
            &auction::msg::ExecuteMsg::FinishAuction {
                limit: limit.unwrap_or(5),
            },
            &[],
        )
    }

    pub fn close_auction_err(
        &mut self,
        pair: Pair,
        limit: Option<u64>,
    ) -> auction::error::ContractError {
        self.close_auction(pair, limit)
            .unwrap_err()
            .downcast()
            .unwrap()
    }

    pub fn resolve_cycle(&mut self) -> &mut Self {
        let pair1 = Pair::from((ATOM.to_string(), NTRN.to_string()));
        let pair2 = Pair::from((NTRN.to_string(), ATOM.to_string()));

        self.rebalance(None).unwrap();

        // Its fine if we can't update price yet
        let _ = self.update_price(pair1.clone(), None);
        let _ = self.update_price(pair2.clone(), None);

        let _ = self.start_auction(
            pair1.clone(),
            None,
            self.app.block_info().height + (DAY / DEFAULT_BLOCK_TIME),
        );
        let auction1_started = self
            .query_auction_details(
                self.auction_addrs
                    .get(&pair1.clone().into())
                    .unwrap()
                    .clone(),
            )
            .status
            == auction::state::ActiveAuctionStatus::Started;

        let _ = self.start_auction(
            pair2.clone(),
            None,
            self.app.block_info().height + (DAY / DEFAULT_BLOCK_TIME),
        );
        let auction2_started = self
            .query_auction_details(
                self.auction_addrs
                    .get(&pair2.clone().into())
                    .unwrap()
                    .clone(),
            )
            .status
            == auction::state::ActiveAuctionStatus::Started;

        self.update_block(HALF_DAY / DEFAULT_BLOCK_TIME);

        if auction1_started {
            self.do_bid(pair1.clone(), coin(100000_u128, pair1.clone().1))
                .unwrap();
        }

        if auction2_started {
            self.do_bid(pair2.clone(), coin(100000_u128, pair2.clone().1))
                .unwrap();
        }

        self.update_block(HALF_DAY / DEFAULT_BLOCK_TIME);

        if auction1_started {
            self.close_auction(pair1, None).unwrap();
        }

        if auction2_started {
            self.close_auction(pair2, None).unwrap();
        }

        self
    }

    pub fn finalize_auction(&mut self, funds: &[Coin]) {
        self.auction_funds(
            self.get_account_addr(0),
            self.get_default_auction_addr(),
            funds,
        );

        self.start_auction_day(self.pair.clone()).unwrap();

        self.update_block(HALF_DAY / DEFAULT_BLOCK_TIME);

        self.do_full_bid(funds[0].amount.u128());

        self.update_block(HALF_DAY / DEFAULT_BLOCK_TIME);

        self.close_auction(self.pair.clone(), None).unwrap();
    }

    // price_change in percentage
    pub fn change_price_perc(&mut self, pair: Pair, price_change: SignedDecimal) {
        let price = self.get_price(pair.clone());
        let new_price = if price_change.is_pos() {
            price + price * price_change.value()
        } else {
            price - price * price_change.value()
        };

        self.update_price(pair, Some(new_price)).unwrap();
    }

    pub fn update_price(
        &mut self,
        pair: Pair,
        price: Option<Decimal>,
    ) -> Result<AppResponse, anyhow::Error> {
        self.app.execute_contract(
            self.admin.clone(),
            self.oracle_addr.clone(),
            &price_oracle::msg::ExecuteMsg::UpdatePrice { pair, price },
            &[],
        )
    }

    pub fn update_price_err(
        &mut self,
        pair: Pair,
        price: Option<Decimal>,
    ) -> price_oracle::error::ContractError {
        self.update_price(pair, price)
            .unwrap_err()
            .downcast()
            .unwrap()
    }

    pub fn do_full_bid(&mut self, amount: u128) -> AppResponse {
        let block_price = self
            .query_auction_price(self.get_default_auction_addr())
            .price;
        let amount = (Decimal::from_atomics(amount, 0).unwrap() * block_price).to_uint_ceil();

        self.app
            .execute_contract(
                self.mm.clone(),
                self.get_default_auction_addr(),
                &auction::msg::ExecuteMsg::Bid {},
                &coins(amount.u128(), self.pair.1.clone()),
            )
            .unwrap()
    }

    pub fn pause_auction(&mut self, pair: Pair) -> &mut Self {
        self.app
            .execute_contract(
                self.admin.clone(),
                self.auctions_manager_addr.clone(),
                &auctions_manager::msg::ExecuteMsg::Admin(Box::new(
                    auctions_manager::msg::AdminMsgs::PauseAuction { pair },
                )),
                &[],
            )
            .unwrap();
        self
    }

    pub fn resume_auction(&mut self, pair: Pair) -> Result<AppResponse, anyhow::Error> {
        self.app.execute_contract(
            self.admin.clone(),
            self.auctions_manager_addr.clone(),
            &auctions_manager::msg::ExecuteMsg::Admin(Box::new(
                auctions_manager::msg::AdminMsgs::ResumeAuction { pair },
            )),
            &[],
        )
    }

    pub fn clean_last_auction(&mut self, auction_addr: Addr) -> &mut Self {
        self.app
            .execute_contract(
                self.auctions_manager_addr.clone(),
                auction_addr,
                &auction::msg::ExecuteMsg::CleanAfterAuction {},
                &[],
            )
            .unwrap();

        self
    }

    pub fn clean_last_auction_err(&mut self, auction_addr: Addr) -> auction::error::ContractError {
        self.app
            .execute_contract(
                self.auctions_manager_addr.clone(),
                auction_addr,
                &auction::msg::ExecuteMsg::CleanAfterAuction {},
                &[],
            )
            .unwrap_err()
            .downcast()
            .unwrap()
    }

    pub fn update_auction_strategy(&mut self, pair: Pair, strategy: AuctionStrategy) -> &mut Self {
        self.app
            .execute_contract(
                self.admin.clone(),
                self.auctions_manager_addr.clone(),
                &auctions_manager::msg::ExecuteMsg::Admin(Box::new(
                    auctions_manager::msg::AdminMsgs::UpdateStrategy { pair, strategy },
                )),
                &[],
            )
            .unwrap();

        self
    }

    pub fn update_oracle(&mut self, oracle_addr: &str) -> &mut Self {
        self.app
            .execute_contract(
                self.admin.clone(),
                self.auctions_manager_addr.clone(),
                &auctions_manager::msg::ExecuteMsg::Admin(Box::new(
                    auctions_manager::msg::AdminMsgs::UpdateOracle {
                        oracle_addr: oracle_addr.to_string(),
                    },
                )),
                &[],
            )
            .unwrap();
        self
    }

    pub fn auction_funds_manager(&mut self, pair: Pair, user: Addr, amount: &[Coin]) -> &mut Self {
        self.app
            .execute_contract(
                user,
                self.auctions_manager_addr.clone(),
                &auctions_manager::msg::ExecuteMsg::AuctionFunds { pair },
                amount,
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
                self.auctions_manager_addr.clone(),
                &auctions_manager::msg::ExecuteMsg::Admin(Box::new(
                    auctions_manager::msg::AdminMsgs::UpdateChainHaltConfig { pair, halt_config },
                )),
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
                self.auctions_manager_addr.clone(),
                &auctions_manager::msg::ExecuteMsg::Admin(Box::new(
                    auctions_manager::msg::AdminMsgs::UpdatePriceFreshnessStrategy {
                        pair,
                        strategy,
                    },
                )),
                &[],
            )
            .unwrap();

        self
    }

    pub fn withdraw_funds(
        &mut self,
        user: Addr,
        auction_addr: Addr,
    ) -> Result<AppResponse, anyhow::Error> {
        self.app.execute_contract(
            user,
            auction_addr,
            &auction::msg::ExecuteMsg::WithdrawFunds {},
            &[],
        )
    }

    pub fn withdraw_funds_err(
        &mut self,
        user: Addr,
        auction_addr: Addr,
    ) -> auction::error::ContractError {
        self.withdraw_funds(user, auction_addr)
            .unwrap_err()
            .downcast()
            .unwrap()
    }

    pub fn withdraw_funds_manager(&mut self, pair: Pair, user: Addr) -> &mut Self {
        self.app
            .execute_contract(
                user,
                self.auctions_manager_addr.clone(),
                &auctions_manager::msg::ExecuteMsg::WithdrawFunds { pair },
                &[],
            )
            .unwrap();

        self
    }
}

// Queries
impl Suite {
    pub fn get_default_auction_addr(&self) -> Addr {
        self.auction_addrs
            .get(&self.pair.clone().into())
            .unwrap()
            .clone()
    }

    pub fn get_price(&self, pair: Pair) -> Decimal {
        self.app
            .wrap()
            .query_wasm_smart::<GetPriceResponse>(
                self.auctions_manager_addr.clone(),
                &auction_package::msgs::AuctionsManagerQueryMsg::GetPrice { pair },
            )
            .unwrap()
            .price
    }

    pub fn get_send_min_limit(&mut self, denom: &str) -> Uint128 {
        self.app
            .wrap()
            .query_wasm_smart::<MinAmount>(
                self.auctions_manager_addr.clone(),
                &auction_package::msgs::AuctionsManagerQueryMsg::GetMinLimit {
                    denom: denom.to_string(),
                },
            )
            .unwrap()
            .send
    }

    pub fn query_auction_details(&self, auction_addr: Addr) -> ActiveAuction {
        self.app
            .wrap()
            .query_wasm_smart(auction_addr, &auction::msg::QueryMsg::GetAuction)
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

    pub fn query_auction_price(&self, auction_addr: Addr) -> GetPriceResponse {
        self.app
            .wrap()
            .query_wasm_smart(auction_addr, &auction::msg::QueryMsg::GetPrice)
            .unwrap()
    }

    pub fn query_auction_strategy(&self, auction_addr: Addr) -> AuctionStrategy {
        self.app
            .wrap()
            .query_wasm_smart(auction_addr, &auction::msg::QueryMsg::GetStrategy)
            .unwrap()
    }

    pub fn query_auction_config(
        &self,
        auction_addr: Addr,
    ) -> auction_package::helpers::AuctionConfig {
        self.app
            .wrap()
            .query_wasm_smart(auction_addr, &auction::msg::QueryMsg::GetConfig {})
            .unwrap()
    }

    pub fn query_oracle_addr(&self) -> Addr {
        self.app
            .wrap()
            .query_wasm_smart(
                self.auctions_manager_addr.clone(),
                &auction_package::msgs::AuctionsManagerQueryMsg::GetOracleAddr,
            )
            .unwrap()
    }

    pub fn query_auction_funds(&self, user: Addr, auction_addr: Addr) -> GetFundsAmountResponse {
        self.app
            .wrap()
            .query_wasm_smart(
                auction_addr,
                &auction::msg::QueryMsg::GetFundsAmount {
                    addr: user.to_string(),
                },
            )
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

    pub fn query_auctions_manager_all_pairs(&self) -> Vec<(Pair, Addr)> {
        self.app
            .wrap()
            .query_wasm_smart(
                self.auctions_manager_addr.clone(),
                &AuctionsManagerQueryMsg::GetPairs {
                    start_after: None,
                    limit: None,
                },
            )
            .unwrap()
    }
}

// Helpers
impl Suite {
    pub fn calc_price_per_block(&self, auction_addr: Addr) -> Decimal {
        let auction_details = self.query_auction_details(auction_addr);

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
        let auction_details = self.query_auction_details(self.get_default_auction_addr());

        assert_eq!(auction_details.status, status);
    }
}
