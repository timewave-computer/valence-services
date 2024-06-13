use cosmwasm_std::{testing::mock_env, Timestamp, Uint128};
use valence_package::services::rebalancer::{BaseDenom, ServiceFeeConfig};

use crate::suite::suite::{ATOM, NTRN, OSMO};

#[derive(Clone)]
pub struct RebalancerInstantiate {
    pub msg: rebalancer::msg::InstantiateMsg,
}

impl From<RebalancerInstantiate> for rebalancer::msg::InstantiateMsg {
    fn from(value: RebalancerInstantiate) -> Self {
        value.msg
    }
}

impl From<&mut RebalancerInstantiate> for rebalancer::msg::InstantiateMsg {
    fn from(value: &mut RebalancerInstantiate) -> Self {
        value.msg.clone()
    }
}

impl RebalancerInstantiate {
    pub fn default(services_manager: &str, auctions_manager: &str) -> Self {
        Self {
            msg: rebalancer::msg::InstantiateMsg {
                denom_whitelist: vec![ATOM.to_string(), NTRN.to_string(), OSMO.to_string()],
                base_denom_whitelist: vec![
                    BaseDenom {
                        denom: ATOM.to_string(),
                        min_balance_limit: Uint128::from(100_u128),
                    },
                    BaseDenom {
                        denom: NTRN.to_string(),
                        min_balance_limit: Uint128::from(100_u128),
                    },
                ],
                services_manager_addr: services_manager.to_string(),
                cycle_start: mock_env().block.time,
                auctions_manager_addr: auctions_manager.to_string(), // to modify
                cycle_period: None,
                fees: ServiceFeeConfig {
                    denom: NTRN.to_string(),
                    register_fee: Uint128::zero(),
                    resume_fee: Uint128::zero(),
                },
            },
        }
    }

    pub fn new(
        denom_whitelist: Vec<String>,
        base_denom_whitelist: Vec<BaseDenom>,
        cycle_start: Timestamp,
        services_manager: &str,
        auctions_manager: &str,
        cycle_period: Option<u64>,
        fees: ServiceFeeConfig,
    ) -> Self {
        Self {
            msg: rebalancer::msg::InstantiateMsg {
                denom_whitelist,
                base_denom_whitelist,
                services_manager_addr: services_manager.to_string(), // to modify
                cycle_start,
                auctions_manager_addr: auctions_manager.to_string(), // to modify
                cycle_period,
                fees,
            },
        }
    }

    /* Change functions */
    pub fn change_denom_whitelist(&mut self, denom_whitelist: Vec<String>) -> &mut Self {
        self.msg.denom_whitelist = denom_whitelist;
        self
    }

    pub fn change_base_denom_whitelist(
        &mut self,
        base_denom_whitelist: Vec<BaseDenom>,
    ) -> &mut Self {
        self.msg.base_denom_whitelist = base_denom_whitelist;
        self
    }

    pub fn change_service_manager(&mut self, services_manager: &str) -> &mut Self {
        self.msg.services_manager_addr = services_manager.to_string();
        self
    }

    pub fn change_cycle_start(&mut self, cycle_start: Timestamp) -> &mut Self {
        self.msg.cycle_start = cycle_start;
        self
    }

    pub fn change_auctions_manager(&mut self, auctions_manager: &str) -> &mut Self {
        self.msg.auctions_manager_addr = auctions_manager.to_string();
        self
    }

    pub fn change_cycle_period(mut self, cycle_period: Option<u64>) -> Self {
        self.msg.cycle_period = cycle_period;
        self
    }

    pub fn change_fees(&mut self, fee: impl Into<Uint128> + Copy) -> &mut Self {
        self.msg.fees = ServiceFeeConfig {
            denom: NTRN.to_string(),
            register_fee: fee.into(),
            resume_fee: fee.into(),
        };
        self
    }
}
