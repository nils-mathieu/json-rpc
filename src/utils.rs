//! Utility functions for JSON-RPC 2.0.

use std::borrow::Cow;

use crate::{Error, ErrorCode, Id, Request, Response};

/// A type that cannot be serialized.
enum CantSerialize {}

impl serde::Serialize for CantSerialize {
    #[inline(always)]
    fn serialize<S>(&self, _: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        match *self {}
    }
}

/// Writes a JSON-RPC 2.0 request with the provided method, params and ID to a vector of bytes.
pub fn write_request<'a, T>(
    method: &str,
    params: T,
    id: impl Into<Option<Id<'a>>>,
) -> serde_json::Result<Vec<u8>>
where
    T: serde::Serialize,
{
    let request = Request {
        method: Cow::Borrowed(method),
        params,
        id: id.into(),
    };
    serde_json::to_vec(&request)
}

/// Writes a JSON-RPC 2.0 response with the provided result and ID to a vector of bytes.
pub fn write_response<T, E>(result: Result<T, Error<E>>, id: Id) -> serde_json::Result<Vec<u8>>
where
    T: serde::Serialize,
    E: serde::Serialize,
{
    let response = Response { result, id };
    serde_json::to_vec(&response)
}

/// Writes a successful JSON-RPC 2.0 response with the provided result and ID to a vector of bytes.
pub fn write_success<T>(value: T, id: Id) -> serde_json::Result<Vec<u8>>
where
    T: serde::Serialize,
{
    let response = Response::<T, CantSerialize> {
        result: Ok(value),
        id,
    };
    serde_json::to_vec(&response)
}

/// Writes a failed JSON-RPC 2.0 response with the provided error and ID to a vector of bytes.
pub fn write_failure<E>(
    code: impl Into<ErrorCode>,
    message: &str,
    id: Id,
    data: E,
) -> serde_json::Result<Vec<u8>>
where
    E: serde::Serialize,
{
    let response = Response::<CantSerialize, E> {
        result: Err(Error {
            code: code.into(),
            message: Cow::Borrowed(message),
            data: Some(data),
        }),
        id,
    };
    serde_json::to_vec(&response)
}

/// Writes a failed JSON-RPC 2.0 response with the provided error and ID to a vector of bytes.
pub fn write_datalass_failure(
    code: impl Into<ErrorCode>,
    message: &str,
    id: Id,
) -> serde_json::Result<Vec<u8>> {
    let response = Response::<CantSerialize, CantSerialize> {
        result: Err(Error {
            code: code.into(),
            message: Cow::Borrowed(message),
            data: None,
        }),
        id,
    };
    serde_json::to_vec(&response)
}

/// Attmepts to read a request from a slice of bytes.
pub fn read_request<'a, T>(bytes: &'a [u8]) -> serde_json::Result<Request<'a, T>>
where
    T: serde::de::Deserialize<'a>,
{
    serde_json::from_slice(bytes)
}

/// Read a response from a slice of bytes.
pub fn read_response<'a, T, E>(bytes: &'a [u8]) -> serde_json::Result<Response<T, E>>
where
    T: serde::Deserialize<'a>,
    E: serde::Deserialize<'a>,
{
    serde_json::from_slice(bytes)
}

/// Read a response from a slice of bytes, ignoring any potential error data.
pub fn read_response_ignore_error<'a, T>(
    bytes: &'a [u8],
) -> serde_json::Result<Response<T, serde::de::IgnoredAny>>
where
    T: serde::Deserialize<'a>,
{
    serde_json::from_slice(bytes)
}
