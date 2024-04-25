use macros_helpers::merge_variants;
use proc_macro::TokenStream;

use quote::quote;

mod macros_helpers;

/* EXECUTE */

/// Add execute msgs that required for service managers to implament.
#[proc_macro_attribute]
pub fn valence_service_manager_execute_msgs(
    metadata: TokenStream,
    input: TokenStream,
) -> TokenStream {
    let quote = quote! {
      enum ServiceManagerExecuteMsg {
          /// Register sender to a service.
          RegisterToService {
              service_name: ValenceServices,
              data: Option<Binary>,
          },
          /// Deregister sender from a service.
          DeregisterFromService {
              service_name: ValenceServices,
          },
          /// Update the config of a service for the sender
          UpdateService {
              service_name: ValenceServices,
              data: Binary,
          },
          /// Pause service for the pause_for address
          /// Only callable by the account or the trustee
          PauseService {
              service_name: ValenceServices,
              pause_for: String,
              reason: Option<String>
          },
          /// Resume service for the pause_for address
          /// Only callable by the account or the trustee
          ResumeService {
              service_name: ValenceServices,
              resume_for: String
          },
          /// Message to aprprove the admin change if you are the new admin
          ApproveAdminChange {},
      }
    };

    merge_variants(metadata, input, quote.into())
}

/// Messages that are called by the services on the account
#[proc_macro_attribute]
pub fn valence_account_execute_msgs(metadata: TokenStream, input: TokenStream) -> TokenStream {
    let quote = quote! {
      enum AccountExecuteMsg {
        /// Register account to a service.
        RegisterToService {
            service_name: ValenceServices,
            data: Option<Binary>,
        },
        /// Unregister account from a service.
        DeregisterFromService {
            service_name: ValenceServices,
        },
        /// Update service config for the account.
        UpdateService {
            service_name: ValenceServices,
            data: Binary,
        },
        /// Pause service
        PauseService {
            service_name: ValenceServices,
            reason: Option<String>
        },
        /// Resume service
        ResumeService {
            service_name: ValenceServices,
        },

        /// Messages that are called by the services on the account to send funds
        SendFundsByService {
            msgs: Vec<CosmosMsg>,
            atomic: bool,
        },
        /// Messages that doesn't send funds called by the services
        ExecuteByService {
            msgs: Vec<CosmosMsg>,
            atomic: bool,
        },
        /// Messages that can be executed by the admin
        ExecuteByAdmin {
          msgs: Vec<CosmosMsg>
        },

        StartAdminChange {
          addr: String,
          expiration: Expiration,
        },
        CancelAdminChange {},
        ApproveAdminChange {},
      }
    };

    merge_variants(metadata, input, quote.into())
}

/// Add messages that are called on the service from the services manager
#[proc_macro_attribute]
pub fn valence_service_execute_msgs(metadata: TokenStream, input: TokenStream) -> TokenStream {
    let quote = quote! {
      enum ServiceExecuteMsg {
        /// Register this account to the service.
        Register {
            register_for: String,
            data: Option<A>,
        },
        /// Deregister the account from the services
        Deregister {
            deregister_for: String,
        },
        /// Update the config of the service for the account
        Update {
            update_for: String,
            data: B,
        },
        /// Pause the service
        Pause {
            pause_for: String,
            sender: String,
            reason: Option<String>
        },
        /// Resume the service
        Resume {
            resume_for: String,
            sender: String,
        },
      }
    };

    merge_variants(metadata, input, quote.into())
}

/// Add admin messages to services manager
#[proc_macro_attribute]
pub fn valence_service_manager_admin_msgs(
    metadata: TokenStream,
    input: TokenStream,
) -> TokenStream {
    let quote = quote! {
      enum ServiceManagerAdminMsgs {
          /// Add admin messages
          Admin(ServicesManagerAdminMsg)
      }
    };

    merge_variants(metadata, input, quote.into())
}

/* QUERIES */

/// Macro to add queries to services manager
#[proc_macro_attribute]
pub fn valence_services_manager_query_msgs(
    metadata: TokenStream,
    input: TokenStream,
) -> TokenStream {
    let quote = quote! {
      enum ServicesManagerQueryMsg {
        /// Check if address is of a service
        #[returns(Option<String>)]
        IsService {
            addr: String,
        },
        /// Get the address of a service
        #[returns(Addr)]
        GetServiceAddr {
            service: ValenceServices,
        },
        /// Get the admin of the services manager
        #[returns(Addr)]
        GetAdmin,
        /// Get list of all services and their addresses
        #[returns(Vec<(String, Addr)>)]
        GetAllServices {
          start_from: Option<String>,
          limit: Option<u64>
        },
        /// Get the service fee of a service
        #[returns(Option<Coin>)]
        GetServiceFee {
          account: String,
          service: ValenceServices,
          action: QueryFeeAction,
        },
      }
    };

    merge_variants(metadata, input, quote.into())
}

/// Macro to add queries to services manager
#[proc_macro_attribute]
pub fn valence_service_query_msgs(metadata: TokenStream, input: TokenStream) -> TokenStream {
    let quote = quote! {
      enum ServiceQueryMsg {
        /// Get the service fee of a service
        #[returns(Option<Coin>)]
        GetServiceFee {
          account: String,
          action: QueryFeeAction,
        },
      }
    };

    merge_variants(metadata, input, quote.into())
}

/* Rebalancer specific message */
#[proc_macro_attribute]
pub fn valence_rebalancer_msgs(metadata: TokenStream, input: TokenStream) -> TokenStream {
    let quote = quote! {
      enum RebalancerSpecificMsg {}
    };

    merge_variants(metadata, input, quote.into())
}
