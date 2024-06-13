use cosmwasm_std::Addr;

pub struct OracleInstantiate {
    pub msg: price_oracle::msg::InstantiateMsg,
}

impl From<OracleInstantiate> for price_oracle::msg::InstantiateMsg {
    fn from(value: OracleInstantiate) -> Self {
        value.msg
    }
}

impl From<&mut OracleInstantiate> for price_oracle::msg::InstantiateMsg {
    fn from(value: &mut OracleInstantiate) -> Self {
        value.msg.clone()
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
                seconds_allow_manual_change: 60 * 60 * 24 * 2, // 2 days
                seconds_auction_prices_fresh: 60 * 60 * 24 * 3, // 3 days
            },
        }
    }

    /* Change functions */
    pub fn change_auction_manager_addr(&mut self, auctions_manager_addr: Addr) -> &mut Self {
        self.msg.auctions_manager_addr = auctions_manager_addr.to_string();
        self
    }

    /* Change functions */
    pub fn change_seconds_allow_manual_change(
        &mut self,
        seconds_allow_manual_change: u64,
    ) -> &mut Self {
        self.msg.seconds_allow_manual_change = seconds_allow_manual_change;
        self
    }
}
