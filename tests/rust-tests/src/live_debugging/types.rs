use std::str::FromStr;

use auction_package::{states::MinAmount, Pair, Price};
use chrono::{DateTime, NaiveDate, NaiveDateTime, Timelike};
use cosmwasm_schema::cw_serde;
use cosmwasm_std::{BlockInfo, Coin, Timestamp};
use valence_package::services::rebalancer::{BaseDenom, RebalancerConfig};

#[derive(
    cosmwasm_schema::serde::Serialize,
    cosmwasm_schema::serde::Deserialize,
    std::clone::Clone,
    std::fmt::Debug,
    std::cmp::PartialEq,
    cosmwasm_schema::schemars::JsonSchema,
)]
#[allow(clippy::derive_partial_eq_without_eq)] // Allow users of `#[cw_serde]` to not implement Eq without clippy complaining
#[serde(crate = "::cosmwasm_schema::serde")]
#[schemars(crate = "::cosmwasm_schema::schemars")]
pub struct Balances {
    pub balances: Vec<Coin>,
}

#[derive(
    cosmwasm_schema::serde::Serialize,
    cosmwasm_schema::serde::Deserialize,
    std::clone::Clone,
    std::fmt::Debug,
    std::cmp::PartialEq,
    cosmwasm_schema::schemars::JsonSchema,
)]
#[allow(clippy::derive_partial_eq_without_eq)] // Allow users of `#[cw_serde]` to not implement Eq without clippy complaining
#[serde(crate = "::cosmwasm_schema::serde")]
#[schemars(crate = "::cosmwasm_schema::schemars")]
pub struct BlockRes {
    pub block: Block,
}

#[derive(
    cosmwasm_schema::serde::Serialize,
    cosmwasm_schema::serde::Deserialize,
    std::clone::Clone,
    std::fmt::Debug,
    std::cmp::PartialEq,
    cosmwasm_schema::schemars::JsonSchema,
)]
#[allow(clippy::derive_partial_eq_without_eq)] // Allow users of `#[cw_serde]` to not implement Eq without clippy complaining
#[serde(crate = "::cosmwasm_schema::serde")]
#[schemars(crate = "::cosmwasm_schema::schemars")]
pub struct Block {
    pub header: BlockInfoTemp,
}

#[derive(
    cosmwasm_schema::serde::Serialize,
    cosmwasm_schema::serde::Deserialize,
    std::clone::Clone,
    std::fmt::Debug,
    std::cmp::PartialEq,
    cosmwasm_schema::schemars::JsonSchema,
)]
#[allow(clippy::derive_partial_eq_without_eq)] // Allow users of `#[cw_serde]` to not implement Eq without clippy complaining
#[serde(crate = "::cosmwasm_schema::serde")]
#[schemars(crate = "::cosmwasm_schema::schemars")]
pub struct BlockInfoTemp {
    pub height: String,
    pub time: String,
    pub chain_id: String,
}

impl From<BlockInfoTemp> for BlockInfo {
    fn from(block_info_temp: BlockInfoTemp) -> Self {
        let seconds =
            NaiveDateTime::parse_from_str(&block_info_temp.time, "%Y-%m-%dT%H:%M:%S%.9fZ")
                .unwrap()
                .and_utc()
                .timestamp_nanos_opt()
                .unwrap();

        BlockInfo {
            height: block_info_temp.height.parse().unwrap(),
            time: Timestamp::from_nanos(seconds as u64),
            chain_id: block_info_temp.chain_id,
        }
    }
}

#[cw_serde]
pub struct ConfigRes {
    pub data: RebalancerConfig,
}

#[cw_serde]
pub struct AllPricesRes {
    pub data: Prices,
}

pub type Prices = Vec<(Pair, Price)>;

#[cw_serde]
pub struct WhitelistDenomsRes {
    pub data: WhitelistDenoms,
}

#[cw_serde]
pub struct WhitelistDenoms {
    pub denom_whitelist: Vec<String>,
    pub base_denom_whitelist: Vec<BaseDenom>,
}

#[cw_serde]
pub struct MinLimitsRes {
    pub data: MinAmount,
}
