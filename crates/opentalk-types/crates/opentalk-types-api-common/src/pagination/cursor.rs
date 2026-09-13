// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use derive_more::{AsRef, Deref, DerefMut};

/// Opaque token which represents T as a base64 string (where T is encoded using postcard)
///
/// Used for cursor based pagination
#[derive(Deref, DerefMut, AsRef, Debug, Copy, Clone, Eq, PartialEq)]
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
pub struct Cursor<T>(pub T);

#[cfg(feature = "serde")]
mod serde_impls {
    use std::marker::PhantomData;

    use base64::{Engine, engine::general_purpose::URL_SAFE_NO_PAD};
    use serde::{Deserialize, Deserializer, Serialize, Serializer, de::DeserializeOwned};

    use super::*;

    impl<T: Serialize> Cursor<T> {
        /// Encode T using postcard and return it as base64 string
        pub fn to_base64(&self) -> String {
            URL_SAFE_NO_PAD.encode(postcard::to_stdvec(&self.0).unwrap())
        }
    }

    impl<T: Serialize> Serialize for Cursor<T> {
        fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
        where
            S: Serializer,
        {
            serializer.serialize_str(&self.to_base64())
        }
    }

    impl<T: DeserializeOwned> Cursor<T> {
        /// Decode T using postcard from a base64 string
        pub fn from_base64(input: &str) -> Result<Self, serde::de::value::Error> {
            Self::from_base64_inner(input)
        }

        fn from_base64_inner<E>(input: &str) -> Result<Self, E>
        where
            E: serde::de::Error,
        {
            let expected = &CursorVisitor::<T>(PhantomData::<T>);
            let bytes = URL_SAFE_NO_PAD.decode(input).map_err(|_| {
                serde::de::Error::invalid_value(serde::de::Unexpected::Str(input), expected)
            })?;
            let data = postcard::from_bytes(&bytes).map_err(|_| {
                serde::de::Error::invalid_value(serde::de::Unexpected::Bytes(&bytes), expected)
            })?;

            Ok(Self(data))
        }
    }

    impl<'de, T: DeserializeOwned> Deserialize<'de> for Cursor<T> {
        fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
        where
            D: Deserializer<'de>,
        {
            deserializer.deserialize_str(CursorVisitor::<T>(PhantomData))
        }
    }

    struct CursorVisitor<T>(PhantomData<T>);

    impl<T: DeserializeOwned> serde::de::Visitor<'_> for CursorVisitor<T> {
        type Value = Cursor<T>;

        fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
            write!(formatter, "base64 + postcard encoded cursor data")
        }

        fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
        where
            E: serde::de::Error,
        {
            Cursor::from_base64_inner(v)
        }
    }
}

#[cfg(all(test, feature = "serde"))]
mod serde_tests {
    use serde::{Deserialize, Serialize};
    use serde_json::json;

    use super::Cursor;

    #[test]
    fn serialize_empty() {
        let cursor = Cursor(());

        assert_eq!(serde_json::to_value(cursor).unwrap(), json!(""));
    }

    #[test]
    fn deserialize_empty() {
        assert_eq!(
            serde_json::from_value::<Cursor::<()>>(json!("")).unwrap(),
            Cursor(())
        );
    }

    #[test]
    fn serialize_simple_string() {
        let cursor = Cursor("abc".to_string());

        assert_eq!(serde_json::to_value(cursor).unwrap(), json!("A2FiYw"));
    }

    #[test]
    fn deserialize_simple_string() {
        assert_eq!(
            serde_json::from_value::<Cursor::<String>>(json!("A2FiYw")).unwrap(),
            Cursor("abc".to_string())
        );
    }

    #[derive(Debug, PartialEq, Eq, Serialize, Deserialize)]
    struct Paging {
        offset: usize,
        count: usize,
    }

    #[test]
    fn serialize_struct() {
        let cursor = Cursor(Paging {
            offset: 10,
            count: 5,
        });

        assert_eq!(serde_json::to_value(cursor).unwrap(), json!("CgU"));
    }

    #[test]
    fn deserialize_struct() {
        let cursor = Cursor(Paging {
            offset: 10,
            count: 5,
        });

        assert_eq!(
            serde_json::from_value::<Cursor::<Paging>>(json!("CgU")).unwrap(),
            cursor
        );
    }

    #[test]
    fn deserialize_invalid_encoded() {
        assert!(serde_json::from_value::<Cursor::<String>>(json!("XXXXYYYZZZINVALID")).is_err(),);
    }

    #[test]
    fn deserialize_incompatible_type() {
        assert!(serde_json::from_value::<Cursor::<String>>(json!("CgU")).is_err(),);
    }
}
