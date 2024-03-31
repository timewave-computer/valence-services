use auction::state::ActiveAuctionStatus;
use cosmwasm_std::{coins, testing::mock_env, Uint128};

use crate::suite::{
    suite::{Suite, DEFAULT_BALANCE_AMOUNT},
    suite_builder::SuiteBuilder,
};

#[test]
fn test_close_auction() {
    let mut suite = Suite::default();
    let funds = coins(1000_u128, suite.pair.0.clone());
    suite.auction_funds(
        suite.get_account_addr(0),
        suite.get_default_auction_addr(),
        &funds,
    );

    suite
        .start_auction(
            suite.pair.clone(),
            Some(mock_env().block.height),
            mock_env().block.height + 1000,
        )
        .unwrap();

    suite.update_block(500);

    suite.do_full_bid(1000_u128);
    suite.assert_auction_status(ActiveAuctionStatus::Finished);

    suite.close_auction(suite.pair.clone(), None).unwrap();
    suite.assert_auction_status(ActiveAuctionStatus::AuctionClosed);

    let user_balance = suite
        .app
        .wrap()
        .query_balance(suite.get_account_addr(0), suite.pair.1.clone())
        .unwrap();
    assert_eq!(user_balance.amount, Uint128::from(1500_u128));

    let mm_balance = suite
        .app
        .wrap()
        .query_balance(suite.mm.clone(), suite.pair.0.clone())
        .unwrap();
    assert_eq!(
        mm_balance.amount,
        DEFAULT_BALANCE_AMOUNT + Uint128::from(1000_u128)
    );
}

#[test]
fn test_close_auction_time() {
    let mut suite = Suite::default();
    let funds = coins(1000_u128, suite.pair.0.clone());
    suite.auction_funds(
        suite.get_account_addr(0),
        suite.get_default_auction_addr(),
        &funds,
    );

    suite
        .start_auction(
            suite.pair.clone(),
            Some(mock_env().block.height),
            mock_env().block.height + 1000,
        )
        .unwrap();

    suite.update_block(500);

    suite.do_full_bid(500_u128);

    suite.update_block(501);
    suite.close_auction(suite.pair.clone(), None).unwrap();

    let user_balance = suite
        .app
        .wrap()
        .query_balance(suite.get_account_addr(0), suite.pair.1.clone())
        .unwrap();
    assert_eq!(user_balance.amount, Uint128::from(750_u128));

    let mm_balance = suite
        .app
        .wrap()
        .query_balance(suite.mm.clone(), suite.pair.0.clone())
        .unwrap();
    assert_eq!(
        mm_balance.amount,
        DEFAULT_BALANCE_AMOUNT + Uint128::from(500_u128)
    );
}

#[test]
fn test_close_auction_no_bids() {
    let mut suite = Suite::default();
    let funds = coins(1000_u128, suite.pair.0.clone());
    suite.auction_funds(
        suite.get_account_addr(0),
        suite.get_default_auction_addr(),
        &funds,
    );

    suite
        .start_auction(
            suite.pair.clone(),
            Some(mock_env().block.height),
            mock_env().block.height + 1000,
        )
        .unwrap();

    suite.update_block(1001);

    suite.close_auction(suite.pair.clone(), None).unwrap();

    let user_balance = suite
        .app
        .wrap()
        .query_balance(suite.get_account_addr(0), suite.pair.1.clone())
        .unwrap();
    assert_eq!(user_balance.amount, Uint128::zero());

    let mm_balance = suite
        .app
        .wrap()
        .query_balance(suite.mm.clone(), suite.pair.0.clone())
        .unwrap();
    assert_eq!(mm_balance.amount, DEFAULT_BALANCE_AMOUNT);
}

