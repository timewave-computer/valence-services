#[derive(Clone)]
pub struct ServicesManagerInstantiate {
    pub msg: services_manager::msg::InstantiateMsg,
}

impl From<ServicesManagerInstantiate> for services_manager::msg::InstantiateMsg {
    fn from(value: ServicesManagerInstantiate) -> Self {
        value.msg
    }
}

impl ServicesManagerInstantiate {
    pub fn new(code_ids: Vec<u64>) -> Self {
        Self {
            msg: services_manager::msg::InstantiateMsg {
                whitelisted_code_ids: code_ids,
            },
        }
    }
}
