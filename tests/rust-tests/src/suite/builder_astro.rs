use std::collections::HashMap;

use cosmwasm_schema::serde;
use cosmwasm_std::{to_json_binary, Addr, Coin, StdError, Uint128};
use cw_multi_test::{App, Executor};

use super::{
    instantiates::{AstroFactoryInstantiate, AstroRegisteryInstantiate},
    suite::{ATOM, NTRN, OSMO},
    suite_builder::SuiteBuilder,
};

impl SuiteBuilder {
    fn init_astro_factory(
        &mut self,
        app: &mut App,
        init_msg: astroport::factory::InstantiateMsg,
    ) -> Addr {
        app.instantiate_contract(
            self.astro_factory_code_id,
            self.admin.clone(),
            &init_msg,
            &[],
            "astro_factor",
            Some(self.admin.to_string()),
        )
        .unwrap()
    }

    fn init_astro_registery(
        &mut self,
        app: &mut App,
        init_msg: astroport::native_coin_registry::InstantiateMsg,
    ) -> Addr {
        app.instantiate_contract(
            self.astro_registery_code_id,
            self.admin.clone(),
            &init_msg,
            &[],
            "astro_registery",
            Some(self.admin.to_string()),
        )
        .unwrap()
    }

    // fn init_astro_whitelist(
    //     &mut self,
    //     app: &mut App,
    //     init_msg: cw1_whitelist::msg::InstantiateMsg,
    // ) -> Addr {
    //     app.instantiate_contract(
    //         self.astro_whitelist_code_id,
    //         self.admin.clone(),
    //         &init_msg,
    //         &[],
    //         "astro_whitelist",
    //         Some(self.admin.to_string()),
    //     )
    //     .unwrap()
    // }

    fn init_pair_pool<T: serde::Serialize>(
        &mut self,
        app: &mut App,
        factory_addr: &Addr,
        pair_type: astroport::factory::PairType,
        asset_infos: Vec<astroport::asset::AssetInfo>,
        init_params: T,
    ) -> Addr {
        let init_atom_ntrn_msg = astroport::factory::ExecuteMsg::CreatePair {
            pair_type,
            asset_infos: asset_infos.clone(),
            init_params: Some(to_json_binary::<T>(&init_params).unwrap()),
        };

        app.execute_contract(
            self.admin.clone(),
            factory_addr.clone(),
            &init_atom_ntrn_msg,
            &[],
        )
        .unwrap();

        let pair_info: astroport::asset::PairInfo = app
            .wrap()
            .query_wasm_smart(
                factory_addr,
                &astroport::factory::QueryMsg::Pair { asset_infos },
            )
            .unwrap();

        pair_info.contract_addr
    }

