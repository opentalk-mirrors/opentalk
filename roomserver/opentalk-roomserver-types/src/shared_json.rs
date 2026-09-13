// SPDX-License-Identifier: EUPL-1.2
//
// SPDX-FileCopyrightText: OpenTalk Team <mail@opentalk.eu>

use std::sync::Arc;

use serde::{Deserialize, Serialize, Serializer};
use serde_json::Value;

/// Type to deal with opaque JSON values.
///
/// Some scenarios require sending the same value to a large amount of participants,
/// which is why the value is reference counted and therefore cheap to clone.
#[derive(Debug, Clone)]
pub struct SharedJson {
    inner: Arc<Value>,
}

impl SharedJson {
    pub fn clone_inner(&self) -> Value {
        (*self.inner).clone()
    }
}

impl From<Value> for SharedJson {
    fn from(value: Value) -> Self {
        Self {
            inner: Arc::new(value),
        }
    }
}

impl Serialize for SharedJson {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        Value::serialize(&*self.inner, serializer)
    }
}

impl<'de> Deserialize<'de> for SharedJson {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        Value::deserialize(deserializer).map(Self::from)
    }
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;

    use insta::assert_snapshot;

    use crate::shared_json::SharedJson;

    #[test]
    fn serialize() {
        let json = serde_json::to_value(BTreeMap::from([("test", 1), ("b", 2)])).unwrap();

        // Insta does not use serde_json for it's json tests
        let raw = serde_json::to_string(&SharedJson::from(json)).unwrap();
        assert_snapshot!(raw, @r#"{"b":2,"test":1}"#);
    }
}
