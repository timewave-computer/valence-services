use auction::state::{ActiveAuction, ActiveAuctionStatus};
use auction_package::{error::AuctionError, states::TWAP_PRICES};
use cosmwasm_std::{coin, coins, testing::mock_env, Addr, Decimal, Timestamp, Uint128};
use cw_multi_test::Executor;

use crate::suite::suite::{Suite, DAY, DEFAULT_BLOCK_TIME, DEFAULT_NTRN_PRICE_BPS};

#[test]
fn test_open_auction() {
    let mut suite = Suite::default();
    let funds = coins(1000_u128, suite.pair.0.clone());
    suite.auction_funds(
        suite.get_account_addr(0),
        suite
            .auction_addrs
            .get(&suite.pair.clone().into())
            .unwrap()
            .clone(),
        &funds,
    );

    suite
        .start_auction(
            suite.pair.clone(),
            Some(mock_env().block.height),
            mock_env().block.height + 1000,
        )
        .unwrap();

    let active_auction = suite.query_auction_details(suite.get_default_auction_addr());

    assert_eq!(
        active_auction,
        ActiveAuction {
            status: auction::state::ActiveAuctionStatus::Started,
            start_block: mock_env().block.height,
            end_block: mock_env().block.height + 1000,
            start_price: Decimal::bps(DEFAULT_NTRN_PRICE_BPS)
                + Decimal::bps(DEFAULT_NTRN_PRICE_BPS) * Decimal::bps(2000),
            end_price: Decimal::bps(DEFAULT_NTRN_PRICE_BPS)
                - Decimal::bps(DEFAULT_NTRN_PRICE_BPS) * Decimal::bps(2000),
            available_amount: funds[0].amount,
            resolved_amount: Uint128::zero(),
            total_amount: funds[0].amount,
            leftovers: [Uint128::zero(), Uint128::zero()],
            last_checked_block: mock_env().block
        }
    );
}

#[test]
fn test_auction_prices() {
    let mut suite = Suite::default();
    let funds = coins(1000_u128, suite.pair.0.clone());
    suite.auction_funds(
        suite.get_account_addr(0),
        suite
            .auction_addrs
            .get(&suite.pair.clone().into())
            .unwrap()
            .clone(),
        &funds,
    );

    suite
        .start_auction(
            suite.pair.clone(),
            Some(mock_env().block.height),
            mock_env().block.height + 1000,
        )
        .unwrap();

    let price_per_block = suite.calc_price_per_block(suite.get_default_auction_addr());
    let start_price = Decimal::bps(DEFAULT_NTRN_PRICE_BPS)
        + Decimal::bps(DEFAULT_NTRN_PRICE_BPS) * Decimal::bps(2000);

    // Update 100 block
    suite.update_block(100);

    let price = suite.query_auction_price(suite.get_default_auction_addr());
    assert_eq!(
        price.price,
        start_price - price_per_block * Decimal::from_atomics(100_u128, 0).unwrap()
    );

    // Update 150 more block total 250
    suite.update_block(150);

    let price = suite.query_auction_price(suite.get_default_auction_addr());
    assert_eq!(
        price.price,
        start_price - price_per_block * Decimal::from_atomics(250_u128, 0).unwrap()
    );

    // Update 250 more block total 500
    suite.update_block(250);

    let price = suite.query_auction_price(suite.get_default_auction_addr());
    assert_eq!(
        price.price,
        start_price - price_per_block * Decimal::from_atomics(500_u128, 0).unwrap()
    );

    // Update 123 more block total 623
    suite.update_block(123);

    let price = suite.query_auction_price(suite.get_default_auction_addr());
    assert_eq!(
        price.price,
        start_price - price_per_block * Decimal::from_atomics(623_u128, 0).unwrap()
    );
}

