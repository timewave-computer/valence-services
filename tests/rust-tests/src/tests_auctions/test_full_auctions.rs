use auction::state::ActiveAuctionStatus;
use cosmwasm_std::{coins, testing::mock_env, Addr, Uint128};

use crate::suite::suite::{Suite, FUNDS_PROVIDER2, FUNDS_PROVIDER3};

#[test]
fn test_close_auction() {
    let mut suite = Suite::default();
    let funds = coins(1000_u128, suite.pair.0.clone());
    suite.auction_funds(None, &funds);

    suite.start_auction(
        Some(mock_env().block.height),
        mock_env().block.height + 1000,
    );

    suite.update_block(500);

    suite.do_full_bid(1000_u128);
    suite.assert_auction_status(ActiveAuctionStatus::Finished);

    suite.close_auction(None);
    suite.assert_auction_status(ActiveAuctionStatus::AuctionClosed);

    let user_balance = suite
        .app
        .wrap()
        .query_balance(suite.funds_provider.clone(), suite.pair.1.clone())
        .unwrap();
    assert_eq!(user_balance.amount, Uint128::from(1500_u128));

    let mm_balance = suite
        .app
        .wrap()
        .query_balance(suite.mm.clone(), suite.pair.0.clone())
        .unwrap();
    assert_eq!(mm_balance.amount, Uint128::from(1000_u128));
}

#[test]
fn test_close_auction_time() {
    let mut suite = Suite::default();
    let funds = coins(1000_u128, suite.pair.0.clone());
    suite.auction_funds(None, &funds);

    suite.start_auction(
        Some(mock_env().block.height),
        mock_env().block.height + 1000,
    );

    suite.update_block(500);

    suite.do_full_bid(500_u128);

    suite.update_block(501);
    suite.close_auction(None);

    let user_balance = suite
        .app
        .wrap()
        .query_balance(suite.funds_provider.clone(), suite.pair.1.clone())
        .unwrap();
    assert_eq!(user_balance.amount, Uint128::from(750_u128));

    let mm_balance = suite
        .app
        .wrap()
        .query_balance(suite.mm.clone(), suite.pair.0.clone())
        .unwrap();
    assert_eq!(mm_balance.amount, Uint128::from(500_u128));
}

#[test]
fn test_close_auction_no_bids() {
    let mut suite = Suite::default();
    let funds = coins(1000_u128, suite.pair.0.clone());
    suite.auction_funds(None, &funds);

    suite.start_auction(
        Some(mock_env().block.height),
        mock_env().block.height + 1000,
    );

    suite.update_block(1001);

    suite.close_auction(None);

    let user_balance = suite
        .app
        .wrap()
        .query_balance(suite.funds_provider.clone(), suite.pair.1.clone())
        .unwrap();
    assert_eq!(user_balance.amount, Uint128::zero());

    let mm_balance = suite
        .app
        .wrap()
        .query_balance(suite.mm.clone(), suite.pair.0.clone())
        .unwrap();
    assert_eq!(mm_balance.amount, Uint128::zero());
}

#[test]
fn test_auction_multiple_providers() {
    let mut suite = Suite::default();
    let funds = coins(1000_u128, suite.pair.0.clone());
    let provider2 = Addr::unchecked(FUNDS_PROVIDER2);
    let provider3 = Addr::unchecked(FUNDS_PROVIDER3);

    suite.auction_funds(None, &funds);
    suite.auction_funds(Some(provider2.clone()), &funds);
    suite.auction_funds(Some(provider3.clone()), &funds);

    suite.start_auction(
        Some(mock_env().block.height),
        mock_env().block.height + 1000,
    );

    suite.update_block(500);

    suite.do_full_bid(3000_u128);

    suite.close_auction(Some(1));
    suite.assert_auction_status(ActiveAuctionStatus::CloseAuction(
        Some(suite.funds_provider.clone()),
        Uint128::zero(),
        1500_u128.into(),
    ));
    suite.close_auction(Some(1));
    suite.assert_auction_status(ActiveAuctionStatus::CloseAuction(
        Some(provider2.clone()),
        Uint128::zero(),
        3000_u128.into(),
    ));
    suite.close_auction(Some(1));
    suite.assert_auction_status(ActiveAuctionStatus::CloseAuction(
        Some(provider3.clone()),
        Uint128::zero(),
        4500_u128.into(),
    ));
    suite.close_auction(Some(1));
    suite.assert_auction_status(ActiveAuctionStatus::AuctionClosed);

    let provider1_balance = suite
        .app
        .wrap()
        .query_balance(suite.funds_provider.clone(), suite.pair.1.clone())
        .unwrap();
    assert_eq!(provider1_balance.amount.u128(), 1500_u128);

    let provider2_balance = suite
        .app
        .wrap()
        .query_balance(provider2, suite.pair.1.clone())
        .unwrap();
    assert_eq!(provider2_balance.amount.u128(), 1500_u128);

    let provider3_balance = suite
        .app
        .wrap()
        .query_balance(provider3, suite.pair.1.clone())
        .unwrap();
    assert_eq!(provider3_balance.amount.u128(), 1500_u128);

    let mm_balance = suite
        .app
        .wrap()
        .query_balance(suite.mm.clone(), suite.pair.0.clone())
        .unwrap();
    assert_eq!(mm_balance.amount.u128(), 3000_u128);
}

