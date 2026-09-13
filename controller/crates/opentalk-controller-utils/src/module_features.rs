// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use std::collections::{BTreeMap, BTreeSet};

use opentalk_inventory::{Inventory, Room};
use opentalk_types_api_common::error::ApiError;
use opentalk_types_common::{
    features::{FeatureId, ModuleFeatureId},
    modules::ModuleId,
    tariffs::TariffResource,
    users::UserId,
};
use snafu::{Snafu, ensure};

use crate::CaptureApiError;

/// Error that is returned when a required feature is disabled, either by configuration or by tariff.
#[derive(Debug, Snafu)]
#[snafu(display("The feature \"{feature}\" is disabled"))]
pub struct FeatureRequiredError {
    /// The required feature that is disabled.
    pub feature: ModuleFeatureId,
}

impl From<FeatureRequiredError> for ApiError {
    fn from(e: FeatureRequiredError) -> Self {
        ApiError::forbidden()
            .with_code("feature_disabled")
            .with_message(e.to_string())
    }
}

impl From<FeatureRequiredError> for CaptureApiError {
    fn from(e: FeatureRequiredError) -> Self {
        ApiError::from(e).into()
    }
}

/// Extension functions for [`TariffResource`].
pub trait TariffResourceExt {
    /// Require a feature to be present, fail with an [`ApiError`] instead.
    fn require_feature(&self, feature: &ModuleFeatureId) -> Result<(), FeatureRequiredError>;
}

impl TariffResourceExt for TariffResource {
    fn require_feature(&self, feature: &ModuleFeatureId) -> Result<(), FeatureRequiredError> {
        ensure!(
            self.has_feature_enabled(&feature.module, &feature.feature),
            FeatureRequiredSnafu {
                feature: feature.clone()
            }
        );
        Ok(())
    }
}

/// Get the tariff for a room based on the set of disabled features and available module features.
pub async fn get_tariff_for_room(
    inventory: &mut dyn Inventory,
    room: &Room,
    disabled_features: BTreeSet<ModuleFeatureId>,
    module_features: BTreeMap<ModuleId, BTreeSet<FeatureId>>,
) -> Result<TariffResource, opentalk_inventory::Error> {
    get_tariff_for_user(
        inventory,
        room.created_by,
        disabled_features,
        module_features,
    )
    .await
}

/// Get the tariff for a user based on the set of disabled features and available module features.
pub async fn get_tariff_for_user(
    inventory: &mut dyn Inventory,
    user_id: UserId,
    disabled_features: BTreeSet<ModuleFeatureId>,
    module_features: BTreeMap<ModuleId, BTreeSet<FeatureId>>,
) -> Result<TariffResource, opentalk_inventory::Error> {
    let tariff = inventory.get_tariff_for_user(user_id).await?;

    Ok(tariff.to_tariff_resource(disabled_features, module_features))
}