#[test]
fn test_bid() {
    let mut suite = Suite::default();
    let funds = coins(1000_u128, suite.pair.0.clone());
    suite.auction_funds(
        suite.get_account_addr(0),
        suite
            .auction_addrs
            .get(&suite.pair.clone().into())
            .unwrap()
            .clone(),
        &funds,
    );

    suite
        .start_auction(
            suite.pair.clone(),
            Some(mock_env().block.height),
            mock_env().block.height + 1000,
        )
        .unwrap();

    let price_per_block = suite.calc_price_per_block(suite.get_default_auction_addr());
    let start_price = Decimal::bps(DEFAULT_NTRN_PRICE_BPS)
        + Decimal::bps(DEFAULT_NTRN_PRICE_BPS) * Decimal::bps(2000);

    suite.update_block(100);
    let block_price = start_price - price_per_block * Decimal::from_atomics(100_u128, 0).unwrap();

    // buy 250 atom
    let ntrn_to_send = (Decimal::from_atomics(250_u128, 0).unwrap() * block_price).to_uint_ceil();
    suite
        .do_bid(
            suite.pair.clone(),
            coin(ntrn_to_send.u128(), suite.pair.1.clone()),
        )
        .unwrap();

    let active_auction = suite.query_auction_details(suite.get_default_auction_addr());
    assert_eq!(
        active_auction.available_amount,
        active_auction.total_amount - Uint128::from(250_u128)
    );
    let mm_balance = suite
        .app
        .wrap()
        .query_balance(suite.mm.clone(), suite.pair.1.clone())
        .unwrap();
    assert_eq!(
        Uint128::from(1000000000_u128) - mm_balance.amount,
        active_auction.resolved_amount
    );

    // Do another bid
    suite.update_block(113);
    let block_price = start_price - price_per_block * Decimal::from_atomics(213_u128, 0).unwrap();

    // buy 123 atom
    let ntrn_to_send = (Decimal::from_atomics(123_u128, 0).unwrap() * block_price).to_uint_ceil();
    suite
        .do_bid(
            suite.pair.clone(),
            coin(ntrn_to_send.u128(), suite.pair.1.clone()),
        )
        .unwrap();

    let active_auction = suite.query_auction_details(suite.get_default_auction_addr());
    assert_eq!(
        active_auction.available_amount,
        active_auction.total_amount - Uint128::from(250_u128) - Uint128::from(123_u128)
    );
    let mm_balance = suite
        .app
        .wrap()
        .query_balance(suite.mm.clone(), suite.pair.1.clone())
        .unwrap();
    assert_eq!(
        Uint128::from(1000000000_u128) - mm_balance.amount,
        active_auction.resolved_amount
    );

    // Do another bid
    suite.update_block(700);
    let block_price = start_price - price_per_block * Decimal::from_atomics(913_u128, 0).unwrap();

    // try to buy 1000 atom (should buy everything that is left, and return the leftovers)
    let ntrn_to_send = (Decimal::from_atomics(1000_u128, 0).unwrap() * block_price).to_uint_ceil();
    suite
        .do_bid(
            suite.pair.clone(),
            coin(ntrn_to_send.u128(), suite.pair.1.clone()),
        )
        .unwrap();

    let active_auction = suite.query_auction_details(suite.get_default_auction_addr());
    assert_eq!(active_auction.available_amount, Uint128::zero());
    let mm_balance = suite
        .app
        .wrap()
        .query_balance(suite.mm.clone(), suite.pair.1.clone())
        .unwrap();
    assert_eq!(
        Uint128::from(1000000000_u128) - mm_balance.amount,
        active_auction.resolved_amount
    );
    assert_eq!(active_auction.status, ActiveAuctionStatus::Finished)
}

