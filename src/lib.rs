//! A lightweight library defining the types declared by the [JSON-RPC 2.0] specification.
//!
//! [JSON-RPC 2.0]: https://www.jsonrpc.org/specification

#![warn(missing_docs, missing_debug_implementations)]

mod id;
pub use self::id::*;

mod request;
pub use self::request::*;

mod response;
pub use self::response::*;

mod batch;
pub use self::batch::*;

#[cfg(feature = "unknown_params")]
mod unknown_params;
#[cfg(feature = "unknown_params")]
pub use self::unknown_params::*;
