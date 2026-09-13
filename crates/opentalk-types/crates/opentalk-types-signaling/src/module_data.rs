// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use std::collections::BTreeMap;

use opentalk_types_common::{
    modules::{ModuleId, module_id},
    utils::ExampleData,
};
use serde::{Deserialize, Serialize};

use crate::SignalingModuleFrontendData;

/// A struct containing data for multiple signaling modules, each associated
/// with the module's namespace.
#[derive(Default, Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
#[cfg_attr(
    feature = "utoipa",
    derive(utoipa::ToSchema),
    schema(example = json!(ModuleData::example_data()))
)]
pub struct ModuleData(BTreeMap<ModuleId, serde_json::Value>);

impl ModuleData {
    /// Create a new empty [`ModuleData`].
    pub fn new() -> Self {
        Self(BTreeMap::default())
    }

    /// Get an iterator over all module data entries
    pub fn iter(&self) -> impl Iterator<Item = (&ModuleId, &serde_json::Value)> {
        self.0.iter()
    }

    /// Get the participant data for a specific module
    pub fn get<T: SignalingModuleFrontendData>(&self) -> Result<Option<T>, serde_json::Error> {
        if let Some(namespace) = T::NAMESPACE {
            self.0
                .get(&namespace)
                .map(|m| serde_json::from_value(m.clone()))
                .transpose()
        } else {
            Ok(None)
        }
    }

    /// Set the participant data for a specific module
    ///
    /// If an entry with the namespace already exists, it will be overwritten.
    /// If the namespace of `T` is [`None`], the data will not be stored at all.
    pub fn insert<T: SignalingModuleFrontendData>(
        &mut self,
        data: &T,
    ) -> Result<(), serde_json::Error> {
        if let Some(namespace) = T::NAMESPACE {
            let _ = self.0.insert(namespace, serde_json::to_value(data)?);
        }
        Ok(())
    }

    /// Query whether the module data contains a value for this key
    pub fn contains_key(&self, key: &ModuleId) -> bool {
        self.0.contains_key(key)
    }

    /// Remove an entry by the namespace of [`SignalingModuleFrontendData`] type.
    ///
    /// This method does not verify that the data actually can be deserialized
    /// into the requested types, it just uses its namespace and removes an
    /// entry with that namespace if it exists.
    pub fn remove<T: SignalingModuleFrontendData>(&mut self) {
        if let Some(namespace) = T::NAMESPACE {
            self.remove_key(&namespace);
        }
    }

    /// Remove an entry by its key if it exists.
    pub fn remove_key(&mut self, key: &ModuleId) {
        let _ = self.0.remove(key);
    }

    /// Take an entry by the namespace of a type.
    ///
    /// If the entry is present but can not be deserialized into the requested type,
    /// the data will remain unchanged, the entry is not removed.
    pub fn take<T: SignalingModuleFrontendData>(&mut self) -> Result<Option<T>, serde_json::Error> {
        let entry = self.get::<T>()?;
        self.remove::<T>();
        Ok(entry)
    }

    /// Retains only the entries specified by the predicate
    pub fn retain<F>(&mut self, f: F)
    where
        F: FnMut(&ModuleId, &mut serde_json::Value) -> bool,
    {
        self.0.retain(f);
    }
}

impl ExampleData for ModuleData {
    fn example_data() -> Self {
        Self(BTreeMap::from([(
            module_id!("livekit"),
            serde_json::json!({
                "public_url": "http://localhost:7880",
                "service_url": "http://localhost:7880",
                "api_key": "devkey",
                "api_secret": "secret",
            }),
        )]))
    }
}
