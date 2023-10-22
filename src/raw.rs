//! Internal module responsible for defining structures that represent JSON-RPC 2.0 messages.

use std::borrow::Cow;

pub fn serialize_request<S, P>(
    request: &crate::Request<P>,
    serializer: S,
) -> Result<S::Ok, S::Error>
where
    S: serde::Serializer,
    P: serde::Serialize,
{
    serde::Serialize::serialize(&OutgoingRequest::from_request(request), serializer)
}

pub fn deserialize_request<'de, D, P>(deserializer: D) -> Result<crate::Request<'de, P>, D::Error>
where
    D: serde::Deserializer<'de>,
    P: serde::Deserialize<'de>,
{
    serde::Deserialize::deserialize(deserializer).and_then(IncomingRequest::into_request)
}

pub fn serialize_response<S, T, E>(
    response: &crate::Response<T, E>,
    serializer: S,
) -> Result<S::Ok, S::Error>
where
    S: serde::Serializer,
    T: serde::Serialize,
    E: serde::Serialize,
{
    serde::Serialize::serialize(&OutogingResponse::from_response(response), serializer)
}

pub fn deserialize_response<'de, D, T, E>(
    deserializer: D,
) -> Result<crate::Response<'de, T, E>, D::Error>
where
    D: serde::Deserializer<'de>,
    T: serde::Deserialize<'de>,
    E: serde::Deserialize<'de>,
{
    serde::Deserialize::deserialize(deserializer).and_then(IncomingResponse::into_response)
}

#[derive(serde::Serialize)]
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
            id: req.id.as_ref().map(id_as_ref),
        }
    }
}

#[derive(serde::Deserialize)]
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

#[derive(serde::Serialize)]
struct OutogingResponse<'a, T, E> {
    jsonrpc: &'a str,
    #[serde(skip_serializing_if = "Option::is_none")]
    result: Option<&'a T>,
    #[serde(skip_serializing_if = "Option::is_none")]
    error: Option<OutgoingError<'a, E>>,
    id: crate::Id<'a>,
}

impl<'a, T, E> OutogingResponse<'a, T, E> {
    fn from_response(response: &'a crate::Response<T, E>) -> Self {
        let (result, error) = match response.result {
            Ok(ref result) => (Some(result), None),
            Err(ref error) => (
                None,
                Some(OutgoingError {
                    code: error.code.0,
                    message: &error.message,
                    data: error.data.as_ref(),
                }),
            ),
        };

        Self {
            jsonrpc: "2.0",
            result,
            error,
            id: id_as_ref(&response.id),
        }
    }
}

#[derive(serde::Serialize)]
struct OutgoingError<'a, E> {
    code: i64,
    message: &'a str,
    #[serde(skip_serializing_if = "Option::is_none")]
    data: Option<&'a E>,
}

#[derive(serde::Deserialize)]
struct IncomingResponse<'a, T, E> {
    #[serde(borrow)]
    jsonrpc: Cow<'a, str>,
    #[serde(default = "Option::default")]
    result: Option<T>,
    #[serde(default = "Option::default", borrow)]
    error: Option<IncomingError<'a, E>>,
    // The option is there to represent the `null` value. The `id` field must still be
    // present.
    #[serde(borrow)]
    id: crate::Id<'a>,
}

impl<'a, T, E> IncomingResponse<'a, T, E> {
    fn into_response<Er>(self) -> Result<crate::Response<'a, T, E>, Er>
    where
        Er: serde::de::Error,
    {
        if self.jsonrpc != "2.0" {
            return Err(Er::invalid_value(
                serde::de::Unexpected::Str(&self.jsonrpc),
                &"2.0",
            ));
        }

        let result = match (self.result, self.error) {
            (Some(result), None) => Ok(result),
            (None, Some(error)) => Err(crate::Error {
                code: crate::ErrorCode(error.code),
                message: error.message,
                data: error.data,
            }),
            (Some(_), Some(_)) => {
                return Err(Er::custom(
                    "response cannot contain both `result` and `error` fields",
                ))
            }
            (None, None) => {
                return Err(Er::custom(
                    "response must contain either `result` or `error` field",
                ))
            }
        };

        Ok(crate::Response {
            result,
            id: self.id,
        })
    }
}

#[derive(serde::Deserialize)]
struct IncomingError<'a, E> {
    code: i64,
    #[serde(borrow)]
    message: Cow<'a, str>,
    #[serde(default = "Option::default")]
    data: Option<E>,
}

fn id_as_ref<'a>(id: &'a crate::Id) -> crate::Id<'a> {
    match *id {
        crate::Id::Null => crate::Id::Null,
        crate::Id::Float(f) => crate::Id::Float(f),
        crate::Id::Str(ref s) => crate::Id::Str(Cow::Borrowed(s)),
        crate::Id::Int(i) => crate::Id::Int(i),
        crate::Id::Uint(u) => crate::Id::Uint(u),
    }
}

pub fn serialize_id<S>(id: &crate::Id, serializer: S) -> Result<S::Ok, S::Error>
where
    S: serde::Serializer,
{
    match *id {
        crate::Id::Null => serializer.serialize_none(),
        crate::Id::Float(f) => serializer.serialize_f64(f),
        crate::Id::Str(ref s) => serializer.serialize_str(s),
        crate::Id::Int(i) => serializer.serialize_i64(i),
        crate::Id::Uint(u) => serializer.serialize_u64(u),
    }
}

pub fn deserialize_id<'de, D>(deserializer: D) -> Result<crate::Id<'de>, D::Error>
where
    D: serde::Deserializer<'de>,
{
    struct IdVisitor;

    impl<'de> serde::de::Visitor<'de> for IdVisitor {
        type Value = crate::Id<'de>;

        fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
            formatter.write_str("a JSON-RPC 2.0 ID")
        }

        fn visit_none<E>(self) -> Result<Self::Value, E>
        where
            E: serde::de::Error,
        {
            Ok(crate::Id::Null)
        }

        fn visit_some<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
        where
            D: serde::Deserializer<'de>,
        {
            deserializer.deserialize_any(self)
        }

        fn visit_borrowed_str<E>(self, v: &'de str) -> Result<Self::Value, E>
        where
            E: serde::de::Error,
        {
            Ok(crate::Id::Str(Cow::Borrowed(v)))
        }

        fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
        where
            E: serde::de::Error,
        {
            Ok(crate::Id::Str(Cow::Owned(v.to_owned())))
        }

        fn visit_string<E>(self, v: String) -> Result<Self::Value, E>
        where
            E: serde::de::Error,
        {
            Ok(crate::Id::Str(Cow::Owned(v)))
        }

        fn visit_i64<E>(self, v: i64) -> Result<Self::Value, E>
        where
            E: serde::de::Error,
        {
            Ok(crate::Id::Int(v))
        }

        fn visit_u64<E>(self, v: u64) -> Result<Self::Value, E>
        where
            E: serde::de::Error,
        {
            Ok(crate::Id::Uint(v))
        }

        fn visit_f64<E>(self, v: f64) -> Result<Self::Value, E>
        where
            E: serde::de::Error,
        {
            Ok(crate::Id::Float(v))
        }
    }

    deserializer.deserialize_option(IdVisitor)
}