#[test]
fn test_exact_bid() {
    let mut suite = Suite::default();
    let funds = coins(1000_u128, suite.pair.0.clone());
    suite.auction_funds(
        suite.get_account_addr(0),
        suite
            .auction_addrs
            .get(&suite.pair.clone().into())
            .unwrap()
            .clone(),
        &funds,
    );

    suite
        .start_auction(
            suite.pair.clone(),
            Some(mock_env().block.height),
            mock_env().block.height + 1000,
        )
        .unwrap();

    let price_per_block = suite.calc_price_per_block(suite.get_default_auction_addr());
    let start_price = Decimal::bps(DEFAULT_NTRN_PRICE_BPS)
        + Decimal::bps(DEFAULT_NTRN_PRICE_BPS) * Decimal::bps(2000);

    suite.update_block(700);
    let block_price = start_price - price_per_block * Decimal::from_atomics(700_u128, 0).unwrap();

    // try to buy 1000 atom, should buy everything and finish the auction
    let ntrn_to_send = (Decimal::from_atomics(1000_u128, 0).unwrap() * block_price).to_uint_ceil();
    suite
        .do_bid(
            suite.pair.clone(),
            coin(ntrn_to_send.u128(), suite.pair.1.clone()),
        )
        .unwrap();

    let active_auction = suite.query_auction_details(suite.get_default_auction_addr());
    assert_eq!(active_auction.available_amount, Uint128::zero());
    let mm_balance = suite
        .app
        .wrap()
        .query_balance(suite.mm.clone(), suite.pair.1.clone())
        .unwrap();
    assert_eq!(
        Uint128::from(1000000000_u128) - mm_balance.amount,
        ntrn_to_send
    );
    assert_eq!(active_auction.status, ActiveAuctionStatus::Finished)
}

#[test]
fn test_overflow_bid() {
    let mut suite = Suite::default();
    let funds = coins(1000_u128, suite.pair.0.clone());
    suite.auction_funds(
        suite.get_account_addr(0),
        suite
            .auction_addrs
            .get(&suite.pair.clone().into())
            .unwrap()
            .clone(),
        &funds,
    );

    suite
        .start_auction(
            suite.pair.clone(),
            Some(mock_env().block.height),
            mock_env().block.height + 1000,
        )
        .unwrap();

    let price_per_block = suite.calc_price_per_block(suite.get_default_auction_addr());
    let start_price = Decimal::bps(DEFAULT_NTRN_PRICE_BPS)
        + Decimal::bps(DEFAULT_NTRN_PRICE_BPS) * Decimal::bps(2000);

    suite.update_block(700);
    let block_price = start_price - price_per_block * Decimal::from_atomics(700_u128, 0).unwrap();

    // try to buy 1100 atom, should buy everything with leftover
    let ntrn_to_send = (Decimal::from_atomics(1100_u128, 0).unwrap() * block_price).to_uint_ceil();
    let ntrn_to_buy_all =
        (Decimal::from_atomics(1000_u128, 0).unwrap() * block_price).to_uint_ceil();
    suite
        .do_bid(
            suite.pair.clone(),
            coin(ntrn_to_send.u128(), suite.pair.1.clone()),
        )
        .unwrap();

    let active_auction = suite.query_auction_details(suite.get_default_auction_addr());
    assert_eq!(active_auction.available_amount, Uint128::zero());
    let mm_balance = suite
        .app
        .wrap()
        .query_balance(suite.mm.clone(), suite.pair.1.clone())
        .unwrap();
    assert_eq!(
        Uint128::from(1000000000_u128) - mm_balance.amount,
        ntrn_to_buy_all
    );
    assert_eq!(active_auction.status, ActiveAuctionStatus::Finished)
}