#[test]
fn test_auction_multiple_providers() {
    let mut suite = SuiteBuilder::default().with_accounts(3).build_default();
    let funds = coins(1000_u128, suite.pair.0.clone());
    let provider2 = suite.get_account_addr(1);
    let provider3 = suite.get_account_addr(2);

    suite.auction_funds(
        suite.get_account_addr(0),
        suite.get_default_auction_addr(),
        &funds,
    );
    suite.auction_funds(provider2.clone(), suite.get_default_auction_addr(), &funds);
    suite.auction_funds(provider3.clone(), suite.get_default_auction_addr(), &funds);

    suite
        .start_auction(
            suite.pair.clone(),
            Some(mock_env().block.height),
            mock_env().block.height + 1000,
        )
        .unwrap();

    suite.update_block(500);

    suite.do_full_bid(3000_u128);

    suite.close_auction(suite.pair.clone(), Some(1)).unwrap();
    suite.assert_auction_status(ActiveAuctionStatus::CloseAuction(
        Some(suite.get_account_addr(0)),
        Uint128::zero(),
        1500_u128.into(),
    ));
    suite.close_auction(suite.pair.clone(), Some(1)).unwrap();
    suite.assert_auction_status(ActiveAuctionStatus::CloseAuction(
        Some(provider2.clone()),
        Uint128::zero(),
        3000_u128.into(),
    ));
    suite.close_auction(suite.pair.clone(), Some(1)).unwrap();
    suite.assert_auction_status(ActiveAuctionStatus::CloseAuction(
        Some(provider3.clone()),
        Uint128::zero(),
        4500_u128.into(),
    ));
    suite.close_auction(suite.pair.clone(), Some(1)).unwrap();
    suite.assert_auction_status(ActiveAuctionStatus::AuctionClosed);

    let provider1_balance = suite
        .app
        .wrap()
        .query_balance(suite.get_account_addr(0), suite.pair.1.clone())
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
    assert_eq!(
        mm_balance.amount.u128(),
        DEFAULT_BALANCE_AMOUNT.u128() + 3000_u128
    );
}

#[test]
fn test_auction_multiple_providers_rounding() {
    let mut suite = SuiteBuilder::default().with_accounts(3).build_default();
    let provider2 = suite.get_account_addr(1);
    let provider3 = suite.get_account_addr(2);

    suite.auction_funds(
        suite.get_account_addr(0),
        suite.get_default_auction_addr(),
        &coins(234_u128, suite.pair.0.clone()),
    );
    suite.auction_funds(
        provider2.clone(),
        suite.get_default_auction_addr(),
        &coins(678_u128, suite.pair.0.clone()),
    );
    suite.auction_funds(
        provider3.clone(),
        suite.get_default_auction_addr(),
        &coins(357_u128, suite.pair.0.clone()),
    );

    suite
        .start_auction(
            suite.pair.clone(),
            Some(mock_env().block.height),
            mock_env().block.height + 1000,
        )
        .unwrap();

    suite.update_block(500);

    suite.do_full_bid(1269_u128);

    suite.close_auction(suite.pair.clone(), None).unwrap();

    let provider1_balance = suite
        .app
        .wrap()
        .query_balance(suite.get_account_addr(0), suite.pair.1.clone())
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

    let active_auction = suite.query_auction_details(suite.get_default_auction_addr());
    // One is leftover
    assert_eq!(total_sent, active_auction.resolved_amount - Uint128::one());

    let mm_balance = suite
        .app
        .wrap()
        .query_balance(suite.mm.clone(), suite.pair.0.clone())
        .unwrap();
    assert_eq!(
        mm_balance.amount.u128(),
        DEFAULT_BALANCE_AMOUNT.u128() + 1269_u128
    );
}