#[test]
fn test_auction_multiple_providers_rounding() {
    let mut suite = Suite::default();
    let provider2 = Addr::unchecked(FUNDS_PROVIDER2);
    let provider3 = Addr::unchecked(FUNDS_PROVIDER3);

    suite.auction_funds(None, &coins(1234_u128, suite.pair.0.clone()));
    suite.auction_funds(
        Some(provider2.clone()),
        &coins(2678_u128, suite.pair.0.clone()),
    );
    suite.auction_funds(
        Some(provider3.clone()),
        &coins(1357_u128, suite.pair.0.clone()),
    );

    suite.start_auction(
        Some(mock_env().block.height),
        mock_env().block.height + 1000,
    );

    suite.update_block(500);

    suite.do_full_bid(5269_u128);

    suite.close_auction(Some(5));

    let provider1_balance = suite
        .app
        .wrap()
        .query_balance(suite.funds_provider.clone(), suite.pair.1.clone())
        .unwrap();

    let provider2_balance = suite
        .app
        .wrap()
        .query_balance(provider2, suite.pair.1.clone())
        .unwrap();

    let provider3_balance = suite
        .app
        .wrap()
        .query_balance(provider3, suite.pair.1.clone())
        .unwrap();

    let total_sent = provider1_balance.amount + provider2_balance.amount + provider3_balance.amount;

    let active_auction = suite.query_auction_details();
    // One is leftover
    assert_eq!(total_sent, active_auction.resolved_amount - Uint128::one());

    let mm_balance = suite
        .app
        .wrap()
        .query_balance(suite.mm.clone(), suite.pair.0.clone())
        .unwrap();
    assert_eq!(mm_balance.amount.u128(), 5269_u128);
}

#[test]
fn test_auction_rounding_leftovers() {
    let mut suite = Suite::default();
    let provider2 = Addr::unchecked(FUNDS_PROVIDER2);
    let provider3 = Addr::unchecked(FUNDS_PROVIDER3);

    suite.auction_funds(None, &coins(1234_u128, suite.pair.0.clone()));
    suite.auction_funds(
        Some(provider2.clone()),
        &coins(2678_u128, suite.pair.0.clone()),
    );
    suite.auction_funds(
        Some(provider3.clone()),
        &coins(1357_u128, suite.pair.0.clone()),
    );

    suite.start_auction(
        Some(mock_env().block.height),
        mock_env().block.height + 1000,
    );

    suite.update_block(500);

    suite.do_full_bid(5268_u128);

    suite.update_block(500);
    suite.close_auction(Some(5));

    let provider1_balance = suite
        .app
        .wrap()
        .query_balance(suite.funds_provider.clone(), suite.pair.1.clone())
        .unwrap();

    let provider2_balance = suite
        .app
        .wrap()
        .query_balance(provider2, suite.pair.1.clone())
        .unwrap();

    let provider3_balance = suite
        .app
        .wrap()
        .query_balance(provider3, suite.pair.1.clone())
        .unwrap();

    let total_sent = provider1_balance.amount + provider2_balance.amount + provider3_balance.amount;

    let active_auction = suite.query_auction_details();
    // One is leftover
    assert_eq!(total_sent, active_auction.resolved_amount - Uint128::one());

    // Fund next auction with 100 pair.0
    suite.auction_funds(None, &coins(100_u128, suite.pair.0.clone()));

    // Start uaction
    suite.start_auction(None, suite.app.block_info().height + 1000);

    let active_auction = suite.query_auction_details();

    // resolved and total funds should have 1 extra because those are leftovers from previous auction
    assert_eq!(active_auction.total_amount, Uint128::from(101_u128)); // 100 funded + 1 leftover
    assert_eq!(active_auction.available_amount, Uint128::from(101_u128)); // 100 funded + 1 leftover
    assert_eq!(active_auction.resolved_amount, Uint128::one()) // 1 leftover
}

#[test]
fn test_multiple_auctions() {
    let mut suite = Suite::default();
    let provider2 = Addr::unchecked(FUNDS_PROVIDER2);
    let provider3 = Addr::unchecked(FUNDS_PROVIDER3);

    suite.auction_funds(None, &coins(100_u128, suite.pair.0.clone()));

    suite.start_auction(
        Some(mock_env().block.height),
        mock_env().block.height + 1000,
    );

    suite.update_block(500);

    suite.do_full_bid(100_u128);

    suite.close_auction(None);

    suite.auction_funds(None, &coins(100_u128, suite.pair.0.clone()));
    suite.auction_funds(
        Some(provider2.clone()),
        &coins(100_u128, suite.pair.0.clone()),
    );
    suite.auction_funds(
        Some(provider3.clone()),
        &coins(100_u128, suite.pair.0.clone()),
    );

    suite.start_auction(None, suite.app.block_info().height + 1000);

    suite.update_block(500);

    suite.do_full_bid(100_u128);
    suite.do_full_bid(100_u128);
    suite.do_full_bid(100_u128);

    suite.close_auction(None);

    suite.auction_funds(None, &coins(100_u128, suite.pair.0.clone()));
    suite.auction_funds(Some(provider2), &coins(100_u128, suite.pair.0.clone()));
    suite.auction_funds(Some(provider3), &coins(101_u128, suite.pair.0.clone()));

    suite.start_auction(None, suite.app.block_info().height + 1000);

    suite.update_block(500);

    suite.do_full_bid(300_u128);

    suite.update_block(500);

    suite.close_auction(None);

    // Check what the new auction looks like
    suite.auction_funds(None, &coins(100_u128, suite.pair.0.clone()));

    suite.start_auction(None, suite.app.block_info().height + 1000);

    let active_auction = suite.query_auction_details();
    assert_eq!(active_auction.available_amount, Uint128::from(101_u128)); // 1 leftover and 100 new funds
    assert_eq!(active_auction.resolved_amount, Uint128::from(2_u128)); // 2 leftover from previous auction
}
