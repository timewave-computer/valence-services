use crate::suite::suite::{ADMIN, FEE};

pub struct AstroFactoryInstantiate {
    pub msg: astroport::factory::InstantiateMsg,
}

impl From<AstroFactoryInstantiate> for astroport::factory::InstantiateMsg {
    fn from(value: AstroFactoryInstantiate) -> Self {
        value.msg
    }
}

impl AstroFactoryInstantiate {
    pub fn into_init(self) -> astroport::factory::InstantiateMsg {
        self.msg
    }

    pub fn default(
        token_code_id: u64,
        coin_registry_address: &str,
        pair_code_id: u64,
        pair_stable_code_id: u64,
    ) -> Self {
        Self::new(
            ADMIN.to_string(),
            Some(FEE.to_string()),
            vec![
                astroport::factory::PairConfig {
                    code_id: pair_code_id,
                    pair_type: astroport::factory::PairType::Xyk {},
                    total_fee_bps: 100,
                    maker_fee_bps: 10,
                    is_disabled: false,
                    is_generator_disabled: true,
                },
                astroport::factory::PairConfig {
                    code_id: pair_stable_code_id,
                    pair_type: astroport::factory::PairType::Stable {},
                    total_fee_bps: 100,
                    maker_fee_bps: 10,
                    is_disabled: false,
                    is_generator_disabled: true,
                },
            ],
            token_code_id,
            None,
            coin_registry_address.to_string(),
        )
    }

    pub fn new(
        owner: String,
        fee_address: Option<String>,
        pair_configs: Vec<astroport::factory::PairConfig>,
        token_code_id: u64,
        generator_address: Option<String>,
        coin_registry_address: String,
    ) -> Self {
        Self {
            msg: astroport::factory::InstantiateMsg {
                owner,
                fee_address,
                pair_configs,
                token_code_id,
                whitelist_code_id: 10000,
                generator_address,
                coin_registry_address,
            },
        }
    }

    /* Change functions */
    pub fn change_pair_configs(
        &mut self,
        pair_configs: Vec<astroport::factory::PairConfig>,
    ) -> &mut Self {
        self.msg.pair_configs = pair_configs;
        self
    }

    pub fn change_owner(&mut self, owner: &str) -> &mut Self {
        self.msg.owner = owner.to_string();
        self
    }

    pub fn change_fee_address(&mut self, fee_address: Option<&str>) -> &mut Self {
        self.msg.fee_address = fee_address.map(|f| f.to_string());
        self
    }

    pub fn change_token_code_id(&mut self, token_code_id: u64) -> &mut Self {
        self.msg.token_code_id = token_code_id;
        self
    }

    pub fn change_whitelist_code_id(&mut self, whitelist_code_id: u64) -> &mut Self {
        self.msg.whitelist_code_id = whitelist_code_id;
        self
    }

    pub fn change_generator_address(&mut self, generator_address: Option<&str>) -> &mut Self {
        self.msg.generator_address = generator_address.map(|f| f.to_string());
        self
    }

    pub fn change_coin_registry_address(&mut self, coin_registry_address: &str) -> &mut Self {
        self.msg.coin_registry_address = coin_registry_address.to_string();
        self
    }
}
