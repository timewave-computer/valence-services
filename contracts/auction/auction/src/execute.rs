use auction_package::{
    states::{ADMIN, MIN_AUCTION_AMOUNT, TWAP_PRICES},
    Price, CLOSEST_TO_ONE_POSSIBLE,
};
use cosmwasm_std::{
    coin, Addr, BankMsg, Coin, CosmosMsg, Decimal, DepsMut, Env, Event, MessageInfo, Response,
    Uint128,
};
use cw_storage_plus::Bound;
use cw_utils::must_pay;

use crate::{
    contract::TWAP_PRICE_MAX_LEN,
    error::ContractError,
    helpers::{calc_buy_amount, calc_price, is_chain_halted},
    state::{
        ActiveAuctionStatus, ACTIVE_AUCTION, AUCTION_CONFIG, AUCTION_FUNDS, AUCTION_FUNDS_SUM,
        AUCTION_IDS,
    },
};

pub(crate) fn auction_funds(
    deps: DepsMut,
    info: &MessageInfo,
    sender: Addr,
) -> Result<Response, ContractError> {
    let config = AUCTION_CONFIG.load(deps.storage)?;
    let admin = ADMIN.load(deps.storage)?;

    if config.is_paused {
        return Err(ContractError::AuctionIsPaused);
    }

    let funds = must_pay(info, &config.pair.0)?;
    let min_amount = match MIN_AUCTION_AMOUNT.query(&deps.querier, admin, config.pair.0)? {
        Some(amount) => Ok(amount.send),
        None => Err(ContractError::NoTokenMinAmount),
    }?;

    if funds < min_amount {
        return Err(ContractError::AuctionAmountTooLow(min_amount));
    }

    let next_auction_id: u64 = AUCTION_IDS.load(deps.storage)?.next;

    // Update funds of the sender for next auction
    AUCTION_FUNDS.update(
        deps.storage,
        (next_auction_id, sender),
        |amount| -> Result<Uint128, ContractError> {
            match amount {
                Some(amount) => Ok(amount.checked_add(funds)?),
                None => Ok(funds),
            }
        },
    )?;

    // update the sum of the next auction
    AUCTION_FUNDS_SUM.update(
        deps.storage,
        next_auction_id,
        |amount| -> Result<Uint128, ContractError> {
            match amount {
                Some(amount) => Ok(amount.checked_add(funds)?),
                None => Ok(funds),
            }
        },
    )?;

    Ok(Response::default())
}

pub fn withdraw_funds(deps: DepsMut, sender: Addr) -> Result<Response, ContractError> {
    let config = AUCTION_CONFIG.load(deps.storage)?;

    let mut send_funds: Coin = coin(0_u128, config.pair.0);
    let auction_ids = AUCTION_IDS.load(deps.storage)?;

    let funds_amount = AUCTION_FUNDS
        .load(deps.storage, (auction_ids.next, sender.clone()))
        .unwrap_or(Uint128::zero());

    if !funds_amount.is_zero() {
        send_funds.amount += funds_amount;
        AUCTION_FUNDS.remove(deps.storage, (auction_ids.next, sender.clone()));
        AUCTION_FUNDS_SUM.update(
            deps.storage,
            auction_ids.next,
            |sum| -> Result<Uint128, ContractError> {
                Ok(sum.unwrap_or(Uint128::zero()).checked_sub(funds_amount)?)
            },
        )?;
    }

    if send_funds.amount.is_zero() {
        return Err(ContractError::NoFundsToWithdraw);
    }

    let bank_msg = BankMsg::Send {
        to_address: sender.to_string(),
        amount: vec![send_funds.clone()],
    };

    Ok(Response::default()
        .add_message(bank_msg)
        .add_event(Event::new("withdraw-funds").add_attribute("amount", send_funds.to_string())))
}

