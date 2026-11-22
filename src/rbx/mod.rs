//! Access into Roblox APIs.
//!
//! Most usage should go through the `RbxCloud` struct.
//!
//! [`client`] and [`credentials`] are the shared transport that new typed
//! API families are built on. The `v1` and `v2` clients do not use it yet.
pub mod client;
pub mod credentials;
pub mod error;
pub mod types;
pub(crate) mod util;
pub mod v1;
pub mod v2;
