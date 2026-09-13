// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use opentalk_types_common::{module_resources::ModuleResourceId, rooms::RoomId, users::UserId};

/// A helper struct for filtering module resources
#[derive(Default, Debug, Clone, PartialEq, Eq)]
pub struct ModuleResourceFilter {
    /// Filter by id
    pub id: Option<ModuleResourceId>,
    /// FIlter by room id
    pub room_id: Option<RoomId>,
    /// Filter by namespace
    pub namespace: Option<String>,
    /// Filter by creator
    pub created_by: Option<UserId>,
    /// Filter by tag
    pub tag: Option<String>,
    /// Filter by json value
    pub json: Option<serde_json::Value>,
}

impl ModuleResourceFilter {
    /// Create a new [`ModuleResourceFilter`].
    pub fn new() -> Self {
        Self::default()
    }

    /// Filter by the id of the module resource
    pub fn with_id(mut self, id: ModuleResourceId) -> Self {
        self.id = Some(id);

        self
    }

    /// Filter by the room id of the module resource
    pub fn with_room_id(mut self, room_id: RoomId) -> Self {
        self.room_id = Some(room_id);

        self
    }

    /// Filter by the namespace of the module resource
    pub fn with_namespace(mut self, namespace: String) -> Self {
        self.namespace = Some(namespace);

        self
    }

    /// Filter by the creator of the module resource
    pub fn with_created_by(mut self, created_by: UserId) -> Self {
        self.created_by = Some(created_by);

        self
    }

    /// Filter by the tag of the module resource
    pub fn with_tag(mut self, tag: String) -> Self {
        self.tag = Some(tag);

        self
    }

    /// Filter by the content of the module resource
    pub fn with_json(mut self, json: serde_json::Value) -> Self {
        self.json = Some(json);

        self
    }
}

impl From<ModuleResourceFilter>
    for (
        Option<ModuleResourceId>,
        Option<RoomId>,
        Option<String>,
        Option<UserId>,
        Option<String>,
        Option<serde_json::Value>,
    )
{
    fn from(
        ModuleResourceFilter {
            id,
            room_id,
            namespace,
            created_by,
            tag,
            json,
        }: ModuleResourceFilter,
    ) -> Self {
        (id, room_id, namespace, created_by, tag, json)
    }
}
