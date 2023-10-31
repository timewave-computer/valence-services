use cosmwasm_std::Addr;

pub struct OracleInstantiate {
    pub msg: price_oracle::msg::InstantiateMsg,
}

impl From<OracleInstantiate> for price_oracle::msg::InstantiateMsg {
    fn from(value: OracleInstantiate) -> Self {
        value.msg
    }
}

impl OracleInstantiate {
    pub fn default(auctions_manager_addr: Addr) -> Self {
        Self::new(auctions_manager_addr)
    }

    pub fn new(auctions_manager_addr: Addr) -> Self {
        Self {
            msg: price_oracle::msg::InstantiateMsg {
                auctions_manager_addr: auctions_manager_addr.to_string(),
            },
        }
    }

    /* Change functions */
    pub fn change_auction_manager_addr(&mut self, auctions_manager_addr: Addr) -> &mut Self {
        self.msg.auctions_manager_addr = auctions_manager_addr.to_string();
        self
    }
}