pub fn do_bid(deps: DepsMut, info: &MessageInfo, env: &Env) -> Result<Response, ContractError> {
    // Verify we have an active auction, else error out
    let mut active_auction = ACTIVE_AUCTION.load(deps.storage)?;

    // Verify auction is not finished
    match active_auction.status {
        ActiveAuctionStatus::Started => Ok(()),
        _ => Err(ContractError::AuctionFinished),
    }?;

    // Verify auction started
    if active_auction.start_block > env.block.height {
        return Err(ContractError::AuctionNotStarted(active_auction.start_block));
    }

    // The end block is smaller then the current height so auction is finished
    if active_auction.end_block < env.block.height {
        return Err(ContractError::AuctionFinished);
    }

    let config = AUCTION_CONFIG.load(deps.storage)?;

    if config.is_paused {
        return Err(ContractError::AuctionIsPaused);
    }

    let sent_funds = must_pay(info, &config.pair.1)?;

    let (buy_amount, leftover_amount) = if is_chain_halted(
        env,
        &active_auction.last_checked_block,
        &config.chain_halt_config,
    ) {
        active_auction.status = ActiveAuctionStatus::Finished;
        (Uint128::zero(), sent_funds)
    } else {
        let curr_price = calc_price(&active_auction, env.block.height);
        let (buy_amount, mut send_leftover) = calc_buy_amount(curr_price, sent_funds);

        let send_amount = match active_auction.available_amount.checked_sub(buy_amount) {
            Ok(available_amount) => {
                if available_amount.is_zero() {
                    active_auction.status = ActiveAuctionStatus::Finished;
                }

                active_auction.available_amount = available_amount;
                active_auction.resolved_amount += sent_funds - send_leftover;

                Ok::<Uint128, ContractError>(buy_amount)
            }
            // If we reach here, it means that we have sold all of the available amount
            // so we can finish the auction
            Err(_) => {
                let to_refund =
                    Decimal::from_atomics(buy_amount - active_auction.available_amount, 0)?;
                let new_leftover = (to_refund * curr_price).to_uint_floor();
                send_leftover += new_leftover;

                // Set the buy_amount to be whatever amount we have left, because we know the bidder over paid
                let buy_amount = active_auction.available_amount;

                active_auction.resolved_amount += sent_funds - send_leftover;
                active_auction.available_amount = Uint128::zero();
                active_auction.status = ActiveAuctionStatus::Finished;

                Ok(buy_amount)
            }
        }?;
        (send_amount, send_leftover)
    };

    let mut send_funds: Vec<Coin> = vec![];

    if !leftover_amount.is_zero() {
        send_funds.push(coin(leftover_amount.u128(), config.pair.1.clone()));
    }

    if !buy_amount.is_zero() {
        send_funds.push(coin(buy_amount.u128(), config.pair.0));
    }

    let response = if !send_funds.is_empty() {
        let bank_msg = BankMsg::Send {
            to_address: info.sender.to_string(),
            amount: send_funds,
        };
        Response::default().add_message(bank_msg)
    } else {
        Response::default()
    };

    active_auction.last_checked_block = env.block.clone();
    ACTIVE_AUCTION.save(deps.storage, &active_auction)?;

    Ok(
        response
            .add_attribute("bought_amount", buy_amount) // pair.0 amount we sent
            .add_attribute("refunded", leftover_amount), // pair.1 amount we refunded
    )
}

