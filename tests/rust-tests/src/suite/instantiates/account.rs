#[derive(Clone)]
pub struct AccountInstantiate {
    pub msg: valence_account::msg::InstantiateMsg,
}

impl From<AccountInstantiate> for valence_account::msg::InstantiateMsg {
    fn from(value: AccountInstantiate) -> Self {
        value.msg
    }
}

impl AccountInstantiate {
    pub fn new(services_manager: &str) -> Self {
        Self {
            msg: valence_account::msg::InstantiateMsg {
                services_manager: services_manager.to_string(),
            },
        }
    }

    /* Change functions */
    pub fn change_service_manager(&mut self, services_manager: &str) {
        self.msg.services_manager = services_manager.to_string();
    }
}
