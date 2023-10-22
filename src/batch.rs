use serde::de::{SeqAccess, Visitor};
use serde::{Deserialize, Deserializer, Serialize, Serializer};

use crate::Request;

/// Represents either one, or multiple JSON-RPC [`Request`]s.
///
/// Note that this type is only really useful for deserializing requests, using the
/// [`UnknownParams`] type as the parameter type.
///
/// [`UnknownParams`]: crate::UnknownParams
#[derive(Debug, Clone)]
pub enum MaybeBatchedRequests<'a, P> {
    /// A single request.
    Single(Request<'a, P>),
    /// A batch of requests.
    Batch(Vec<Request<'a, P>>),
}

impl<'a, T> Serialize for MaybeBatchedRequests<'a, T>
where
    T: Clone + Serialize,
{
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        match self {
            Self::Batch(batch) => batch.serialize(serializer),
            Self::Single(single) => single.serialize(serializer),
        }
    }
}

impl<'de, 'a, P> Deserialize<'de> for MaybeBatchedRequests<'a, P>
where
    'de: 'a,
    P: Deserialize<'de>,
{
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        struct MaybeBatchedVisitor<P>(std::marker::PhantomData<P>);

        impl<'de, P> Visitor<'de> for MaybeBatchedVisitor<P>
        where
            P: Deserialize<'de>,
        {
            type Value = MaybeBatchedRequests<'de, P>;

            fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
                formatter.write_str("a JSON-RPC 2.0 request")
            }

            fn visit_seq<A>(self, seq: A) -> Result<Self::Value, A::Error>
            where
                A: SeqAccess<'de>,
            {
                Vec::deserialize(serde::de::value::SeqAccessDeserializer::new(seq))
                    .map(MaybeBatchedRequests::Batch)
            }

            fn visit_map<A>(self, map: A) -> Result<Self::Value, A::Error>
            where
                A: serde::de::MapAccess<'de>,
            {
                Request::deserialize(serde::de::value::MapAccessDeserializer::new(map))
                    .map(MaybeBatchedRequests::Single)
            }
        }

        deserializer.deserialize_any(MaybeBatchedVisitor(std::marker::PhantomData))
    }
}