pub fn finish_auction(deps: DepsMut, env: &Env, limit: u64) -> Result<Response, ContractError> {
    let mut active_auction = ACTIVE_AUCTION.load(deps.storage)?;

    if active_auction.status == ActiveAuctionStatus::Started
        && active_auction.end_block > env.block.height
        && !active_auction.available_amount.is_zero()
    {
        return Err(ContractError::AuctionStillGoing);
    }

    let (start_from, mut total_sent_sold_token, mut total_sent_bought_token) = match active_auction
        .status
    {
        ActiveAuctionStatus::CloseAuction(addr, total_sent_sold_token, total_sent_bought_token) => {
            Ok((addr, total_sent_sold_token, total_sent_bought_token))
        }
        ActiveAuctionStatus::Finished | ActiveAuctionStatus::Started => {
            Ok((None, Uint128::zero(), Uint128::zero()))
        }
        ActiveAuctionStatus::AuctionClosed => Err(ContractError::AuctionClosed),
    }?;

    let config = AUCTION_CONFIG.load(deps.storage)?;
    let curr_auction_id = AUCTION_IDS.load(deps.storage)?.curr;
    let mut last_resolved = start_from.clone();
    let start_from = start_from.map(Bound::exclusive);
    let mut total_resolved = 0;

    let mut bank_msgs: Vec<CosmosMsg> = vec![];

    AUCTION_FUNDS
        .prefix(curr_auction_id)
        .range(
            deps.storage,
            start_from,
            None,
            cosmwasm_std::Order::Ascending,
        )
        .take(limit as usize)
        .try_for_each(|res| -> Result<(), ContractError> {
            total_resolved += 1;
            let (addr, amount) = res?;
            let mut send_funds: Vec<Coin> = vec![];

            if active_auction.resolved_amount.is_zero() {
                // We didn't sell anything, so refund
                send_funds.push(coin(amount.u128(), &config.pair.0));
                total_sent_sold_token += amount;
            } else {
                // We sold something, calculate only what we sold
                let perc_of_total = Decimal::from_atomics(amount, 0)?
                    / Decimal::from_atomics(active_auction.total_amount, 0)?;
                let to_send_amount =
                    Decimal::from_atomics(active_auction.resolved_amount, 0)? * perc_of_total;

                // TODO: Verify this is correct
                let to_send_amount = if to_send_amount - to_send_amount.floor()
                    >= Decimal::bps(CLOSEST_TO_ONE_POSSIBLE)
                {
                    to_send_amount.to_uint_ceil()
                } else {
                    to_send_amount.to_uint_floor()
                };

                total_sent_bought_token += to_send_amount;
                send_funds.push(coin(to_send_amount.u128(), &config.pair.1));

                // If we still have available amount, we refund based on the perc from total provided
                if !active_auction.available_amount.is_zero() {
                    let to_send_amount =
                        (Decimal::from_atomics(active_auction.available_amount, 0)?
                            * perc_of_total)
                            .to_uint_floor();
                    total_sent_sold_token += to_send_amount;
                    send_funds.push(coin(to_send_amount.u128(), &config.pair.0));
                }
            }

            let send_msg = BankMsg::Send {
                to_address: addr.to_string(),
                amount: send_funds,
            };
            bank_msgs.push(send_msg.into());

            last_resolved = Some(addr);
            Ok(())
        })?;

    // If we looped over less than our limit, it means we resolved everything
    let (status, price, is_closed) = if total_resolved < limit {
        // calculate if we have leftover from rounding and add it to the next auction
        let leftover_sold_token = active_auction
            .available_amount
            .checked_sub(total_sent_sold_token)?;
        let leftover_bought_token = active_auction
            .resolved_amount
            .checked_sub(total_sent_bought_token)?;

        active_auction.leftovers[0] = leftover_sold_token;
        active_auction.leftovers[1] = leftover_bought_token;

        // Update twap price if we have something sold
        let sold_amount = active_auction
            .total_amount
            .checked_sub(active_auction.available_amount)?;

        let price = if !active_auction.total_amount.is_zero() && !sold_amount.is_zero() {
            let avg_price = Decimal::from_atomics(active_auction.resolved_amount, 0)?
                .checked_div(Decimal::from_atomics(sold_amount, 0)?)?;

            let mut prices = TWAP_PRICES.load(deps.storage)?;

            // if we have the needed amount of prices already, remove the last one first
            if prices.len() >= TWAP_PRICE_MAX_LEN as usize {
                prices.pop_back();
            }

            prices.push_front(Price {
                price: avg_price,
                time: env.block.time,
            });

            TWAP_PRICES.save(deps.storage, &prices)?;
            avg_price.to_string()
        } else {
            "0".to_string()
        };

        (ActiveAuctionStatus::AuctionClosed, price, true)
    } else {
        (
            ActiveAuctionStatus::CloseAuction(
                last_resolved,
                total_sent_sold_token,
                total_sent_bought_token,
            ),
            "0".to_string(),
            false,
        )
    };

    active_auction.status = status;
    ACTIVE_AUCTION.save(deps.storage, &active_auction)?;

    Ok(Response::default().add_messages(bank_msgs).add_event(
        Event::new("close-auction")
            .add_attribute("is_closed", is_closed.to_string())
            .add_attribute("price", price),
    ))
}

pub fn clean_auction(deps: DepsMut) -> Result<Response, ContractError> {
    let active_auction = ACTIVE_AUCTION.load(deps.storage)?;

    match active_auction.status {
        ActiveAuctionStatus::AuctionClosed => Ok::<_, ContractError>(()),
        _ => return Err(ContractError::AuctionNotClosed),
    }?;

    let curr_auction_id = AUCTION_IDS.load(deps.storage)?.curr;

    // Clean the funds at the id of ended auction
    AUCTION_FUNDS
        .prefix(curr_auction_id)
        .clear(deps.storage, None);
    // Clean the funds sum
    AUCTION_FUNDS_SUM.remove(deps.storage, curr_auction_id);

    Ok(Response::default())
}
