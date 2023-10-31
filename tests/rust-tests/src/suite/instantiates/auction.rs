use std::str::FromStr;

use auction_package::{helpers::ChainHaltConfig, AuctionStrategy, Pair, PriceFreshnessStrategy};
use cosmwasm_std::Decimal;

use crate::suite::suite::{ATOM, DEFAULT_BLOCK_TIME, NTRN, OSMO};

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
            Pair(ATOM.to_string(), NTRN.to_string()),
            AuctionStrategy {
                start_price_perc: 2000,
                end_price_perc: 2000,
            },
        )
    }

    pub fn atom_osmo() -> Self {
        Self::new(
            Pair(ATOM.to_string(), OSMO.to_string()),
            AuctionStrategy {
                start_price_perc: 2000,
                end_price_perc: 2000,
            },
        )
    }

    pub fn ntrn_atom() -> Self {
        Self::new(
            Pair(NTRN.to_string(), ATOM.to_string()),
            AuctionStrategy {
                start_price_perc: 2000,
                end_price_perc: 2000,
            },
        )
    }

    pub fn osmo_atom() -> Self {
        Self::new(
            Pair(OSMO.to_string(), ATOM.to_string()),
            AuctionStrategy {
                start_price_perc: 2000,
                end_price_perc: 2000,
            },
        )
    }

    pub fn osmo_ntrn() -> Self {
        Self::new(
            Pair(OSMO.to_string(), NTRN.to_string()),
            AuctionStrategy {
                start_price_perc: 2000,
                end_price_perc: 2000,
            },
        )
    }

    pub fn ntrn_osmo() -> Self {
        Self::new(
            Pair(NTRN.to_string(), OSMO.to_string()),
            AuctionStrategy {
                start_price_perc: 2000,
                end_price_perc: 2000,
            },
        )
    }

    pub fn new(pair: Pair, auction_strategy: AuctionStrategy) -> Self {
        Self {
            msg: auction::msg::InstantiateMsg {
                pair,
                auction_strategy,
                chain_halt_config: ChainHaltConfig {
                    cap: 14400_u128,
                    block_avg: Decimal::from_str(&DEFAULT_BLOCK_TIME.to_string()).unwrap(),
                },
                price_freshness_strategy: PriceFreshnessStrategy {
                    limit: Decimal::bps(30000),
                    multipliers: vec![
                        (Decimal::bps(20000), Decimal::bps(20000)),
                        (Decimal::bps(10000), Decimal::bps(15000)),
                    ],
                },
            },
        }
    }

    /* Change functions */
    pub fn change_pair(&mut self, pair: Pair) -> &mut Self {
        self.msg.pair = pair;
        self
    }

    pub fn change_auction_strategy(&mut self, auction_strategy: AuctionStrategy) -> &mut Self {
        self.msg.auction_strategy = auction_strategy;
        self
    }
}
