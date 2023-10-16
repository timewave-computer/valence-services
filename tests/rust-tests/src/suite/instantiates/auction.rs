use std::str::FromStr;

use auction_package::{helpers::ChainHaltConfig, AuctionStrategy, Pair, PriceFreshnessStrategy};
use cosmwasm_std::{Decimal, Uint128};

use crate::suite::suite::{ATOM, NTRN, OSMO};

#[derive(Clone)]
pub struct AuctionInstantiate {
    pub msg: auction::msg::InstantiateMsg,
}

impl From<AuctionInstantiate> for auction::msg::InstantiateMsg {
    fn from(value: AuctionInstantiate) -> Self {
        value.msg
    }
}

impl AuctionInstantiate {
    pub fn default() -> Self {
        Self::atom_ntrn()
    }

    pub fn atom_ntrn() -> Self {
        Self::new(
            Uint128::new(5_u128),
            Pair(ATOM.to_string(), NTRN.to_string()),
            AuctionStrategy {
                start_price_perc: 2000,
                end_price_perc: 2000,
            },
        )
    }

    pub fn atom_osmo() -> Self {
        Self::new(
            Uint128::new(5_u128),
            Pair(ATOM.to_string(), OSMO.to_string()),
            AuctionStrategy {
                start_price_perc: 2000,
                end_price_perc: 2000,
            },
        )
    }

    pub fn ntrn_atom() -> Self {
        Self::new(
            Uint128::new(10_u128),
            Pair(NTRN.to_string(), ATOM.to_string()),
            AuctionStrategy {
                start_price_perc: 2000,
                end_price_perc: 2000,
            },
        )
    }

    pub fn osmo_atom() -> Self {
        Self::new(
            Uint128::new(10_u128),
            Pair(OSMO.to_string(), ATOM.to_string()),
            AuctionStrategy {
                start_price_perc: 2000,
                end_price_perc: 2000,
            },
        )
    }

    pub fn osmo_ntrn() -> Self {
        Self::new(
            Uint128::new(10_u128),
            Pair(OSMO.to_string(), NTRN.to_string()),
            AuctionStrategy {
                start_price_perc: 2000,
                end_price_perc: 2000,
            },
        )
    }

    pub fn ntrn_osmo() -> Self {
        Self::new(
            Uint128::new(10_u128),
            Pair(NTRN.to_string(), OSMO.to_string()),
            AuctionStrategy {
                start_price_perc: 2000,
                end_price_perc: 2000,
            },
        )
    }

    pub fn new(min_auction_amount: Uint128, pair: Pair, auction_strategy: AuctionStrategy) -> Self {
        Self {
            msg: auction::msg::InstantiateMsg {
                min_auction_amount,
                pair,
                auction_strategy,
                chain_halt_config: ChainHaltConfig {
                    cap: 10_u128,
                    block_avg: Decimal::from_str("3").unwrap(),
                },
                price_freshness_strategy: PriceFreshnessStrategy {
                    limit: Decimal::bps(50000),
                    multipliers: vec![(Decimal::bps(50000), Decimal::one())],
                },
            },
        }
    }

    /* Change functions */
    pub fn change_min_amount(&mut self, min_auction_amount: Uint128) {
        self.msg.min_auction_amount = min_auction_amount;
    }

    pub fn change_pair(&mut self, pair: Pair) {
        self.msg.pair = pair;
    }

    pub fn change_auction_strategy(&mut self, auction_strategy: AuctionStrategy) {
        self.msg.auction_strategy = auction_strategy;
    }
}
