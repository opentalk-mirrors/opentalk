// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use opentalk_types_common_identifiers::{
    asset_file_kind::AssetFileKind, feature_id::FeatureId, module_id::ModuleId,
};

/// A trait for providing example data of an item.
pub trait ExampleData {
    /// Get an example instance of the current datatype.
    fn example_data() -> Self;
}

impl ExampleData for FeatureId {
    fn example_data() -> Self {
        FeatureId::example_data()
    }
}

impl ExampleData for ModuleId {
    fn example_data() -> Self {
        ModuleId::example_data()
    }
}

impl ExampleData for AssetFileKind {
    fn example_data() -> Self {
        AssetFileKind::example_data()
    }
}

#[cfg(test)]
mod tests {
    use opentalk_types_common_identifiers::{
        asset_file_kind::AssetFileKind, feature_id::FeatureId, module_id::ModuleId,
    };

    #[test]
    fn feature_id_example_data() {
        assert_eq!(
            FeatureId::example_data(),
            <FeatureId as super::ExampleData>::example_data()
        );
    }

    #[test]
    fn module_id_example_data() {
        assert_eq!(
            ModuleId::example_data(),
            <ModuleId as super::ExampleData>::example_data()
        );
    }

    #[test]
    fn asset_file_kind_example_data() {
        assert_eq!(
            AssetFileKind::example_data(),
            <AssetFileKind as super::ExampleData>::example_data()
        );
    }
}
