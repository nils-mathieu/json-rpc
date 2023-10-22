//! A lightweight library defining the types declared by the [JSON-RPC 2.0] specification.
//!
//! [JSON-RPC 2.0]: https://www.jsonrpc.org/specification

#![warn(missing_docs, missing_debug_implementations)]

use std::borrow::Cow;

mod raw;

/// A JSON-RPC 2.0 request.
#[derive(Debug, Clone)]
pub struct Request<'a, P> {
    /// The method to be invoked.
    pub method: Cow<'a, str>,
    /// The parameters to be passed to the method.
    pub params: P,
    /// The identifier associated with the request.
    pub id: Option<Id<'a>>,
}

impl<'a, P> serde::Serialize for Request<'a, P>
where
    P: serde::Serialize,
{
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        raw::serialize_request(self, serializer)
    }
}

impl<'de, 'a, P> serde::Deserialize<'de> for Request<'a, P>
where
    'de: 'a,
    P: serde::Deserialize<'de>,
{
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        raw::deserialize_request(deserializer)
    }
}

/// A request identifier.
///
/// JSON-RPC 2.0 clients can use this to match responses sent back by a complying server
/// with the request they sent. This is especially useful when sending multiple requests
/// at the same time without waiting for a response in between.
#[derive(Debug, Clone)]
pub enum Id<'a> {
    /// The ID was `null`.
    Null,
    /// The ID was a string.
    Str(Cow<'a, str>),
    /// The ID was a signed integer.
    Int(i64),
    /// The ID was an unsigned integer.
    Uint(u64),
    /// The ID was a floating point number.
    ///
    /// # Note
    ///
    /// THe JSON-RPC 2.0 specification specifies that a client *should not* use floating point
    /// values as request IDs, but they technically are legal. For this reason, we have to
    /// account for them.
    Float(f64),
}

impl<'a> serde::Serialize for Id<'a> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        raw::serialize_id(self, serializer)
    }
}

impl<'de, 'a> serde::Deserialize<'de> for Id<'a>
where
    'de: 'a,
{
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        raw::deserialize_id(deserializer)
    }
}

/// The parameters passed to a JSON-RPC 2.0 request.
///
/// This type can be used by servers to accept arbitrary parameters from clients, allowing
/// them to check the name of the method before calling [`UnknownParams::parse`] to deserialize
/// the corresponding parameters without having to parse the request a second time.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[cfg(feature = "unknown_params")]
pub struct UnknownParams<'a>(#[serde(borrow)] Option<&'a serde_json::value::RawValue>);

#[cfg(feature = "unknown_params")]
impl<'a> UnknownParams<'a> {
    /// Parses the parameters as a JSON value.
    pub fn parse<T>(&self) -> serde_json::Result<T>
    where
        T: serde::Deserialize<'a>,
    {
        let s = self.0.map_or("[]", serde_json::value::RawValue::get);
        serde_json::from_str(s)
    }
}

/// A JSON-RPC 2.0 response.
#[derive(Debug, Clone)]
pub struct Response<'a, T, E> {
    /// The outcome of the request.
    pub result: Result<T, Error<'a, E>>,
    /// The ID of the request to which this repsonse is a reply.
    pub id: Id<'a>,
}

impl<'a, T, E> serde::Serialize for Response<'a, T, E>
where
    T: serde::Serialize,
    E: serde::Serialize,
{
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        raw::serialize_response(self, serializer)
    }
}

impl<'de, 'a, T, E> serde::Deserialize<'de> for Response<'a, T, E>
where
    'de: 'a,
    T: serde::Deserialize<'de>,
    E: serde::Deserialize<'de>,
{
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        raw::deserialize_response(deserializer)
    }
}

/// A JSON-RPC 2.0 error code.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ErrorCode(pub i64);

/// A JSON-RPC 2.0 error.
#[derive(Debug, Clone)]
pub struct Error<'a, E> {
    /// The error code.
    pub code: ErrorCode,
    /// The error message.
    pub message: Cow<'a, str>,
    /// Additional data about the error.
    pub data: Option<E>,
}
