use std::str::FromStr;

use cosmwasm_schema::cw_serde;
use cosmwasm_std::{Addr, Decimal, Timestamp, Uint128};
use valence_macros::valence_service_execute_msgs;

use crate::{error::ValenceError, helpers::OptionalField, signed_decimal::SignedDecimal};

/// Rebalancer execute msgs.
#[valence_service_execute_msgs]
#[cw_serde]
pub enum RebalancerExecuteMsg<A = RebalancerData, B = RebalancerUpdateData> {
    SystemRebalance { limit: Option<u64> },
}

#[cw_serde]
pub struct RebalancerData {
    /// The trustee address that can pause/resume the service
    pub trustee: Option<String>,
    /// Base denom we will be calculating everything based on
    pub base_denom: String,
    /// List of targets to rebalance for this account
    pub targets: Vec<Target>,
    /// PID parameters the account want to calculate the rebalance with
    pub pid: PID,
    /// The max limit in percentage the rebalancer is allowed to sell in cycle
    pub max_limit_bps: Option<u64>, // BPS
    /// The strategy to use when overriding targets
    pub target_override_strategy: TargetOverrideStrategy,
}

#[cw_serde]
pub struct RebalancerUpdateData {
    pub trustee: Option<OptionalField<String>>,
    pub base_denom: Option<String>,
    pub targets: Vec<Target>,
    pub pid: Option<PID>,
    pub max_limit: Option<u64>, // BPS
    pub target_override_strategy: Option<TargetOverrideStrategy>,
}

impl RebalancerData {
    pub fn to_config(self) -> Result<RebalancerConfig, ValenceError> {
        let max_limit = if let Some(max_limit) = self.max_limit_bps {
            Decimal::bps(max_limit)
        } else {
            Decimal::one()
        };

        let has_min_balance = self.targets.iter().any(|t| t.min_balance.is_some());

        Ok(RebalancerConfig {
            is_paused: None,
            trustee: self.trustee,
            base_denom: self.base_denom,
            targets: self.targets.into_iter().map(|t| t.into()).collect(),
            pid: self.pid.into_parsed()?,
            max_limit,
            last_rebalance: Timestamp::from_seconds(0),
            has_min_balance,
            target_override_strategy: self.target_override_strategy,
        })
    }
}

#[cw_serde]
pub struct RebalancerConfig {
    /// Is_paused holds the pauser if it is paused, None if its not paused
    pub is_paused: Option<Addr>,
    /// the address that can pause and resume the service
    pub trustee: Option<String>,
    /// The base denom we will be calculating everything based on
    pub base_denom: String,
    /// A vector of targets to rebalance for this account
    pub targets: Vec<ParsedTarget>,
    /// The PID parameters the account want to rebalance with
    pub pid: ParsedPID,
    /// Percentage from the total balance that we are allowed to sell in 1 rebalance cycle.
    pub max_limit: Decimal, // percentage
    /// When the last rebalance happened.
    pub last_rebalance: Timestamp,
    pub has_min_balance: bool,
    pub target_override_strategy: TargetOverrideStrategy,
}

/// The strategy we will use when overriding targets
#[cw_serde]
pub enum TargetOverrideStrategy {
    Proportional,
    Priority,
}

/// The target struct that holds all info about a single denom target
#[cw_serde]
pub struct Target {
    /// The name of the denom
    pub denom: String,
    /// The percentage of the total balance we want to have in this denom
    pub percentage: u64,
    /// The minimum balance the account should hold for this denom.
    /// Can only be a single one for an account
    pub min_balance: Option<Uint128>,
}

/// A parsed target struct that contains all info about a single denom target
#[cw_serde]
pub struct ParsedTarget {
    /// The name of the denom
    pub denom: String,
    /// The percentage of the total balance we want to have in this denom
    pub percentage: Decimal,
        /// The minimum balance the account should hold for this denom.
    /// Can only be a single one for an account
    pub min_balance: Option<Uint128>,
    /// The input we got from the last rebalance.
    pub last_input: Option<Decimal>,
    /// The last I value we got from the last rebalance PID calculation.
    pub last_i: SignedDecimal,
}

impl From<Target> for ParsedTarget {
    fn from(val: Target) -> Self {
        ParsedTarget {
            denom: val.denom,
            percentage: Decimal::bps(val.percentage),
            min_balance: val.min_balance,
            last_input: None,
            last_i: SignedDecimal::zero(),
        }
    }
}

/// The PID parameters we use to calculate the rebalance amounts
#[cw_serde]
pub struct PID {
    pub p: String,
    pub i: String,
    pub d: String,
}

impl PID {
    pub fn into_parsed(self) -> Result<ParsedPID, ValenceError> {
        ParsedPID {
            p: Decimal::from_str(&self.p)?,
            i: Decimal::from_str(&self.i)?,
            d: Decimal::from_str(&self.d)?,
        }
        .verify()
    }
}

#[cw_serde]
pub struct ParsedPID {
    pub p: Decimal,
    pub i: Decimal,
    pub d: Decimal,
}

impl ParsedPID {
    pub fn verify(self) -> Result<Self, ValenceError> {
        if self.p > Decimal::one() || self.i > Decimal::one() {
            return Err(ValenceError::PIDErrorOver);
        }

        Ok(self)
    }
}

#[cfg(test)]
mod test {
    use crate::error::ValenceError;

    use super::PID;

    #[test]
    fn test_verify() {
        PID {
            p: "1".to_string(),
            i: "0.5".to_string(),
            d: "0.5".to_string(),
        }
        .into_parsed()
        .unwrap();

        let err = PID {
            p: "1.1".to_string(),
            i: "0.5".to_string(),
            d: "0.5".to_string(),
        }
        .into_parsed()
        .unwrap_err();

        assert_eq!(err, ValenceError::PIDErrorOver);

        let err = PID {
            p: "1".to_string(),
            i: "1.5".to_string(),
            d: "0.5".to_string(),
        }
        .into_parsed()
        .unwrap_err();

        assert_eq!(err, ValenceError::PIDErrorOver)
    }
}
