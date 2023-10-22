use std::borrow::Cow;

use serde::{Deserialize, Deserializer, Serialize, Serializer};

use crate::Id;

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

impl<'a, P> Serialize for Request<'a, P>
where
    P: Serialize,
{
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        OutgoingRequest::from_request(self).serialize(serializer)
    }
}

impl<'de, 'a, P> Deserialize<'de> for Request<'a, P>
where
    'de: 'a,
    P: Deserialize<'de>,
{
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        IncomingRequest::deserialize(deserializer).and_then(IncomingRequest::into_request)
    }
}

#[derive(Serialize)]
struct OutgoingRequest<'a, P> {
    jsonrpc: &'a str,
    method: &'a str,
    params: &'a P,
    #[serde(skip_serializing_if = "Option::is_none")]
    id: Option<crate::Id<'a>>,
}

impl<'a, P> OutgoingRequest<'a, P> {
    fn from_request(req: &'a crate::Request<'_, P>) -> Self {
        Self {
            jsonrpc: "2.0",
            method: &req.method,
            params: &req.params,
            id: req.id.as_ref().map(Id::reborrow),
        }
    }
}

#[derive(Deserialize)]
struct IncomingRequest<'a, P> {
    #[serde(borrow)]
    jsonrpc: Cow<'a, str>,
    #[serde(borrow)]
    method: Cow<'a, str>,
    params: P,
    #[serde(borrow, default, skip_serializing_if = "Option::is_none")]
    id: Option<crate::Id<'a>>,
}

impl<'a, P> IncomingRequest<'a, P> {
    fn into_request<E>(self) -> Result<crate::Request<'a, P>, E>
    where
        E: serde::de::Error,
        P: serde::Deserialize<'a>,
    {
        if self.jsonrpc != "2.0" {
            return Err(E::invalid_value(
                serde::de::Unexpected::Str(&self.jsonrpc),
                &"2.0",
            ));
        }

        Ok(crate::Request {
            method: self.method,
            params: self.params,
            id: self.id,
        })
    }
}
