use std::borrow::BorrowMut;

use auction_package::Pair;
use cosmwasm_schema::serde;
use cosmwasm_std::{coin, from_slice, Addr, Decimal};
use cw_multi_test::{App, AppBuilder, Executor};
use cw_storage_plus::Item;

use super::{
    contracts::{auction_contract, auctions_manager_contract, price_oracle_contract},
    instantiates::{AuctionInstantiate, AuctionsManagerInstantiate, OracleInstantiate},
    suite::{
        Suite, ADMIN, ATOM, DEFAULT_PRICE_BPS, FUNDS_PROVIDER, FUNDS_PROVIDER2, FUNDS_PROVIDER3,
        MM, NTRN,
    },
};

pub(crate) struct SuiteBuilder {
    // Users
    pub admin: Addr,
    pub funds_provider: Addr,
    pub mm: Addr,

    // Code ids of contracts
    pub oracle_code_id: u64,
    pub auctions_manager_code_id: u64,
    pub auction_code_id: u64,
}

impl Default for SuiteBuilder {
    fn default() -> Self {
        let admin = Addr::unchecked(ADMIN);
        let funds_provider = Addr::unchecked(FUNDS_PROVIDER);
        let mm = Addr::unchecked(MM);

        Self {
            admin,
            funds_provider,
            mm,

            oracle_code_id: 0,
            auctions_manager_code_id: 0,
            auction_code_id: 0,
        }
    }
}

impl SuiteBuilder {
    pub fn query_wasm_raw_item<T: for<'de> serde::de::Deserialize<'de> + serde::ser::Serialize>(
        app: &App,
        contract_addr: Addr,
        item: Item<T>,
    ) -> T {
        let res: Vec<u8> = app
            .wrap()
            .query_wasm_raw(contract_addr, item.as_slice())
            .unwrap()
            .unwrap();
        from_slice::<T>(&res).unwrap()
    }
}

// Modular build process
impl SuiteBuilder {
    /// Upload all the contracts we use in our testing
    fn upload_contracts(&mut self, app: &mut App) {
        self.auctions_manager_code_id = app.store_code(auctions_manager_contract());
        self.oracle_code_id = app.store_code(price_oracle_contract());
        self.auction_code_id = app.store_code(auction_contract());
    }

    pub fn init_manager(
        &mut self,
        app: &mut App,
        init_msg: auctions_manager::msg::InstantiateMsg,
    ) -> Addr {
        app.instantiate_contract(
            self.auctions_manager_code_id,
            self.admin.clone(),
            &init_msg,
            &[],
            "auctions_manager",
            Some(self.admin.to_string()),
        )
        .unwrap()
    }

    pub fn init_oracle(
        &mut self,
        app: &mut App,
        init_msg: price_oracle::msg::InstantiateMsg,
    ) -> Addr {
        app.instantiate_contract(
            self.oracle_code_id,
            self.admin.clone(),
            &init_msg,
            &[],
            "oracle",
            Some(self.admin.to_string()),
        )
        .unwrap()
    }
}

// Build functions
impl SuiteBuilder {
    fn set_app(&mut self) -> App {
        let all_balances = vec![
            coin(1000000000_u128, ATOM.to_string()),
            coin(1000000000_u128, NTRN.to_string()),
        ];
        let atom_balance = vec![coin(10000_u128, ATOM.to_string())];
        let ntrn_balance = vec![coin(10000_u128, NTRN.to_string())];

        AppBuilder::new().build(|router, _, storage| {
            // Give admin the balances
            router
                .bank
                .init_balance(storage, &self.admin, all_balances.clone())
                .unwrap();

            // give fund provider
            router
                .bank
                .init_balance(storage, &self.funds_provider, atom_balance.clone())
                .unwrap();
            router
                .bank
                .init_balance(
                    storage,
                    &Addr::unchecked(FUNDS_PROVIDER2),
                    atom_balance.clone(),
                )
                .unwrap();
            router
                .bank
                .init_balance(
                    storage,
                    &Addr::unchecked(FUNDS_PROVIDER3),
                    atom_balance.clone(),
                )
                .unwrap();

            router
                .bank
                .init_balance(storage, &self.mm, ntrn_balance)
                .unwrap();
        })
    }

    /// Setup function to test instantiates
    pub fn setup(&mut self) -> (App, &mut Self) {
        let mut app = self.set_app();

        // upload contracts
        self.upload_contracts(app.borrow_mut());

        (app, self)
    }
    /// build a basic suite that upload all contracts and init contracts
    pub fn build_basic(&mut self) -> Suite {
        let mut app = self.set_app();

        // upload contracts
        self.upload_contracts(app.borrow_mut());

        // Init services manager
        let manager_addr = self.init_manager(
            app.borrow_mut(),
            AuctionsManagerInstantiate::default(self.auction_code_id).into(),
        );

        // Init oracle contract
        let oracle_addr = self.init_oracle(
            app.borrow_mut(),
            OracleInstantiate::new(manager_addr.clone()).into(),
        );

        // // Init auction
        // let auction_addr =
        //     self.init_auction(app.borrow_mut(), AuctionInstantiate::atom_ntrn().into());

        Suite {
            app,
            admin: self.admin.clone(),
            funds_provider: self.funds_provider.clone(),
            mm: self.mm.clone(),
            auction_addr: Addr::unchecked(""),
            manager_addr,
            oracle_addr,
            pair: Pair(ATOM.to_string(), NTRN.to_string()),
            _pair_ntrn: Pair(NTRN.to_string(), ATOM.to_string()),
        }
    }

    /// Does a basic build but also add service to manager and register accounts to the services
    pub fn build_default(&mut self) -> Suite {
        let mut suite = self.build_basic();

        // Update the oracle address on the manager
        suite.update_oracle_addr(None);

        // update oracle price
        suite.update_oracle_price(suite.pair.clone(), Some(Decimal::bps(DEFAULT_PRICE_BPS)));

        // Create new auction contract
        suite.init_auction(AuctionInstantiate::default().into(), None);

        suite
    }
}
