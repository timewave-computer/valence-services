use auction_package::Pair;
use cosmwasm_schema::cw_serde;
use cosmwasm_std::{Addr, Api, Decimal, Env, MessageInfo, Timestamp, Uint128};
use cw_utils::{must_pay, Expiration};
use std::borrow::Borrow;
use std::hash::Hash;
use std::{collections::HashSet, hash::Hasher, str::FromStr};
use valence_macros::valence_service_execute_msgs;

use crate::{error::ValenceError, helpers::OptionalField, signed_decimal::SignedDecimal};

/// Rebalancer execute msgs.
#[valence_service_execute_msgs]
#[cw_serde]
pub enum RebalancerExecuteMsg<A = RebalancerData, B = RebalancerUpdateData> {
    Admin(RebalancerAdminMsg),
    SystemRebalance { limit: Option<u64> },
    ApproveAdminChange,
}

#[cw_serde]
pub enum RebalancerAdminMsg {
    UpdateSystemStatus {
        status: SystemRebalanceStatus,
    },
    UpdateDenomWhitelist {
        to_add: Vec<String>,
        to_remove: Vec<String>,
    },
    UpdateBaseDenomWhitelist {
        to_add: Vec<BaseDenom>,
        to_remove: Vec<String>,
    },
    UpdateServicesManager {
        addr: String,
    },
    UpdateAuctionsManager {
        addr: String,
    },
    UpdateCyclePeriod {
        period: u64,
    },
    UpdateFess {
        fees: ServiceFeeConfig,
    },
    StartAdminChange {
        addr: String,
        expiration: Expiration,
    },
    CancelAdminChange,
}

#[cw_serde]
pub struct RebalancerData {
    /// The trustee address that can pause/resume the service
    pub trustee: Option<String>,
    /// Base denom we will be calculating everything based on
    pub base_denom: String,
    /// List of targets to rebalance for this account
    pub targets: HashSet<Target>,
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
    pub targets: HashSet<Target>,
    pub pid: Option<PID>,
    pub max_limit_bps: Option<u64>, // BPS
    pub target_override_strategy: Option<TargetOverrideStrategy>,
}

impl RebalancerData {
    pub fn to_config(self, api: &dyn Api) -> Result<RebalancerConfig, ValenceError> {
        let max_limit = if let Some(max_limit) = self.max_limit_bps {
            // Suggested by clippy to check for a range of 1-10000
            if !(1..=10000).contains(&max_limit) {
                return Err(ValenceError::InvalidMaxLimitRange);
            }

            Decimal::bps(max_limit)
        } else {
            Decimal::one()
        };

        let has_min_balance = self.targets.iter().any(|t| t.min_balance.is_some());
        let trustee = self.trustee.map(|a| api.addr_validate(&a)).transpose()?;

        Ok(RebalancerConfig {
            trustee,
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
    /// the address that can pause and resume the service
    pub trustee: Option<Addr>,
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

#[cw_serde]
pub struct PauseData {
    pub pauser: Addr,
    pub reason: PauseReason,
    pub config: RebalancerConfig,
}

impl PauseData {
    pub fn new(pauser: Addr, reason: String, config: &RebalancerConfig) -> Self {
        Self {
            pauser,
            reason: PauseReason::AccountReason(reason),
            config: config.clone(),
        }
    }

    pub fn new_empty_balance(env: &Env, config: &RebalancerConfig) -> Self {
        Self {
            pauser: env.contract.address.clone(),
            reason: PauseReason::EmptyBalance,
            config: config.clone(),
        }
    }
}

#[cw_serde]
pub enum PauseReason {
    /// This reason can only be called if the rebalancer is pausing the account because it
    /// has an empty balance.
    EmptyBalance,
    /// This reason is given by the user/account, he might forget why he paused the account
    /// this will remind him of it.
    AccountReason(String),
}

/// The strategy we will use when overriding targets
#[cw_serde]
pub enum TargetOverrideStrategy {
    Proportional,
    Priority,
}

#[cw_serde]
pub enum SystemRebalanceStatus {
    NotStarted {
        cycle_start: Timestamp,
    },
    Processing {
        cycle_started: Timestamp,
        start_from: Addr,
        prices: Vec<(Pair, Decimal)>,
    },
    Finished {
        next_cycle: Timestamp,
    },
}

/// The target struct that holds all info about a single denom target
#[derive(
    ::cosmwasm_schema::serde::Serialize,
    ::cosmwasm_schema::serde::Deserialize,
    ::std::clone::Clone,
    ::std::fmt::Debug,
    ::cosmwasm_schema::schemars::JsonSchema,
)]
#[allow(clippy::derive_partial_eq_without_eq)] // Allow users of `#[cw_serde]` to not implement Eq without clippy complaining
#[serde(deny_unknown_fields, crate = "::cosmwasm_schema::serde")]
#[schemars(crate = "::cosmwasm_schema::schemars")]
#[derive(Eq)]
pub struct Target {
    /// The name of the denom
    pub denom: String,
    /// The percentage of the total balance we want to have in this denom
    pub bps: u64,
    /// The minimum balance the account should hold for this denom.
    /// Can only be a single one for an account
    pub min_balance: Option<Uint128>,
}

impl PartialEq for Target {
    fn eq(&self, other: &Target) -> bool {
        self.denom == other.denom
    }
}

impl Hash for Target {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.denom.hash(state);
    }
}

impl Borrow<String> for Target {
    fn borrow(&self) -> &String {
        &self.denom
    }
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
    fn from(value: Target) -> Self {
        ParsedTarget {
            denom: value.denom,
            percentage: Decimal::bps(value.bps),
            min_balance: value.min_balance,
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

#[derive(
    ::cosmwasm_schema::serde::Serialize,
    ::cosmwasm_schema::serde::Deserialize,
    ::std::clone::Clone,
    ::std::fmt::Debug,
    ::cosmwasm_schema::schemars::JsonSchema,
)]
#[allow(clippy::derive_partial_eq_without_eq)] // Allow users of `#[cw_serde]` to not implement Eq without clippy complaining
#[serde(deny_unknown_fields, crate = "::cosmwasm_schema::serde")]
#[schemars(crate = "::cosmwasm_schema::schemars")]
#[derive(Eq)]
pub struct BaseDenom {
    pub denom: String,
    pub min_balance_limit: Uint128,
}

impl BaseDenom {
    pub fn new_empty(denom: impl Into<String>) -> Self {
        Self {
            denom: denom.into(),
            min_balance_limit: Uint128::zero(),
        }
    }
}

impl PartialEq for BaseDenom {
    fn eq(&self, other: &BaseDenom) -> bool {
        self.denom == other.denom
    }
}

impl Hash for BaseDenom {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.denom.hash(state);
    }
}

impl Borrow<String> for BaseDenom {
    fn borrow(&self) -> &String {
        &self.denom
    }
}

#[cw_serde]
pub struct ServiceFeeConfig {
    pub denom: String,
    pub register_fee: Uint128,
    pub resume_fee: Uint128,
}

impl ServiceFeeConfig {
    pub fn verify_registeration_fee_paid(&self, info: &MessageInfo) -> Result<(), ValenceError> {
        if !self.register_fee.is_zero() {
            let paid = must_pay(info, &self.denom).map_err(|_| {
                ValenceError::MustPayRegisterationFee(
                    self.register_fee.to_string(),
                    self.denom.clone(),
                )
            })?;

            if self.register_fee != paid {
                return Err(ValenceError::MustPayRegisterationFee(
                    self.register_fee.to_string(),
                    self.denom.clone(),
                ));
            }
        }

        Ok(())
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
