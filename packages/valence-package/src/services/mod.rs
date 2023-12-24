pub mod rebalancer;

use std::{fmt, str::FromStr};

use cosmwasm_schema::cw_serde;
use cosmwasm_std::{from_json, to_json_binary, Binary, CosmosMsg, Empty, WasmMsg};
use valence_macros::valence_service_execute_msgs;

use crate::error::ValenceError;

use self::rebalancer::{RebalancerData, RebalancerUpdateData};

#[valence_service_execute_msgs]
#[cw_serde]
pub enum GeneralServiceExecuteMsg<A, B> {}

/// An enum that represent all services that available for valence accounts
#[cw_serde]
pub enum ValenceServices {
    /// The rebalancer service
    Rebalancer,
    // /// A boilerplate placeholder for a future services
    // // also look at service management tests
    // Test,
}

impl ValenceServices {
    /// Verify the register message exists and its not None
    fn require_register_msg(
        data: Option<Binary>,
        service_name: &str,
    ) -> Result<Binary, ValenceError> {
        data.ok_or(ValenceError::MissingRegisterData(service_name.to_string()))
    }

    /// Parse the register message into the needed struct based on the service
    fn parse_data<T: for<'de> cosmwasm_schema::serde::Deserialize<'de>>(
        data: Binary,
        service_name: &str,
    ) -> Result<T, ValenceError> {
        from_json::<T>(&data).map_err(|_| {
            ValenceError::RegisterDataParseError(format!("{service_name} register msg",))
        })
    }

    /// Get the register message based on the service
    /// Returns `None` if the service doesn't require a register message
    /// Return `Some(...)` with the required register message if the service requires it
    pub fn get_register_msg(
        &self,
        sender: &str,
        contract_addr: &str,
        data: Option<Binary>,
    ) -> Result<CosmosMsg, ValenceError> {
        match self {
            ValenceServices::Rebalancer => {
                let data = ValenceServices::require_register_msg(data, "rebalancer")?;

                let register = ValenceServices::parse_data(data, "rebalancer")?;

                let msg = WasmMsg::Execute {
                    contract_addr: contract_addr.to_string(),
                    msg: to_json_binary(
                        &GeneralServiceExecuteMsg::<RebalancerData, Empty>::Register {
                            register_for: sender.to_string(),
                            data: Some(register),
                        },
                    )?,
                    funds: vec![],
                };

                Ok(msg.into())
            } // // Example to a service that doesn't require a register message
              // ValenceServices::Test => {
              //     let msg = WasmMsg::Execute {
              //         contract_addr: contract_addr.to_string(),
              //         msg: to_binary(&GeneralServiceExecuteMsg::<Empty, Empty>::Register {
              //             register_for: sender.to_string(),
              //             data: None,
              //         })?,
              //         funds: vec![],
              //     };

              //     Ok(msg.into())
              // }
        }
    }

    /// Get the update message based on the service
    /// Returns `None` if the service doesn't require an update message
    /// Return `Some(...)` with the required register message if the service requires it
    pub fn get_update_msg(
        &self,
        sender: &str,
        contract_addr: &str,
        data: Binary,
    ) -> Result<CosmosMsg, ValenceError> {
        match self {
            ValenceServices::Rebalancer => {
                let update = ValenceServices::parse_data(data, "rebalancer")?;

                let msg = WasmMsg::Execute {
                    contract_addr: contract_addr.to_string(),
                    msg: to_json_binary(
                        &GeneralServiceExecuteMsg::<Empty, RebalancerUpdateData>::Update {
                            update_for: sender.to_string(),
                            data: update,
                        },
                    )?,
                    funds: vec![],
                };

                Ok(msg.into())
            } // // Example to a service that doesn't require a register message
              // ValenceServices::Test => {
              //     let msg = WasmMsg::Execute {
              //         contract_addr: contract_addr.to_string(),
              //         msg: to_binary(&GeneralServiceExecuteMsg::<Empty, Empty>::Update {
              //             update_for: sender.to_string(),
              //             data: Empty {},
              //         })?,
              //         funds: vec![],
              //     };

              //     Ok(msg.into())
              // }
        }
    }

    /// Get deregister msg for services
    /// the deregister msg is the same for all services for now
    /// can be switched to work indevidually if needed.
    pub fn get_deregister_msg(
        &self,
        sender: &str,
        contract_addr: &str,
    ) -> Result<CosmosMsg, ValenceError> {
        Ok(WasmMsg::Execute {
            contract_addr: contract_addr.to_string(),
            msg: to_json_binary(&GeneralServiceExecuteMsg::<Empty, Empty>::Deregister {
                deregister_for: sender.to_string(),
            })?,
            funds: vec![],
        }
        .into())
    }

    pub fn get_pause_msg(
        &self,
        pause_for: String,
        sender: &str,
        contract_addr: &str,
    ) -> Result<CosmosMsg, ValenceError> {
        Ok(WasmMsg::Execute {
            contract_addr: contract_addr.to_string(),
            msg: to_json_binary(&GeneralServiceExecuteMsg::<Empty, Empty>::Pause {
                pause_for,
                sender: sender.to_string(),
            })?,
            funds: vec![],
        }
        .into())
    }

    pub fn get_resume_msg(
        &self,
        resume_for: String,
        sender: &str,
        contract_addr: &str,
    ) -> Result<CosmosMsg, ValenceError> {
        Ok(WasmMsg::Execute {
            contract_addr: contract_addr.to_string(),
            msg: to_json_binary(&GeneralServiceExecuteMsg::<Empty, Empty>::Resume {
                resume_for,
                sender: sender.to_string(),
            })?,
            funds: vec![],
        }
        .into())
    }
}

// TODO: make a macro for the below
/// Turn a string into a ValenceServices enum
impl FromStr for ValenceServices {
    type Err = ValenceError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "rebalancer" => Ok(ValenceServices::Rebalancer),
            // "test" => Ok(ValenceServices::Test),
            _ => Err(ValenceError::InvalidService(s.to_string())),
        }
    }
}

/// Turn a ValenceServices enum into a string
impl fmt::Display for ValenceServices {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            ValenceServices::Rebalancer => write!(f, "rebalancer"),
            // ValenceServices::Test => write!(f, "test"),
        }
    }
}

#[cfg(test)]
mod test {
    use std::str::FromStr;

    use crate::error::ValenceError;

    use super::ValenceServices;

    #[test]
    fn test_parse() {
        let str = ValenceServices::from_str("rebalancer").unwrap();
        assert_eq!(str, ValenceServices::Rebalancer);

        // let str = ValenceServices::from_str("test").unwrap();
        // assert_eq!(str, ValenceServices::Test);

        let err = ValenceServices::from_str("random_unknown_service").unwrap_err();
        assert_eq!(
            err,
            ValenceError::InvalidService("random_unknown_service".to_string())
        );

        let str = ValenceServices::Rebalancer.to_string();
        assert_eq!(str, "rebalancer");

        // let str = ValenceServices::Test.to_string();
        // assert_eq!(str, "test");
    }
}
