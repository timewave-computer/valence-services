use cosmwasm_std::{Decimal, Uint128};
use valence_package::services::rebalancer::ParsedTarget;

pub const TRADE_HARD_LIMIT: Decimal = Decimal::raw(5_u128);

pub(crate) type TradesTuple = (Vec<TargetHelper>, Vec<TargetHelper>);

/// Helper struct for our calculation,
/// it holds the target as well as price, balance, input and the amount we need to trade
#[derive(Debug, Clone)]
pub struct TargetHelper {
    /// our target
    pub target: ParsedTarget,
    /// The price of this denom to base_denom
    /// if this target is a base_denom, the price will be 1
    pub price: Decimal,
    /// The current balance amount of this denom
    pub balance_amount: Uint128,
    /// The current balance value, calculated by balance_amount / price
    pub balance_value: Decimal,
    /// The value we need to trade
    /// can either be to sell or to buy, depends on the calculation
    pub value_to_trade: Decimal,
    /// The minimum value we can send to the auction
    pub auction_min_amount: Decimal,
}
