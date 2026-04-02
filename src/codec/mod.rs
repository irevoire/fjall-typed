mod bytes;
pub use bytes::*;
mod integer;
pub use integer::*;
mod str;
pub use str::*;
mod unit;
pub use unit::*;
#[cfg(feature = "facet_json")]
mod facet_json;
#[cfg(feature = "facet_json")]
pub use facet_json::*;

/// Dummy codec if you don't know yet which codec will be used
pub enum Unspecified {}
