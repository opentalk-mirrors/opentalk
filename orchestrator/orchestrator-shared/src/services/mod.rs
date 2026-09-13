// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use std::collections::HashSet;

use opentalk_service_auth::ApiKeyId;
use opentalk_types_common::rooms::RoomId;
use serde::{Deserialize, Serialize};

use crate::{Metrics, ServiceKind};

pub mod recorder;
pub mod roomserver;
pub mod service_resource;
pub mod transcription;

pub use service_resource::{ServiceResource, ServiceResourceParseError};

/// Marker trait for types that can be used as managed resources within a [`ServiceState`].
pub trait ResourceType:
    Eq + std::hash::Hash + std::fmt::Debug + Sized + Into<ServiceResource>
{
    /// Returns the [`ServiceKind`] this resource type belongs to.
    fn kind() -> ServiceKind;

    /// Extracts `Self` from a [`ServiceResource`] variant, returning `None` if the variant
    /// does not match this resource type.
    fn from_service_resource(resource: ServiceResource) -> Option<Self>;
}

impl ResourceType for RoomId {
    fn kind() -> ServiceKind {
        ServiceKind::Roomserver
    }

    fn from_service_resource(resource: ServiceResource) -> Option<Self> {
        match resource {
            ServiceResource::Roomserver(room_id) => Some(room_id),
            _ => None,
        }
    }
}

/// The general data that is held by each instance
#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
pub struct InstanceData {
    /// The current metrics of the instance
    pub metrics: Metrics,
    /// Possible key ids for requests towards the service
    pub api_key_ids: Vec<ApiKeyId>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ServiceState<T: ResourceType> {
    pub instance_data: InstanceData,
    pub managed_resources: HashSet<T>,
}

impl<T: ResourceType> ServiceState<T> {
    pub fn set_metrics(&mut self, metrics: Metrics) {
        self.instance_data.metrics = metrics;
    }
}
