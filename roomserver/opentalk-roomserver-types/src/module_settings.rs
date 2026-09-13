// SPDX-License-Identifier: EUPL-1.2
// SPDX-FileCopyrightText: OpenTalk Team <mail@opentalk.eu>

use std::{collections::BTreeMap, fmt::Debug};

use opentalk_types_common::{
    modules::{ModuleId, module_id},
    utils::ExampleData,
};
use serde::{Deserialize, Serialize, de::DeserializeOwned};
use serde_json::json;

pub trait SignalingModuleSettings: ExampleData + Serialize + DeserializeOwned + Debug {
    const NAMESPACE: ModuleId;
}

/// A struct containing settings for multiple signaling modules, each associated with the module's
/// namespace.
#[derive(Default, Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema), schema(example = json!(ModuleSettings::example_data())))]
pub struct ModuleSettings(BTreeMap<ModuleId, serde_json::Value>);

impl ModuleSettings {
    /// Create a new empty [`ModuleSettings`].
    pub fn new() -> Self {
        Self(BTreeMap::new())
    }

    /// Get the settings for a specific module
    pub fn get<T: SignalingModuleSettings>(&self) -> Result<Option<T>, serde_json::Error> {
        self.0
            .get(&T::NAMESPACE)
            .map(|m| serde_json::from_value(m.clone()))
            .transpose()
    }

    /// Set the settings for a specific module
    ///
    /// If an entry with the namespace already exists, it will be overwritten.
    pub fn insert<T: SignalingModuleSettings>(
        &mut self,
        data: &T,
    ) -> Result<(), serde_json::Error> {
        self.0.insert(T::NAMESPACE, serde_json::to_value(data)?);
        Ok(())
    }

    /// Insert an empty object for the specified module namespace
    ///
    /// This is useful for adding modules that don't require any settings.
    pub fn insert_empty(&mut self, namespace: ModuleId) {
        self.0
            .insert(namespace, serde_json::Value::Object(serde_json::Map::new()));
    }

    /// Remove the settings for a specified module.
    pub fn remove(&mut self, namespace: &ModuleId) -> Option<serde_json::Value> {
        self.0.remove(namespace)
    }

    /// Retains only the entries specified by the predicate
    pub fn retain<F>(&mut self, f: F)
    where
        F: FnMut(&ModuleId, &mut serde_json::Value) -> bool,
    {
        self.0.retain(f);
    }

    /// Returns an iterator over the module IDs in the settings
    pub fn ids(&self) -> impl Iterator<Item = &ModuleId> {
        self.0.keys()
    }

    /// Checks if specified module id is present in the settings
    pub fn contains(&self, namespace: ModuleId) -> bool {
        self.0.contains_key(&namespace)
    }
}

impl ExampleData for ModuleSettings {
    fn example_data() -> Self {
        Self(BTreeMap::from([(
            module_id!("livekit"),
            json!({
                "public_url": "http://localhost:7880",
                "service_url": "http://localhost:7880",
                "api_key": "devkey",
                "api_secret": "secret",
            }),
        )]))
    }
}