    // Returns (factory_Addr, registery_Addr, pools)
    pub fn init_astro(&mut self, app: &mut App) -> HashMap<(String, String), Addr> {
        let registery_init_msg = AstroRegisteryInstantiate::default();
        let registery_addr = self.init_astro_registery(app, registery_init_msg.into());

        // Add pairs to registery
        app.execute_contract(
            self.admin.clone(),
            registery_addr.clone(),
            &astroport::native_coin_registry::ExecuteMsg::Add {
                native_coins: vec![
                    (ATOM.to_string(), 6),
                    (NTRN.to_string(), 6),
                    (OSMO.to_string(), 6),
                ],
            },
            &[],
        )
        .unwrap();

        let factory_init_msg = AstroFactoryInstantiate::default(
            self.astro_token_code_id,
            registery_addr.as_str(),
            self.astro_pair_code_id,
            self.astro_stable_pair_code_id,
        );
        let factory_addr = self.init_astro_factory(app, factory_init_msg.into());

        let mut pools: HashMap<(String, String), Addr> = HashMap::new();

        // Init atom ntrn as a stable pair
        let asset_infos = vec![
            astroport::asset::AssetInfo::NativeToken {
                denom: ATOM.to_string(),
            },
            astroport::asset::AssetInfo::NativeToken {
                denom: NTRN.to_string(),
            },
        ];
        let pair_type = astroport::factory::PairType::Stable {};
        let init_params = astroport::pair::StablePoolParams {
            owner: Some(self.admin.to_string()),
            amp: 1,
        };

        let atom_ntrn_pool_addr =
            self.init_pair_pool(app, &factory_addr, pair_type, asset_infos, init_params);
        pools.insert(
            (ATOM.to_string(), NTRN.to_string()),
            atom_ntrn_pool_addr.clone(),
        );
        pools.insert(
            (NTRN.to_string(), ATOM.to_string()),
            atom_ntrn_pool_addr.clone(),
        );

        let assets = vec![
            astroport::asset::Asset {
                info: astroport::asset::AssetInfo::NativeToken {
                    denom: ATOM.to_string(),
                },
                amount: Uint128::from(1_000_000_000_u128),
            },
            astroport::asset::Asset {
                info: astroport::asset::AssetInfo::NativeToken {
                    denom: NTRN.to_string(),
                },
                amount: Uint128::from(1_000_000_000_u128),
            },
        ];

        self.astro_provide_liquidity(app, self.admin.clone(), atom_ntrn_pool_addr, assets);

        // Init NTRN OSMO pair as XYK pool
        let asset_infos = vec![
            astroport::asset::AssetInfo::NativeToken {
                denom: NTRN.to_string(),
            },
            astroport::asset::AssetInfo::NativeToken {
                denom: OSMO.to_string(),
            },
        ];
        let pair_type = astroport::factory::PairType::Xyk {};
        let init_params = astroport::pair::XYKPoolParams {
            track_asset_balances: None,
        };

        let ntrn_osmo_pool_addr =
            self.init_pair_pool(app, &factory_addr, pair_type, asset_infos, init_params);
        pools.insert(
            (NTRN.to_string(), OSMO.to_string()),
            ntrn_osmo_pool_addr.clone(),
        );
        pools.insert(
            (OSMO.to_string(), NTRN.to_string()),
            ntrn_osmo_pool_addr.clone(),
        );

        let assets = vec![
            astroport::asset::Asset {
                info: astroport::asset::AssetInfo::NativeToken {
                    denom: NTRN.to_string(),
                },
                amount: Uint128::from(1_000_000_000_u128),
            },
            astroport::asset::Asset {
                info: astroport::asset::AssetInfo::NativeToken {
                    denom: OSMO.to_string(),
                },
                amount: Uint128::from(2_000_000_000_u128),
            },
        ];

        self.astro_provide_liquidity(app, self.admin.clone(), ntrn_osmo_pool_addr, assets);

        pools
    }
}

impl SuiteBuilder {
    pub fn astro_provide_liquidity(
        &mut self,
        app: &mut App,
        from: Addr,
        pool_addr: Addr,
        assets: Vec<astroport::asset::Asset>,
    ) -> &mut Self {
        let balances = assets
            .iter()
            .map(|asset| {
                let denom = match asset.info.clone() {
                    astroport::asset::AssetInfo::Token { .. } => Err(StdError::generic_err(
                        "we do not support tokens, only native",
                    )),
                    astroport::asset::AssetInfo::NativeToken { denom } => Ok(denom),
                }
                .unwrap();

                Coin {
                    denom,
                    amount: asset.amount,
                }
            })
            .collect::<Vec<Coin>>();

        let provide_liquidity_msg = astroport::pair::ExecuteMsg::ProvideLiquidity {
            assets,
            slippage_tolerance: None,
            auto_stake: Some(false),
            receiver: Some(from.to_string()),
        };

        app.execute_contract(
            from,
            Addr::unchecked(pool_addr),
            &provide_liquidity_msg,
            &balances,
        )
        .unwrap();

        self
    }
}
