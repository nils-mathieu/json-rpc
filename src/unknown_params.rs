use serde::{Deserialize, Serialize};

/// The parameters passed to a JSON-RPC 2.0 request.
///
/// This type can be used by servers to accept arbitrary parameters from clients, allowing
/// them to check the name of the method before calling [`UnknownParams::parse`] to deserialize
/// the corresponding parameters without having to parse the request a second time.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UnknownParams<'a>(#[serde(borrow)] Option<&'a serde_json::value::RawValue>);

impl<'a> UnknownParams<'a> {
    /// Parses the parameters as a JSON value.
    pub fn parse<T>(&self) -> serde_json::Result<T>
    where
        T: Deserialize<'a>,
    {
        let s = self.0.map_or("[]", serde_json::value::RawValue::get);
        serde_json::from_str(s)
    }
}
