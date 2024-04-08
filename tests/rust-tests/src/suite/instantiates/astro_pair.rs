use cosmwasm_std::Binary;

use crate::suite::suite::{ATOM, NTRN, OSMO};

pub struct AstroPairInstantiate {
    pub msg: astroport::pair::InstantiateMsg,
}

impl From<AstroPairInstantiate> for astroport::pair::InstantiateMsg {
    fn from(value: AstroPairInstantiate) -> Self {
        value.msg
    }
}

impl AstroPairInstantiate {
    pub fn into_init(self) -> astroport::pair::InstantiateMsg {
        self.msg
    }

    pub fn default_atom_ntrn(token_code_id: u64, factory_addr: &str) -> Self {
        Self::new(
            token_code_id,
            factory_addr,
            vec![
                astroport::asset::AssetInfo::NativeToken {
                    denom: ATOM.to_string(),
                },
                astroport::asset::AssetInfo::NativeToken {
                    denom: NTRN.to_string(),
                },
            ],
            None,
        )
    }

    pub fn default_ntrn_osmo(token_code_id: u64, factory_addr: &str) -> Self {
        Self::new(
            token_code_id,
            factory_addr,
            vec![
                astroport::asset::AssetInfo::NativeToken {
                    denom: NTRN.to_string(),
                },
                astroport::asset::AssetInfo::NativeToken {
                    denom: OSMO.to_string(),
                },
            ],
            None,
        )
    }

    pub fn new(
        token_code_id: u64,
        factory_addr: &str,
        asset_infos: Vec<astroport::asset::AssetInfo>,
        init_params: Option<Binary>,
    ) -> Self {
        Self {
            msg: astroport::pair::InstantiateMsg {
                asset_infos,
                token_code_id,
                factory_addr: factory_addr.to_string(),
                init_params,
            },
        }
    }

    /* Change functions */
    pub fn change_token_code_id(&mut self, token_code_id: u64) -> &mut Self {
        self.msg.token_code_id = token_code_id;
        self
    }

    pub fn change_factory_addr(&mut self, factory_addr: &str) -> &mut Self {
        self.msg.factory_addr = factory_addr.to_string();
        self
    }

    pub fn change_asset_infos(
        &mut self,
        asset_infos: Vec<astroport::asset::AssetInfo>,
    ) -> &mut Self {
        self.msg.asset_infos = asset_infos;
        self
    }

    pub fn change_init_params(&mut self, init_params: Option<Binary>) -> &mut Self {
        self.msg.init_params = init_params;
        self
    }
}
