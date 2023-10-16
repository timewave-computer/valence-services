#[derive(Clone)]
pub struct ServicesManagerInstantiate {
    pub msg: services_manager::msg::InstantiateMsg,
}

impl From<ServicesManagerInstantiate> for services_manager::msg::InstantiateMsg {
    fn from(value: ServicesManagerInstantiate) -> Self {
        value.msg
    }
}

impl Default for ServicesManagerInstantiate {
    fn default() -> Self {
        Self::new()
    }
}
impl ServicesManagerInstantiate {
    pub fn new() -> Self {
        Self {
            msg: services_manager::msg::InstantiateMsg {},
        }
    }
}
