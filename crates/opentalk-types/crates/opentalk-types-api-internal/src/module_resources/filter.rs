// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use opentalk_types_common::{
    module_resources::ModuleResourceId, modules::ModuleId, rooms::RoomId, users::UserId,
};
use serde::{Deserialize, Serialize};

use crate::module_resources::ModuleResource;

/// The filter for the internal module resource API
///
/// A blank filter will be declined by the module resource API
#[derive(Default, Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
pub struct ModuleResourceFilter {
    /// Filter by the associated room id
    pub room_id: Option<RoomId>,
    /// Filter by namespace
    pub namespace: Option<ModuleId>,
    /// Filter by id
    pub id: Option<ModuleResourceId>,
    /// Filter by creator of the resource
    pub created_by: Option<UserId>,
    /// Filter by tag
    pub tag: Option<String>,
}

impl ModuleResourceFilter {
    /// Create a new resource filter
    pub fn new(room_id: RoomId, namespace: ModuleId) -> Self {
        Self {
            room_id: Some(room_id),
            namespace: Some(namespace),
            id: None,
            created_by: None,
            tag: None,
        }
    }

    /// Set the module resource id  of the filter
    pub fn id(mut self, id: ModuleResourceId) -> Self {
        self.id = Some(id);
        self
    }

    /// Set the room_id of the filter
    pub fn room_id(mut self, room_id: RoomId) -> Self {
        self.room_id = Some(room_id);
        self
    }

    /// Set the created_by of the filter
    pub fn created_by(mut self, user_id: UserId) -> Self {
        self.created_by = Some(user_id);
        self
    }

    /// Set the namespace of the filter
    pub fn namespace(mut self, namespace: ModuleId) -> Self {
        self.namespace = Some(namespace);
        self
    }

    /// Set the tag of the filter
    pub fn tag(mut self, tag: Option<String>) -> Self {
        self.tag = tag;
        self
    }

    /// Returns true when none of the filter values are set
    pub fn is_empty(&self) -> bool {
        self.room_id.is_none()
            && self.namespace.is_none()
            && self.id.is_none()
            && self.created_by.is_none()
            && self.tag.is_none()
    }

    /// Returns true when all values of the filter match the given [`ModuleResource`] values
    pub fn applies_to(&self, resource: &ModuleResource) -> bool {
        if let Some(id) = &self.id
            && &resource.id != id
        {
            return false;
        }

        if let Some(room_id) = &self.room_id
            && &resource.room_id != room_id
        {
            return false;
        }

        if let Some(created_by) = &self.created_by
            && &resource.created_by != created_by
        {
            return false;
        }

        if let Some(namespace) = &self.namespace
            && &resource.namespace != namespace
        {
            return false;
        }

        if let Some(tag) = &self.tag
            && resource.tag.as_ref() != Some(tag)
        {
            return false;
        }

        true
    }
}