#[test]
fn test_chain_halt() {
    let avg_block_time = ((100.0 * 3.6) as f32).floor() as u64;
    let mut suite = Suite::default();
    let funds = coins(1000_u128, suite.pair.0.clone());
    suite.auction_funds(
        suite.get_account_addr(0),
        suite
            .auction_addrs
            .get(&suite.pair.clone().into())
            .unwrap()
            .clone(),
        &funds,
    );

    suite
        .start_auction(
            suite.pair.clone(),
            Some(mock_env().block.height),
            mock_env().block.height + 1000,
        )
        .unwrap();

    suite.app.update_block(|b| {
        b.height += 100;
        b.time = Timestamp::from_seconds(b.time.seconds() + avg_block_time);
    });

    // Should pass, no chain halts
    suite.do_full_bid(1_u128);

    // Chain halted for 1 hour (should be successful)
    suite.app.update_block(|b| {
        b.height += 100;
        b.time = Timestamp::from_seconds(b.time.seconds() + avg_block_time + (60 * 60 * 4 + 1));
    });

    // Should return 0 as the bought amount
    let res = suite.do_full_bid(1_u128);
    let amount_bought = suite
        .get_attr_value(&res, "bought_amount")
        .unwrap()
        .parse::<u128>()
        .unwrap();
    let amount_refunded = suite
        .get_attr_value(&res, "refunded")
        .unwrap()
        .parse::<u128>()
        .unwrap();

    assert_eq!(amount_bought, 0_u128);
    assert_eq!(amount_refunded, 2_u128);
}

#[test]
fn test_not_admin() {
    let mut suite = Suite::default();
    let funds = coins(1000_u128, suite.pair.0.clone());
    suite.auction_funds(
        suite.get_account_addr(0),
        suite
            .auction_addrs
            .get(&suite.pair.clone().into())
            .unwrap()
            .clone(),
        &funds,
    );

    suite
        .start_auction(
            suite.pair.clone(),
            Some(mock_env().block.height),
            mock_env().block.height + 1000,
        )
        .unwrap();

    // Try to pause and auction not as admin (not manager)
    let err = suite
        .app
        .execute_contract(
            Addr::unchecked("not_admin"),
            suite.get_default_auction_addr(),
            &auction::msg::ExecuteMsg::Admin(Box::new(auction::msg::AdminMsgs::PauseAuction)),
            &[],
        )
        .unwrap_err();
    let err = err.source().unwrap().to_string();
    assert_eq!(err, AuctionError::NotAdmin.to_string());
}

#[test]
fn test_clean_auction() {
    let mut suite = Suite::default();
    let funds = coins(1000_u128, suite.pair.0.clone());

    suite.finalize_auction(&funds);

    let auction_ids = auction::state::AUCTION_IDS
        .query(&suite.app.wrap(), suite.get_default_auction_addr())
        .unwrap();
    let funds = auction::state::AUCTION_FUNDS
        .query(
            &suite.app.wrap(),
            suite.get_default_auction_addr(),
            (auction_ids.curr, suite.get_account_addr(0)),
        )
        .unwrap();
    let funds_sum = auction::state::AUCTION_FUNDS_SUM
        .query(
            &suite.app.wrap(),
            suite.get_default_auction_addr(),
            auction_ids.curr,
        )
        .unwrap();
    assert!(funds.is_some());
    assert!(funds_sum.is_some());

    // do the cleanup
    suite.clean_last_auction(suite.get_default_auction_addr());
    let funds = auction::state::AUCTION_FUNDS
        .query(
            &suite.app.wrap(),
            suite.get_default_auction_addr(),
            (auction_ids.curr, suite.get_account_addr(0)),
        )
        .unwrap();
    let funds_sum = auction::state::AUCTION_FUNDS_SUM
        .query(
            &suite.app.wrap(),
            suite.get_default_auction_addr(),
            auction_ids.curr,
        )
        .unwrap();
    assert!(funds.is_none());
    assert!(funds_sum.is_none());
}

#[test]
fn test_clean_auction_not_closed() {
    let mut suite = Suite::default();
    let funds = coins(1000_u128, suite.pair.0.clone());

    suite.auction_funds(
        suite.get_account_addr(0),
        suite
            .auction_addrs
            .get(&suite.pair.clone().into())
            .unwrap()
            .clone(),
        &funds,
    );
    suite.start_auction_day(suite.pair.clone()).unwrap();

    let err = suite.clean_last_auction_err(suite.get_default_auction_addr());
    assert_eq!(err, auction::error::ContractError::AuctionNotClosed);
}

