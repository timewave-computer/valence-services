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
    pub fn default(auction_manager_addr: Addr) -> Self {
        Self::new(auction_manager_addr)
    }

    pub fn new(auction_manager_addr: Addr) -> Self {
        Self {
            msg: price_oracle::msg::InstantiateMsg {
                auction_manager_addr: auction_manager_addr.to_string(),
            },
        }
    }

    /* Change functions */
    pub fn change_auction_manager_addr(&mut self, auction_manager_addr: Addr) {
        self.msg.auction_manager_addr = auction_manager_addr.to_string();
    }
}
