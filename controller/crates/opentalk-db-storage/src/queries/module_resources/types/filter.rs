// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use diesel::{ExpressionMethods, pg::Pg, prelude::*};
use opentalk_inventory as inventory;
use opentalk_types_common::{module_resources::ModuleResourceId, rooms::RoomId, users::UserId};

use crate::schema::module_resources;

/// Helper struct for filtering module resources
#[derive(Default)]
pub struct Filter {
    /// Filter by the UUID of the module resource
    id: Option<ModuleResourceId>,
    /// Filter by the room id of the module resource
    room_id: Option<RoomId>,
    /// Filter by the namespace of the module resource
    namespace: Option<String>,
    /// Filter by the creator of the module resource
    created_by: Option<UserId>,
    /// Filter by the tag of the module resource
    tag: Option<String>,
    /// Filter by the content of the module resource
    json: Option<serde_json::Value>,
}

impl From<inventory::ModuleResourceFilter> for Filter {
    fn from(value: inventory::ModuleResourceFilter) -> Self {
        let (id, room_id, namespace, created_by, tag, json) = value.into();

        let mut filter = Self::default();
        if let Some(id) = id {
            filter = filter.with_id(id);
        }
        if let Some(room_id) = room_id {
            filter = filter.with_room_id(room_id);
        }
        if let Some(namespace) = namespace {
            filter = filter.with_namespace(namespace);
        }
        if let Some(created_by) = created_by {
            filter = filter.with_created_by(created_by);
        }
        if let Some(tag) = tag {
            filter = filter.with_tag(tag);
        }
        if let Some(json) = json {
            filter = filter.with_json(json);
        }
        filter
    }
}

type ResourceFilter =
    Box<dyn BoxableExpression<module_resources::table, Pg, SqlType = diesel::sql_types::Bool>>;

impl Filter {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_id(mut self, id: ModuleResourceId) -> Self {
        self.id = Some(id);

        self
    }

    pub fn with_room_id(mut self, room_id: RoomId) -> Self {
        self.room_id = Some(room_id);

        self
    }

    pub fn with_namespace(mut self, namespace: String) -> Self {
        self.namespace = Some(namespace);

        self
    }

    pub fn with_created_by(mut self, created_by: UserId) -> Self {
        self.created_by = Some(created_by);

        self
    }

    pub fn with_tag(mut self, tag: String) -> Self {
        self.tag = Some(tag);

        self
    }

    pub fn with_json(mut self, json: serde_json::Value) -> Self {
        self.json = Some(json);

        self
    }

    pub fn into_diesel_filter(self) -> Option<ResourceFilter> {
        fn append(query: &mut Option<ResourceFilter>, filter: ResourceFilter) {
            if let Some(query_) = query.take() {
                *query = Some(Box::new(query_.and(filter)));
            } else {
                *query = Some(filter);
            }
        }

        let mut query: Option<ResourceFilter> = None;

        if let Some(id) = self.id {
            append(&mut query, Box::new(module_resources::id.eq(id)));
        }

        if let Some(room_id) = self.room_id {
            append(&mut query, Box::new(module_resources::room_id.eq(room_id)));
        }

        if let Some(namespace) = self.namespace {
            append(
                &mut query,
                Box::new(module_resources::namespace.eq(namespace)),
            );
        }

        if let Some(created_by) = self.created_by {
            append(
                &mut query,
                Box::new(module_resources::created_by.eq(created_by)),
            );
        }

        if let Some(tag) = self.tag {
            // Unwrapping the Nullable<Bool> here with an if is_not_null then compare, else false
            append(
                &mut query,
                Box::new(
                    module_resources::tag
                        .is_not_null()
                        .and(module_resources::tag.eq(tag))
                        .assume_not_null(),
                ),
            );
        }

        if let Some(json) = self.json {
            append(&mut query, Box::new(module_resources::data.contains(json)));
        }

        query
    }
}