#[test]
fn test_auction_paused() {
    let mut suite = Suite::default();
    let funds = coins(1000_u128, suite.pair.0.clone());

    suite.pause_auction(suite.pair.clone());

    let err = suite.auction_funds_err(
        suite.get_account_addr(0),
        suite
            .auction_addrs
            .get(&suite.pair.clone().into())
            .unwrap()
            .clone(),
        &funds,
    );
    assert_eq!(err, auction::error::ContractError::AuctionIsPaused);
}

#[test]
fn test_auction_too_low_amount() {
    let mut suite = Suite::default();
    let funds = coins(1_u128, suite.pair.0.clone());

    let err = suite.auction_funds_err(
        suite.get_account_addr(0),
        suite
            .auction_addrs
            .get(&suite.pair.clone().into())
            .unwrap()
            .clone(),
        &funds,
    );
    assert_eq!(
        err,
        auction::error::ContractError::AuctionAmountTooLow(5_u128.into())
    );
}

#[test]
fn test_auction_twice() {
    let mut suite = Suite::default();
    let funds = coins(500_u128, suite.pair.0.clone());
    suite.auction_funds(
        suite.get_account_addr(0),
        suite
            .auction_addrs
            .get(&suite.pair.clone().into())
            .unwrap()
            .clone(),
        &funds,
    );

    suite.finalize_auction(&funds);
}

#[test]
fn test_bid_before_auction_started() {
    let mut suite = Suite::default();
    let funds = coins(1000_u128, suite.pair.0.clone());
    suite.auction_funds(
        suite.get_account_addr(0),
        suite
            .auction_addrs
            .get(&suite.pair.clone().into())
            .unwrap()
            .clone(),
        &funds,
    );

    let err = suite.do_bid_err(suite.pair.clone(), coin(1000_u128, suite.pair.1.clone()));
    assert_eq!(err, auction::error::ContractError::AuctionFinished);

    suite
        .start_auction(
            suite.pair.clone(),
            Some(suite.app.block_info().height + 1000),
            suite.app.block_info().height + 1000 + (DAY / DEFAULT_BLOCK_TIME),
        )
        .unwrap();

    let err = suite.do_bid_err(suite.pair.clone(), coin(1000_u128, suite.pair.1.clone()));
    assert_eq!(
        err,
        auction::error::ContractError::AuctionNotStarted(suite.app.block_info().height + 1000)
    );

    suite.update_block(1000);

    suite.do_full_bid(10_u128);

    suite.pause_auction(suite.pair.clone());

    let err = suite.do_bid_err(suite.pair.clone(), coin(1000_u128, suite.pair.1.clone()));
    assert_eq!(err, auction::error::ContractError::AuctionIsPaused);
}

#[test]
fn test_closing_still_going_auction() {
    let mut suite = Suite::default();
    let funds = coins(1000_u128, suite.pair.0.clone());
    suite.auction_funds(
        suite.get_account_addr(0),
        suite
            .auction_addrs
            .get(&suite.pair.clone().into())
            .unwrap()
            .clone(),
        &funds,
    );

    suite
        .start_auction(
            suite.pair.clone(),
            None,
            suite.app.block_info().height + DAY / DEFAULT_BLOCK_TIME,
        )
        .unwrap();

    suite.do_full_bid(100_u128);

    let err = suite.close_auction_err(suite.pair.clone(), None);
    assert_eq!(err, auction::error::ContractError::AuctionStillGoing);
}

#[test]
fn test_closing_closed_auction() {
    let mut suite = Suite::default();
    let funds = coins(1000_u128, suite.pair.0.clone());
    suite.finalize_auction(&funds);

    let err = suite.close_auction_err(suite.pair.clone(), None);
    assert_eq!(err, auction::error::ContractError::AuctionClosed);
}