#[test]
fn test_auction_rounding_leftovers() {
    let mut suite = SuiteBuilder::default().with_accounts(3).build_default();
    let provider2 = suite.get_account_addr(1);
    let provider3 = suite.get_account_addr(2);

    suite.auction_funds(
        suite.get_account_addr(0),
        suite.get_default_auction_addr(),
        &coins(134_u128, suite.pair.0.clone()),
    );
    suite.auction_funds(
        provider2.clone(),
        suite.get_default_auction_addr(),
        &coins(278_u128, suite.pair.0.clone()),
    );
    suite.auction_funds(
        provider3.clone(),
        suite.get_default_auction_addr(),
        &coins(359_u128, suite.pair.0.clone()),
    );

    suite
        .start_auction(
            suite.pair.clone(),
            Some(mock_env().block.height),
            mock_env().block.height + 1000,
        )
        .unwrap();

    suite.update_block(500);

    // We bid 770, but the total is 771, so 1 pair.0 is left as leftover
    suite.do_full_bid(770_u128);

    suite.update_block(500);
    suite.close_auction(suite.pair.clone(), None).unwrap();

    let provider1_balance = suite
        .app
        .wrap()
        .query_balance(suite.get_account_addr(0), suite.pair.1.clone())
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

    let active_auction = suite.query_auction_details(suite.get_default_auction_addr());
    // Two is leftover
    assert_eq!(
        total_sent,
        active_auction.resolved_amount - Uint128::from(2_u128)
    );

    // Fund next auction with 100 pair.0
    suite.auction_funds(
        suite.get_account_addr(0),
        suite.get_default_auction_addr(),
        &coins(100_u128, suite.pair.0.clone()),
    );

    // Start uaction
    suite
        .start_auction(
            suite.pair.clone(),
            None,
            suite.app.block_info().height + 1000,
        )
        .unwrap();

    let active_auction = suite.query_auction_details(suite.get_default_auction_addr());

    // resolved and total funds should have 1 extra because those are leftovers from previous auction
    assert_eq!(active_auction.total_amount, Uint128::from(101_u128)); // 100 funded + 1 leftover pair.0
    assert_eq!(active_auction.available_amount, Uint128::from(101_u128)); // 100 funded + 1 leftover pair.0
    assert_eq!(active_auction.resolved_amount, Uint128::from(2_u128)) // 2 leftover pair.1
}

#[test]
fn test_multiple_auctions() {
    let mut suite = SuiteBuilder::default().with_accounts(3).build_default();
    let provider2 = suite.get_account_addr(1);
    let provider3 = suite.get_account_addr(2);

    suite.auction_funds(
        suite.get_account_addr(0),
        suite.get_default_auction_addr(),
        &coins(100_u128, suite.pair.0.clone()),
    );

    suite
        .start_auction(
            suite.pair.clone(),
            Some(mock_env().block.height),
            mock_env().block.height + 1000,
        )
        .unwrap();

    suite.update_block(500);

    suite.do_full_bid(100_u128);

    suite.close_auction(suite.pair.clone(), None).unwrap();

    suite.auction_funds(
        suite.get_account_addr(0),
        suite.get_default_auction_addr(),
        &coins(100_u128, suite.pair.0.clone()),
    );
    suite.auction_funds(
        provider2.clone(),
        suite.get_default_auction_addr(),
        &coins(100_u128, suite.pair.0.clone()),
    );
    suite.auction_funds(
        provider3.clone(),
        suite.get_default_auction_addr(),
        &coins(100_u128, suite.pair.0.clone()),
    );

    suite
        .start_auction(
            suite.pair.clone(),
            None,
            suite.app.block_info().height + 1000,
        )
        .unwrap();

    suite.update_block(500);

    suite.do_full_bid(100_u128);
    suite.do_full_bid(100_u128);
    suite.do_full_bid(100_u128);

    suite.close_auction(suite.pair.clone(), None).unwrap();

    suite.auction_funds(
        suite.get_account_addr(0),
        suite.get_default_auction_addr(),
        &coins(100_u128, suite.pair.0.clone()),
    );
    suite.auction_funds(
        provider2,
        suite.get_default_auction_addr(),
        &coins(100_u128, suite.pair.0.clone()),
    );
    suite.auction_funds(
        provider3,
        suite.get_default_auction_addr(),
        &coins(101_u128, suite.pair.0.clone()),
    );

    suite
        .start_auction(
            suite.pair.clone(),
            None,
            suite.app.block_info().height + 1000,
        )
        .unwrap();

    suite.update_block(500);

    suite.do_full_bid(300_u128);

    suite.update_block(500);

    suite.close_auction(suite.pair.clone(), None).unwrap();

    // Check what the new auction looks like
    suite.auction_funds(
        suite.get_account_addr(0),
        suite.get_default_auction_addr(),
        &coins(100_u128, suite.pair.0.clone()),
    );

    suite
        .start_auction(
            suite.pair.clone(),
            None,
            suite.app.block_info().height + 1000,
        )
        .unwrap();

    let active_auction = suite.query_auction_details(suite.get_default_auction_addr());
    assert_eq!(active_auction.available_amount, Uint128::from(101_u128)); // 1 leftover and 100 new funds
    assert_eq!(active_auction.resolved_amount, Uint128::from(2_u128)); // 2 leftover from previous auction
}
