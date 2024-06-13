use std::{collections::HashMap, hash::Hash};

use auction_package::Pair;
use cosmwasm_std::{BlockInfo, Coin};
use valence_package::services::rebalancer::RebalancerConfig;

use crate::suite::{suite::Suite, suite_builder::SuiteBuilder};

use super::types::{Prices, WhitelistDenoms};

impl SuiteBuilder {
    pub fn build_live_debug(
        block_info: BlockInfo,
        whitelists: WhitelistDenoms,
        prices: Prices,
        balances: Vec<Coin>,
        config: RebalancerConfig,
    ) -> Suite {
        let mut builder = SuiteBuilder::default();

        // Get init app
        let mut app = builder.set_app();

        // Update our mocked block info with the correct info from mainnet
        app.update_block(|b| *b = block_info.clone());

        // Upload contracts
        builder.upload_contracts(&mut app);

        // init oracle, auctions manager and auctions
        let (auctions_manager_addr, oracle_addr, auction_addrs) =
            builder.ld_init_auctions(&mut app, whitelists.denom_whitelist.clone(), prices);

        // init services manager
        let manager_addr = builder.init_manager(&mut app);

        // init and register rebalancer
        let rebalancer_addr = builder.ld_init_rebalancer(
            &mut app,
            auctions_manager_addr.clone(),
            manager_addr.clone(),
            whitelists.clone(),
        );

        let account_addr = builder.ld_init_accounts(
            &mut app,
            whitelists.denom_whitelist.clone(),
            manager_addr.clone(),
            rebalancer_addr.clone(),
            balances,
            config,
        );

        Suite {
            app,
            admin: builder.admin,
            owner: builder.owner,
            trustee: builder.trustee,
            mm: builder.mm,
            auctions_manager_addr,
            oracle_addr,
            manager_addr,
            rebalancer_addr,
            account_addrs: vec![account_addr],
            auction_addrs,
            pair: Pair::from((
                whitelists.denom_whitelist[0].clone(),
                whitelists.denom_whitelist[1].clone(),
            )),
            account_code_id: builder.account_code_id,
            astro_pools: HashMap::new(),
        }
    }
}
