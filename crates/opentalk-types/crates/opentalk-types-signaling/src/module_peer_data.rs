// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use std::collections::BTreeMap;

use opentalk_types_common::modules::ModuleId;
use serde::{Deserialize, Serialize};

use crate::SignalingModulePeerFrontendData;

/// A struct containing data of a peer for multiple signaling modules, each
/// associated with the module's namespace.
#[derive(Default, Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct ModulePeerData(BTreeMap<ModuleId, serde_json::Value>);

impl ModulePeerData {
    /// Create a new empty [`ModulePeerData`].
    pub fn new() -> Self {
        Self(BTreeMap::default())
    }

    /// Get the peer frontend data for a specific module
    pub fn get<T: SignalingModulePeerFrontendData>(&self) -> Result<Option<T>, serde_json::Error> {
        if let Some(namespace) = T::NAMESPACE {
            self.0
                .get(&namespace)
                .map(|m| serde_json::from_value(m.clone()))
                .transpose()
        } else {
            Ok(None)
        }
    }

    /// Set the peer frontend data for a specific module
    ///
    /// If an entry with the namespace already exists, it will be overwritten.
    /// If the namespace of `T` is [`None`], the data will not be stored at all.
    pub fn insert<T: SignalingModulePeerFrontendData>(
        &mut self,
        data: &T,
    ) -> Result<(), serde_json::Error> {
        if let Some(namespace) = T::NAMESPACE {
            let _ = self.0.insert(namespace, serde_json::to_value(data)?);
        }
        Ok(())
    }

    /// Updates the module data and returns the new data
    pub fn update<T: SignalingModulePeerFrontendData, F: FnOnce(&mut T)>(
        &mut self,
        update: F,
    ) -> Result<Option<T>, serde_json::Error> {
        if let Some(mut data) = self.get::<T>()? {
            update(&mut data);
            self.insert(&data)?;
            return Ok(Some(data));
        }

        Ok(None)
    }

    /// Query whether the module data contains a value for this key
    pub fn contains_key(&self, key: &ModuleId) -> bool {
        self.0.contains_key(key)
    }

    /// Remove an entry by the namespace of [`SignalingModulePeerFrontendData`] type.
    ///
    /// This method does not verify that the data actually can be deserialized
    /// into the requested types, it just uses its namespace and removes an
    /// entry with that namespace if it exists.
    pub fn remove<T: SignalingModulePeerFrontendData>(&mut self) {
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
    pub fn take<T: SignalingModulePeerFrontendData>(
        &mut self,
    ) -> Result<Option<T>, serde_json::Error> {
        let entry = self.get::<T>()?;
        self.remove::<T>();
        Ok(entry)
    }
}

#[cfg(test)]
mod tests {
    use opentalk_types_common::modules::{ModuleId, module_id};
    use serde::{Deserialize, Serialize};

    // NOTE: Clippy is not understanding that the import is necessary
    #[allow(unused_imports)]
    use crate::ModulePeerData;
    use crate::SignalingModulePeerFrontendData;

    #[derive(Debug, Eq, PartialEq, Deserialize, Serialize)]
    struct TestState {
        flag: bool,
    }

    impl SignalingModulePeerFrontendData for TestState {
        const NAMESPACE: Option<ModuleId> = Some(module_id!("test"));
    }

    #[test]
    fn update_should_update_on_existing_data() {
        let mut module_data: ModulePeerData = ModulePeerData::new();
        let old_state = TestState { flag: false };
        module_data.insert(&old_state).unwrap();

        let new_state = module_data
            .update::<TestState, _>(|state| {
                state.flag = !old_state.flag;
            })
            .expect("update call should work without errors")
            .expect("data should be returned");

        assert_ne!(new_state, old_state);
    }

    #[test]
    fn update_should_not_update_on_missing_data() {
        let mut module_data: ModulePeerData = ModulePeerData::new();

        let new_state = module_data
            .update::<TestState, _>(|_| {
                panic!("this should not be called");
            })
            .expect("update call should work without errors");

        assert!(new_state.is_none());
    }
}
