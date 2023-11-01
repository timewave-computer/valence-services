use cosmwasm_std::Uint128;

use crate::suite::suite::{ATOM, NTRN, OSMO};

pub struct AuctionsManagerInstantiate {
    pub msg: auctions_manager::msg::InstantiateMsg,
}

impl From<AuctionsManagerInstantiate> for auctions_manager::msg::InstantiateMsg {
    fn from(value: AuctionsManagerInstantiate) -> Self {
        value.msg
    }
}

impl AuctionsManagerInstantiate {
    pub fn default(auction_code_id: u64) -> Self {
        Self::new(auction_code_id)
    }

    pub fn new(auction_code_id: u64) -> Self {
        let min_auction_amount = vec![
            (ATOM.to_string(), Uint128::new(5)),
            (NTRN.to_string(), Uint128::new(10)),
            (OSMO.to_string(), Uint128::new(10)),
        ];

        Self {
            msg: auctions_manager::msg::InstantiateMsg {
                auction_code_id,
                min_auction_amount,
            },
        }
    }

    /* Change functions */
    pub fn change_auction_code_id(&mut self, auction_code_id: u64) -> &mut Self {
        self.msg.auction_code_id = auction_code_id;
        self
    }
}