#[test]
fn test_saving_10_twap_prices() {
    let mut suite = Suite::default();
    let funds = coins(10_u128, suite.pair.0.clone());

    // Do 11 auctions
    for i in 0..11 {
        suite.finalize_auction(&funds);
        if i < 3 {
            suite
                .update_price(suite.pair.clone(), Some(Decimal::one()))
                .unwrap();
        } else {
            suite.update_price(suite.pair.clone(), None).unwrap();
        }
    }

    let prices = TWAP_PRICES
        .query(&suite.app.wrap(), suite.get_default_auction_addr())
        .unwrap();
    assert_eq!(prices.len(), 10);
}

#[test]
fn test_change_strategy() {
    let mut suite = Suite::default();
    let strategy = suite.query_auction_strategy(suite.get_default_auction_addr());
    assert_eq!(
        strategy,
        auction_package::AuctionStrategy {
            start_price_perc: 2000,
            end_price_perc: 2000
        }
    );

    suite.update_auction_strategy(
        suite.pair.clone(),
        auction_package::AuctionStrategy {
            start_price_perc: 4000,
            end_price_perc: 4000,
        },
    );

    // Verify the strategy was changed
    let strategy = suite.query_auction_strategy(suite.get_default_auction_addr());
    assert_eq!(
        strategy,
        auction_package::AuctionStrategy {
            start_price_perc: 4000,
            end_price_perc: 4000
        }
    );
}

#[test]
fn test_open_auction_when_paused() {
    let mut suite = Suite::default();

    suite.pause_auction(suite.pair.clone());

    let err = suite.start_auction_day_err(suite.pair.clone());
    assert_eq!(err, auction::error::ContractError::AuctionIsPaused);
}

#[test]
fn test_open_auction_not_closed() {
    let mut suite = Suite::default();
    let funds = coins(10_u128, suite.pair.0.clone());
    suite.auction_funds(
        suite.get_account_addr(0),
        suite
            .auction_addrs
            .get(&suite.pair.clone().into())
            .unwrap()
            .clone(),
        &funds,
    );
    suite.start_auction_day(suite.pair.clone()).unwrap();

    let err = suite.start_auction_day_err(suite.pair.clone());
    assert_eq!(err, auction::error::ContractError::AuctionNotClosed);
}

#[test]
fn test_open_auction_no_funds() {
    let mut suite = Suite::default();

    let err = suite.start_auction_day_err(suite.pair.clone());
    assert_eq!(err, auction::error::ContractError::NoFundsForAuction);
}

#[test]
fn test_auction_modified_strategy_for_price_freshness() {
    let mut suite = Suite::default();
    let funds = coins(10_u128, suite.pair.0.clone());

    suite.finalize_auction(&funds);

    let active_auction = suite.query_auction_details(suite.get_default_auction_addr());
    let price = suite.query_oracle_price(suite.pair.clone()).price;
    assert_eq!(
        active_auction.start_price,
        price + price * Decimal::bps(2000)
    );
    assert_eq!(active_auction.end_price, price - price * Decimal::bps(2000));

    suite.finalize_auction(&funds);
    suite.finalize_auction(&funds);

    suite.update_price(suite.pair.clone(), None).unwrap();

    suite.update_block_cycle();
    suite.update_block_cycle();

    suite.finalize_auction(&funds);

    // we missed 2 days of auction, so our price should multiply by 30% (20% * 1.5)
    let active_auction = suite.query_auction_details(suite.get_default_auction_addr());
    let price = suite.query_oracle_price(suite.pair.clone()).price;
    assert_eq!(
        active_auction.start_price,
        price + price * Decimal::bps(3000)
    );
    assert_eq!(active_auction.end_price, price - price * Decimal::bps(3000));

    suite.update_price(suite.pair.clone(), None).unwrap();

    suite.update_block_cycle();
    suite.update_block_cycle();
    suite.update_block_cycle();

    suite.update_price(suite.pair.clone(), None).unwrap();

    suite.finalize_auction(&funds);

    // we missed 3 days of auction, so our price should multiply by 40% (20% * 2)
    let active_auction = suite.query_auction_details(suite.get_default_auction_addr());
    let price = suite.query_oracle_price(suite.pair.clone()).price;
    assert_eq!(
        active_auction.start_price,
        price + price * Decimal::bps(4000)
    );
    assert_eq!(active_auction.end_price, price - price * Decimal::bps(4000));

    suite.update_price(suite.pair.clone(), None).unwrap();

    suite.finalize_auction(&funds);

    // last auction was yesterday, so back to normal 20%
    let active_auction = suite.query_auction_details(suite.get_default_auction_addr());
    let price = suite.query_oracle_price(suite.pair.clone()).price;
    assert_eq!(
        active_auction.start_price,
        price + price * Decimal::bps(2000)
    );
    assert_eq!(active_auction.end_price, price - price * Decimal::bps(2000));
}

