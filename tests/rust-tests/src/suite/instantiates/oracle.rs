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
        Self::new(auction_manager_addr, 60 * 60 * 24)
    }

    pub fn new(auction_manager_addr: Addr, price_freshness: u64) -> Self {
        Self {
            msg: price_oracle::msg::InstantiateMsg {
                auction_manager_addr: auction_manager_addr.to_string(),
                price_freshness,
            },
        }
    }

    /* Change functions */
    pub fn change_auction_manager_addr(&mut self, auction_manager_addr: Addr) {
        self.msg.auction_manager_addr = auction_manager_addr.to_string();
    }

    pub fn change_price_freshness(&mut self, price_freshness: u64) {
        self.msg.price_freshness = price_freshness;
    }
}
