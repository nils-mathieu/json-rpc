use std::borrow::Cow;

use serde::de::{Deserialize, Deserializer, Visitor};
use serde::ser::{Serialize, Serializer};

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

impl<'a> Id<'a> {
    /// Reborrows this [`Id`], creating a new instance without reallocating.
    pub fn reborrow<'b>(&'b self) -> Id<'b>
    where
        'b: 'a,
    {
        match *self {
            Self::Null => Self::Null,
            Self::Str(ref s) => Self::Str(Cow::Borrowed(s)),
            Self::Int(i) => Self::Int(i),
            Self::Uint(u) => Self::Uint(u),
            Self::Float(f) => Self::Float(f),
        }
    }
}

impl<'a> Serialize for Id<'a> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        match *self {
            Self::Null => serializer.serialize_none(),
            Self::Float(f) => serializer.serialize_f64(f),
            Self::Str(ref s) => serializer.serialize_str(s),
            Self::Int(i) => serializer.serialize_i64(i),
            Self::Uint(u) => serializer.serialize_u64(u),
        }
    }
}

impl<'de, 'a> Deserialize<'de> for Id<'a>
where
    'de: 'a,
{
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        struct IdVisitor;

        impl<'de> Visitor<'de> for IdVisitor {
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
                D: Deserializer<'de>,
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
}
