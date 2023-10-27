use auction_package::helpers::ChainHaltConfig;
use cosmwasm_std::{BlockInfo, Decimal, Env, Uint128};

use crate::state::ActiveAuction;

pub fn calc_price(terms: &ActiveAuction, curr_height: u64) -> Decimal {
    let block_diff = Decimal::from_atomics(terms.end_block - terms.start_block, 0).unwrap();
    let price_diff = terms.start_price - terms.end_price;

    let price_per_block = price_diff / block_diff;
    let block_passed = Decimal::from_atomics(curr_height - terms.start_block, 0).unwrap();

    terms.start_price - (price_per_block * block_passed)
}

/// Calc how much of pair.0 to send (bought amount) and how much pair.1 to refund (leftover)
pub fn calc_buy_amount(price: Decimal, amount: Uint128) -> (Uint128, Uint128) {
    let amount = Decimal::from_atomics(amount, 0).unwrap();

    let buy_amount = amount / price;
    let buy_floor = buy_amount.floor();
    let leftover = (amount - (buy_floor * price)).to_uint_floor();

    (buy_floor.to_uint_floor(), leftover)
}

/// Check the diff of blocks and time to see if we had a chain halt of around our time_cap
pub fn is_chain_halted(env: &Env, check_block: &BlockInfo, halt_config: &ChainHaltConfig) -> bool {
    let block_diff = Uint128::from(env.block.height - check_block.height);
    let time_diff = (env.block.time.seconds() - check_block.time.seconds()) as u128;

    let avg_time_passed = (block_diff * halt_config.block_avg).u128();

    // Chain halted for at least 4 hours
    if time_diff > avg_time_passed + halt_config.cap {
        return true;
    }
    false
}
