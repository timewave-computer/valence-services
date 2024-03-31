use crate::suite::suite::ADMIN;

pub struct AstroRegisteryInstantiate {
    pub msg: astroport::native_coin_registry::InstantiateMsg,
}

impl From<AstroRegisteryInstantiate> for astroport::native_coin_registry::InstantiateMsg {
    fn from(value: AstroRegisteryInstantiate) -> Self {
        value.msg
    }
}

impl Default for AstroRegisteryInstantiate {
    fn default() -> Self {
        Self::new(ADMIN)
    }
}

impl AstroRegisteryInstantiate {
    pub fn into_init(self) -> astroport::native_coin_registry::InstantiateMsg {
        self.msg
    }

    pub fn new(owner: &str) -> Self {
        Self {
            msg: astroport::native_coin_registry::InstantiateMsg {
                owner: owner.to_string(),
            },
        }
    }

    /* Change functions */
    pub fn change_owner(&mut self, owner: &str) -> &mut Self {
        self.msg.owner = owner.to_string();
        self
    }
}
