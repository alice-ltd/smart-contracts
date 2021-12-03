//! The `cosmos-sdk-proto` crate provides access to the Cosmos SDK proto-defined structs.
//! These are then re-exported in a module structure as close as possible to the proto files.
//!
//! The version strings are intentionally excluded, that way users may specify `cosmos-sdk` versions
//! as a feature argument to this crate and not have to change their imports. For as much as that is
//! worth considering that modules seem to show up and go away with every RC.
//!
//! TODO: actually implement features tag based compilation

#![cfg_attr(docsrs, feature(doc_cfg))]
#![doc(
    html_logo_url = "https://raw.githubusercontent.com/cosmos/cosmos-rust/main/.images/cosmos.png",
    html_root_url = "https://docs.rs/cosmos-sdk-proto/0.7.1"
)]
#![forbid(unsafe_code)]
#![warn(trivial_casts, trivial_numeric_casts, unused_import_braces)]

pub use tendermint_proto as tendermint;

/// The version (commit hash) of the Cosmos SDK used when generating this library.
pub const COSMOS_SDK_VERSION: &str = include_str!("prost/COSMOS_SDK_COMMIT");

/// Cosmos protobuf definitions.
pub mod cosmos {
    /// Grant arbitrary privileges from one account to another.
    pub mod authz {
        pub mod v1beta1 {
            include!("prost/cosmos.authz.v1beta1.rs");
        }
    }

    /// Balances.
    pub mod bank {
        pub mod v1beta1 {
            include!("prost/cosmos.bank.v1beta1.rs");
        }
    }

    /// Base functionality.
    pub mod base {
        pub mod v1beta1 {
            include!("prost/cosmos.base.v1beta1.rs");
        }
    }
}