#[test]
fn test_bid_over() {
    let mut suite = Suite::default();
    let funds = coins(100_u128, suite.pair.0.clone());

    let init_mm_balance_0 = suite
        .app
        .wrap()
        .query_balance(suite.mm.clone(), suite.pair.0.clone())
        .unwrap();
    let init_mm_balance_1 = suite
        .app
        .wrap()
        .query_balance(suite.mm.clone(), suite.pair.1.clone())
        .unwrap();

    suite.auction_funds(
        suite.get_account_addr(0),
        suite
            .auction_addrs
            .get(&suite.pair.clone().into())
            .unwrap()
            .clone(),
        &funds,
    );
    suite.start_auction_day(suite.pair.clone()).unwrap();

    suite
        .do_bid(suite.pair.clone(), coin(1000_u128, suite.pair.1.clone()))
        .unwrap();

    let mm_balance_0 = suite
        .app
        .wrap()
        .query_balance(suite.mm.clone(), suite.pair.0.clone())
        .unwrap();
    let mm_balance_1 = suite
        .app
        .wrap()
        .query_balance(suite.mm.clone(), suite.pair.1.clone())
        .unwrap();

    suite.close_auction(suite.pair.clone(), None).unwrap();

    // check we got the funds we bought
    assert!(mm_balance_0.amount > init_mm_balance_0.amount);
    // check we paid the funds
    assert!(init_mm_balance_1.amount > mm_balance_1.amount);
}

#[test]
fn test_open_auction_no_bids() {
    let mut suite = Suite::default();
    let funds = coins(100_u128, suite.pair.0.clone());

    suite.auction_funds(
        suite.get_account_addr(0),
        suite
            .auction_addrs
            .get(&suite.pair.clone().into())
            .unwrap()
            .clone(),
        &funds,
    );
    suite.start_auction_day(suite.pair.clone()).unwrap();
    suite.update_block_cycle();
    suite.close_auction(suite.pair.clone(), None).unwrap();

    // Auction funds again
    suite.auction_funds(
        suite.get_account_addr(0),
        suite
            .auction_addrs
            .get(&suite.pair.clone().into())
            .unwrap()
            .clone(),
        &funds,
    );

    suite.start_auction_day(suite.pair.clone()).unwrap();

    let active_auction = suite.query_auction_details(suite.get_default_auction_addr());
    assert_eq!(active_auction.available_amount, funds[0].amount);
    assert_eq!(active_auction.total_amount, funds[0].amount);
}

#[test]
fn test_open_auction_bid_after_end_block_passed() {
    let mut suite = Suite::default();
    let funds = coins(100_u128, suite.pair.0.clone());

    suite.auction_funds(
        suite.get_account_addr(0),
        suite
            .auction_addrs
            .get(&suite.pair.clone().into())
            .unwrap()
            .clone(),
        &funds,
    );
    suite.start_auction_day(suite.pair.clone()).unwrap();
    suite.update_block_cycle();
    suite.add_block();

    // Auction is finished, but we don't close it yet, to test if we can bid on it.
    let err = suite.do_bid_err(suite.pair.clone(), coin(1000_u128, suite.pair.1.clone()));

    assert_eq!(err, auction::error::ContractError::AuctionFinished)
}
