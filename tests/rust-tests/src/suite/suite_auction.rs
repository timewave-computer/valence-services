use auction::msg::NewAuctionParams;
use auction_package::{helpers::GetPriceResponse, Pair};
use cosmwasm_std::{coin, Addr, Coin, Decimal, Uint128};
use cw_multi_test::{AppResponse, Executor};
use valence_package::signed_decimal::SignedDecimal;

use super::suite::{Suite, ATOM, DAY, DEFAULT_BLOCK_TIME, HALF_DAY, NTRN};

// Executables
impl Suite {
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

        self.update_price(&pair1, None);
        self.update_price(&pair2, None);

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

    // price_change in percentage
    pub fn change_price_perc(&mut self, pair: &Pair, price_change: SignedDecimal) {
        let price = self.get_price(pair);
        let new_price = if price_change.is_pos() {
            price + price * price_change.0
        } else {
            price - price * price_change.0
        };

        self.update_price(pair, Some(new_price)).unwrap();
    }

    pub fn update_price(
        &mut self,
        pair: &Pair,
        price: Option<Decimal>,
    ) -> Result<AppResponse, anyhow::Error> {
        self.app.execute_contract(
            self.admin.clone(),
            self.oracle_addr.clone(),
            &price_oracle::msg::ExecuteMsg::UpdatePrice {
                pair: pair.clone(),
                price,
            },
            &[],
        )
    }
}

// Queries
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
