use std::{
    collections::{hash_map::RandomState, BTreeMap, HashMap, HashSet},
    process::Command,
};

use auction_package::{states::{MinAmount, PRICES}, AuctionStrategy, Pair};
use cosmwasm_std::{from_json, to_json_binary, to_json_vec, Addr, Coin, Storage};
use cw_multi_test::{App, Executor};
use cw_storage_plus::Prefixer;
use rebalancer::state::CONFIGS;
use valence_package::services::{
    rebalancer::{RebalancerConfig, Target},
    ValenceServices,
};

use crate::suite::{
    instantiates::{
        AccountInstantiate, AuctionInstantiate, AuctionsManagerInstantiate, OracleInstantiate,
        RebalancerInstantiate,
    },
    suite_builder::SuiteBuilder,
};

use super::{
    helpers::concat,
    types::{MinLimitsRes, Prices, WhitelistDenoms},
    AUCTIONS_MANAGER_ADDR, NAMESPACE_WASM, REBALANCER_ADDR,
};

impl SuiteBuilder {
    pub fn ld_init_auctions(
        &mut self,
        app: &mut App,
        whitelist_denoms: Vec<String>,
        prices: Prices,
    ) -> (Addr, Addr, HashMap<(String, String), Addr>) {
        // init auction manager
        let auctions_manager_addr = self.init_auctions_manager(
            app,
            AuctionsManagerInstantiate::new(self.auction_code_id, self.mm.to_string()).into(),
        );

        // init the oracle
        let price_oracle_addr = self.init_oracle(
            app,
            OracleInstantiate::default(auctions_manager_addr.clone())
                .change_seconds_allow_manual_change(1)
                .into(),
        );

        // add oracle addr to the manager
        app.execute_contract(
            self.admin.clone(),
            auctions_manager_addr.clone(),
            &auctions_manager::msg::ExecuteMsg::Admin(Box::new(
                auctions_manager::msg::AdminMsgs::UpdateOracle {
                    oracle_addr: price_oracle_addr.to_string(),
                },
            )),
            &[],
        )
        .unwrap();

        // get min limits of each denom
        let min_limits = SuiteBuilder::get_auction_min_amounts(&whitelist_denoms);

        let mut auctions_addrs: HashMap<(String, String), Addr> = HashMap::new();
        // create auction for each whitelisted pair of denoms
        whitelist_denoms.iter().for_each(|denom_1| {
            whitelist_denoms.iter().for_each(|denom_2| {
                if denom_1 == denom_2 {
                    return;
                }

                let pair = Pair::from((denom_1.clone(), denom_2.clone()));
                let auction_init_msg = AuctionInstantiate::new(
                    pair.clone(),
                    AuctionStrategy {
                        start_price_perc: 5000,
                        end_price_perc: 5000,
                    },
                );
                let min_amount = min_limits.get(denom_1).unwrap().clone();
                app.execute_contract(
                    self.admin.clone(),
                    auctions_manager_addr.clone(),
                    &auctions_manager::msg::ExecuteMsg::Admin(Box::new(
                        auctions_manager::msg::AdminMsgs::NewAuction {
                            msg: auction_init_msg.into(),
                            label: "auction".to_string(),
                            min_amount: Some(min_amount),
                        },
                    )),
                    &[],
                )
                .unwrap();

                // Get the addr of the auction
                let auction_addr: Addr = app
                    .wrap()
                    .query_wasm_smart(
                        auctions_manager_addr.clone(),
                        &auction_package::msgs::AuctionsManagerQueryMsg::GetPairAddr {
                            pair: pair.clone(),
                        },
                    )
                    .unwrap();

                auctions_addrs.insert(pair.clone().into(), auction_addr);

                // Update the price of the auction
                let price = prices.iter().find(|(p, _)| p == &pair).unwrap().1.clone();
                let mut oracle_storage = app.contract_storage_mut(&price_oracle_addr);
                PRICES.save(oracle_storage.as_mut(), pair, &price).unwrap();
                // app.execute_contract(
                //     self.admin.clone(),
                //     price_oracle_addr.clone(),
                //     &price_oracle::msg::ExecuteMsg::ManualPriceUpdate {
                //         pair: pair.clone(),
                //         price: price.price,
                //     },
                //     &[],
                // )
                // .unwrap();
            })
        });

        (auctions_manager_addr, price_oracle_addr, auctions_addrs)
    }

    pub fn ld_init_rebalancer(
        &mut self,
        app: &mut App,
        auctions_manager_addr: Addr,
        manager_addr: Addr,
        whitelists: WhitelistDenoms,
    ) -> Addr {
        // init rebalancer
        let rebalancer_instantiate_msg: rebalancer::msg::InstantiateMsg =
            RebalancerInstantiate::default(manager_addr.as_str(), auctions_manager_addr.as_str())
                .change_denom_whitelist(whitelists.denom_whitelist)
                .change_base_denom_whitelist(whitelists.base_denom_whitelist)
                .into();

        let rebalancer_addr = app
            .instantiate_contract(
                self.rebalancer_code_id,
                self.admin.clone(),
                &rebalancer_instantiate_msg,
                &[],
                "rebalancer",
                Some(self.admin.to_string()),
            )
            .unwrap();

        app.execute_contract(
            self.admin.clone(),
            manager_addr,
            &valence_package::msgs::core_execute::ServicesManagerExecuteMsg::Admin(
                valence_package::msgs::core_execute::ServicesManagerAdminMsg::AddService {
                    name: ValenceServices::Rebalancer,
                    addr: rebalancer_addr.to_string(),
                },
            ),
            &[],
        )
        .unwrap();

        rebalancer_addr
    }

    /// Init an account and register it to the rebalancer
    pub fn ld_init_accounts(
        &mut self,
        app: &mut App,
        whitelist_denoms: Vec<String>,
        manager_addr: Addr,
        rebalancer_addr: Addr,
        account_balances: Vec<Coin>,
        account_config: RebalancerConfig,
    ) -> Addr {
        let account_init_msg: valence_account::msg::InstantiateMsg =
            AccountInstantiate::new(manager_addr.as_str()).into();

        // Instantiate the account contract
        let account_addr = app
            .instantiate_contract(
                self.account_code_id,
                self.owner.clone(),
                &account_init_msg,
                &[],
                format!("account"),
                Some(self.owner.to_string()),
            )
            .unwrap();

        // update account balance based on mainnet balance
        app.init_modules(|router, _api, storage| {
            router
                .bank
                .init_balance(storage, &account_addr, account_balances)
                .unwrap();
        });

        // register to the rebalancer with some mock data first
        let targets: HashSet<Target, _> = HashSet::from_iter(
            [
                Target {
                    denom: whitelist_denoms[0].to_string(),
                    bps: 7500,
                    min_balance: None,
                },
                Target {
                    denom: whitelist_denoms[1].to_string(),
                    bps: 2500,
                    min_balance: None,
                },
            ]
            .into_iter(),
        );

        let mut mock_rebalancer_data = SuiteBuilder::get_default_rebalancer_register_data();
        mock_rebalancer_data.base_denom = "untrn".to_string();
        mock_rebalancer_data.targets = targets;

        app.execute_contract(
            self.owner.clone(),
            account_addr.clone(),
            &valence_package::msgs::core_execute::AccountBaseExecuteMsg::RegisterToService {
                service_name: ValenceServices::Rebalancer,
                data: Some(to_json_binary(&mock_rebalancer_data).unwrap()),
            },
            &[],
        )
        .unwrap();

        // Update config in place with mainnet config
        let mut contract_storage = app.contract_storage_mut(&rebalancer_addr);
        CONFIGS
            .save(
                contract_storage.as_mut(),
                account_addr.clone(),
                &account_config,
            )
            .unwrap();

        // app.init_modules(|_router, _api, storage| {
        //     let namespace = concat(NAMESPACE_WASM, b"contract_data/");
        //     let namespace = concat(&namespace, REBALANCER_ADDR.as_bytes());
        //     let storage_key = concat(&namespace, b"configs");
        //     let account_key = concat(&storage_key, account_addr.as_bytes());
        //     storage.set(&account_key, &to_json_vec(&account_config).unwrap());
        // });

        account_addr
    }
}

impl SuiteBuilder {
    pub fn get_auction_min_amounts(denoms: &Vec<String>) -> BTreeMap<String, MinAmount> {
        BTreeMap::from_iter(denoms.iter().map(|denom| {
            let q_string = format!("{{\"get_min_limit\": {{\"denom\": \"{}\"}}}}", denom);
            let min_limits_output = Command::new("neutrond")
                .args([
                    "q",
                    "wasm",
                    "contract-state",
                    "smart",
                    AUCTIONS_MANAGER_ADDR,
                ])
                .arg(q_string)
                .args(["--node", "https://neutron-tw-rpc.polkachu.com:443"])
                .args(["--chain-id", "neutron-1"])
                .args(["--output", "json"])
                .output()
                .expect("Failed getting auctions min limits");

            (
                denom.clone(),
                from_json::<MinLimitsRes>(
                    String::from_utf8_lossy(&min_limits_output.stdout).to_string(),
                )
                .unwrap()
                .data,
            )
        }))
    }
}
