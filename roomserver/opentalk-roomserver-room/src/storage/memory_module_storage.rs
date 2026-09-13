// SPDX-License-Identifier: EUPL-1.2
// SPDX-FileCopyrightText: OpenTalk Team <mail@opentalk.eu>

use anyhow::Result;
use async_trait::async_trait;
use json_patch::{
    AddOperation, CopyOperation, MoveOperation, Patch, PatchOperation, RemoveOperation,
    ReplaceOperation, TestOperation, patch,
};
use opentalk_roomserver_signaling::storage::module_resources::{
    Error, provider::ModuleResourceProvider,
};
use opentalk_types_api_internal::module_resources::{
    ModuleResource, ModuleResourceFilter, ModuleResourceOperation, NewModuleResource,
};
use opentalk_types_common::{
    module_resources::ModuleResourceId, tenants::TenantId, time::Timestamp,
};
use tokio::sync::RwLock;

/// An in-memory implementation for the [`ModuleResourceProvider`]
#[derive(Debug, Default)]
pub struct MemoryModuleResourceStorage {
    module_resources: RwLock<Vec<ModuleResource>>,
}

impl MemoryModuleResourceStorage {
    pub fn new() -> Self {
        Self {
            module_resources: RwLock::new(Vec::new()),
        }
    }

    #[cfg(test)]
    pub fn new_with_entries(entries: Vec<ModuleResource>) -> Self {
        Self {
            module_resources: RwLock::new(entries),
        }
    }
}

#[async_trait]
impl ModuleResourceProvider for MemoryModuleResourceStorage {
    async fn create(&self, resource: NewModuleResource) -> Result<ModuleResource, Error> {
        let id = ModuleResourceId::generate();
        let now = Timestamp::now();

        let NewModuleResource {
            room_id,
            created_by,
            namespace,
            tag,
            data,
        } = resource;

        let resource = ModuleResource {
            id,
            tenant_id: TenantId::nil(),
            room_id,
            created_by,
            created_at: now,
            updated_at: now,
            namespace,
            tag,
            data,
        };

        let mut resources = self.module_resources.write().await;

        resources.push(resource.clone());

        Ok(resource)
    }

    async fn get(&self, filter: ModuleResourceFilter) -> Result<Vec<ModuleResource>, Error> {
        let resources = self.module_resources.read().await;

        Ok(resources
            .iter()
            .filter(|m| filter.applies_to(m))
            .cloned()
            .collect())
    }

    async fn patch(
        &self,
        filter: ModuleResourceFilter,
        operations: Vec<ModuleResourceOperation>,
    ) -> Result<Vec<ModuleResource>, Error> {
        let mut resources = self.module_resources.write().await;

        let patch_operations = Patch(
            operations
                .into_iter()
                .map(to_patch_operation)
                .collect::<Result<Vec<_>>>()
                .map_err(|e| Error::PatchFailed {
                    msg: format!("{e:?}"),
                })?,
        );

        let mut patched_resources: Vec<(usize, ModuleResource)> = resources
            .iter()
            .cloned()
            .enumerate()
            .filter(|(_, resource)| filter.applies_to(resource))
            .collect();

        for (_, resource) in &mut patched_resources {
            patch(&mut resource.data, &patch_operations).map_err(|e| Error::PatchFailed {
                msg: format!("{e:?}"),
            })?;
        }

        for (index, resource) in &patched_resources {
            resources[*index] = resource.clone();
        }

        Ok(patched_resources
            .into_iter()
            .map(|(_, resource)| resource)
            .collect())
    }

    async fn delete(&self, filter: ModuleResourceFilter) -> Result<Vec<ModuleResource>, Error> {
        let mut resources = self.module_resources.write().await;

        let removed_resources = resources
            .extract_if(.., |resource| filter.applies_to(resource))
            .collect();

        Ok(removed_resources)
    }
}

fn to_patch_operation(operation: ModuleResourceOperation) -> Result<PatchOperation> {
    Ok(match operation {
        ModuleResourceOperation::Add { path, value } => PatchOperation::Add(AddOperation {
            path: path.parse()?,
            value,
        }),
        ModuleResourceOperation::Remove { path } => PatchOperation::Remove(RemoveOperation {
            path: path.parse()?,
        }),
        ModuleResourceOperation::Replace { path, value } => {
            PatchOperation::Replace(ReplaceOperation {
                path: path.parse()?,
                value,
            })
        }
        ModuleResourceOperation::Move { from, path } => PatchOperation::Move(MoveOperation {
            from: from.parse()?,
            path: path.parse()?,
        }),
        ModuleResourceOperation::Copy { from, path } => PatchOperation::Copy(CopyOperation {
            from: from.parse()?,
            path: path.parse()?,
        }),
        ModuleResourceOperation::Test { path, value } => PatchOperation::Test(TestOperation {
            path: path.parse()?,
            value,
        }),
    })
}

#[cfg(test)]
mod tests {
    use opentalk_roomserver_signaling::storage::module_resources::provider::ModuleResourceProvider;
    use opentalk_types_common::{
        module_resources::ModuleResourceId,
        modules::{ModuleId, module_id},
        rooms::RoomId,
        tenants::TenantId,
        time::Timestamp,
        users::UserId,
    };
    use serde_json::json;

    use super::{
        MemoryModuleResourceStorage, ModuleResource, ModuleResourceFilter, ModuleResourceOperation,
        NewModuleResource,
    };

    #[derive(Debug, Default)]
    struct ResourceBuilder {
        id: Option<ModuleResourceId>,
        room_id: Option<RoomId>,
        created_by: Option<UserId>,
        namespace: Option<ModuleId>,
        tag: Option<String>,
        data: Option<serde_json::Value>,
    }

    impl ResourceBuilder {
        fn new() -> Self {
            Self::default()
        }

        fn id(mut self, id: ModuleResourceId) -> Self {
            self.id = Some(id);
            self
        }

        fn room_id(mut self, room_id: RoomId) -> Self {
            self.room_id = Some(room_id);
            self
        }

        fn created_by(mut self, created_by: UserId) -> Self {
            self.created_by = Some(created_by);
            self
        }

        fn namespace(mut self, namespace: ModuleId) -> Self {
            self.namespace = Some(namespace);
            self
        }

        fn tag(mut self, tag: String) -> Self {
            self.tag = Some(tag);
            self
        }

        fn data(mut self, data: serde_json::Value) -> Self {
            self.data = Some(data);
            self
        }

        fn build(self) -> ModuleResource {
            let now = Timestamp::now();

            ModuleResource {
                id: self.id.unwrap_or(ModuleResourceId::generate()),
                tenant_id: TenantId::nil(),
                room_id: self.room_id.unwrap_or(RoomId::nil()),
                created_by: self.created_by.unwrap_or(UserId::nil()),
                created_at: now,
                updated_at: now,
                namespace: self.namespace.unwrap_or(module_id!("test")),
                tag: self.tag,
                data: self.data.unwrap_or(json!({})),
            }
        }
    }

    /// Create a new storage with the given module resource entries
    async fn test_storage(
        resources: impl IntoIterator<Item = ResourceBuilder>,
    ) -> MemoryModuleResourceStorage {
        let resources = resources
            .into_iter()
            .map(|builder| builder.build())
            .collect();

        MemoryModuleResourceStorage::new_with_entries(resources)
    }

    #[test_log::test(tokio::test)]
    async fn create() {
        let storage = test_storage([ResourceBuilder::new().tag("ignored".into())]).await;

        let module_resource = storage
            .create(NewModuleResource {
                room_id: RoomId::nil(),
                created_by: UserId::nil(),
                namespace: module_id!("test"),
                tag: None,
                data: json!({"foo":"bar"}),
            })
            .await
            .unwrap();

        let response = storage
            .get(ModuleResourceFilter::default().id(module_resource.id))
            .await
            .unwrap();

        assert_eq!(response.len(), 1);
        assert_eq!(module_resource, response[0])
    }

    #[test_log::test(tokio::test)]
    async fn filter_by_id() {
        let resource_id = ModuleResourceId::from_u128(123);

        let resources = [
            ResourceBuilder::new().id(resource_id),
            ResourceBuilder::new().id(ModuleResourceId::from_u128(456)),
        ];

        let storage = test_storage(resources).await;

        let id_filter = ModuleResourceFilter::default().id(resource_id);

        let response = storage.get(id_filter).await.unwrap();

        assert_eq!(response.len(), 1);
        assert_eq!(response[0].id, resource_id);
    }

    #[test_log::test(tokio::test)]
    async fn filter_by_room() {
        let room_id = RoomId::from_u128(1);

        let resources = [
            ResourceBuilder::new()
                .room_id(room_id)
                .tag("test_resource".into()),
            ResourceBuilder::new()
                .room_id(room_id)
                .tag("test_resource".into()),
            ResourceBuilder::new().room_id(RoomId::from_u128(2)),
            ResourceBuilder::new().room_id(RoomId::from_u128(3)),
        ];

        let storage = test_storage(resources).await;

        // Get all resources for room 1
        let room_filter = ModuleResourceFilter::default().room_id(room_id);
        let response = storage.get(room_filter).await.unwrap();

        assert_eq!(response.len(), 2);
        for resource in response {
            assert_eq!(resource.room_id, room_id);
            assert_eq!(resource.tag, Some("test_resource".into()));
        }
    }

    #[test_log::test(tokio::test)]
    async fn filter_by_user() {
        let user_id = UserId::from_u128(1);

        let resources = [
            ResourceBuilder::new().room_id(RoomId::from_u128(2)),
            ResourceBuilder::new().room_id(RoomId::from_u128(3)),
            ResourceBuilder::new()
                .created_by(user_id)
                .tag("test_resource".into()),
            ResourceBuilder::new()
                .created_by(user_id)
                .tag("test_resource".into()),
        ];

        let storage = test_storage(resources).await;

        // Get all resources for the specific user
        let room_filter = ModuleResourceFilter::default().created_by(user_id);
        let response = storage.get(room_filter).await.unwrap();

        assert_eq!(response.len(), 2);
        for resource in response {
            assert_eq!(resource.created_by, user_id);
            assert_eq!(resource.tag, Some("test_resource".into()));
        }
    }

    #[test_log::test(tokio::test)]
    async fn filter_by_room_and_namespace() {
        let room_id = RoomId::from_u128(1);
        let namespace_foo = module_id!("foo");

        let room_id2 = RoomId::from_u128(2);
        let namespace_bar = module_id!("bar");

        let resources = [
            ResourceBuilder::new()
                .room_id(room_id)
                .namespace(namespace_foo.clone()),
            ResourceBuilder::new()
                .room_id(room_id)
                .namespace(namespace_foo.clone()),
            ResourceBuilder::new()
                .room_id(RoomId::from_u128(2))
                .namespace(namespace_bar.clone()),
            ResourceBuilder::new()
                .room_id(RoomId::from_u128(3))
                .namespace(module_id!("baz")),
        ];
        let storage = test_storage(resources).await;

        // Get resources for room 1 and namespace foo
        let filter = ModuleResourceFilter::new(room_id, namespace_foo.clone());
        let response = storage.get(filter).await.unwrap();

        assert_eq!(response.len(), 2);
        for resource in response {
            assert_eq!(resource.room_id, room_id);
            assert_eq!(resource.namespace, namespace_foo);
        }

        // Get resources for room 2 and namespace bar
        let filter = ModuleResourceFilter::new(room_id2, namespace_bar.clone());
        let response = storage.get(filter).await.unwrap();

        assert_eq!(response.len(), 1);
        for resource in response {
            assert_eq!(resource.room_id, room_id2);
            assert_eq!(resource.namespace, namespace_bar);
        }
    }

    #[test_log::test(tokio::test)]
    async fn delete() {
        let resource_id = ModuleResourceId::from_u128(1);

        let resources = [
            ResourceBuilder::new()
                .id(resource_id)
                .tag("test_resource".into()),
            ResourceBuilder::new(),
        ];

        let storage = test_storage(resources).await;

        // Ensure there are two resources in the storage
        let response = storage.get(ModuleResourceFilter::default()).await.unwrap();
        assert_eq!(response.len(), 2);

        // Delete a resource by id
        let id_filter = ModuleResourceFilter::default().id(resource_id);
        let response = storage.delete(id_filter).await.unwrap();
        assert_eq!(response.len(), 1);
        assert_eq!(response[0].id, resource_id);
        assert_eq!(response[0].tag, Some("test_resource".into()));

        // Ensure there is only one resource left in the storage
        let response = storage.get(ModuleResourceFilter::default()).await.unwrap();
        assert_eq!(response.len(), 1);
    }

    #[test_log::test(tokio::test)]
    async fn patch_add() {
        let resource_id = ModuleResourceId::from_u128(1);
        let data = json!({"foo": 1});

        let resources = [ResourceBuilder::new().id(resource_id).data(data.clone())];

        let storage = test_storage(resources).await;

        // Ensure the initial json value is correct
        let response = storage
            .get(ModuleResourceFilter::default().id(resource_id))
            .await
            .unwrap();
        assert_eq!(response.len(), 1);
        assert_eq!(response[0].data, data);

        // Add a 'bar' field to the resource
        let id_filter = ModuleResourceFilter::default().id(resource_id);
        let operations = vec![ModuleResourceOperation::Add {
            path: "/bar".into(),
            value: json!(2),
        }];
        let response = storage.patch(id_filter, operations).await.unwrap();

        assert_eq!(response.len(), 1);
        assert_eq!(
            response[0].data,
            json!({
                "foo": 1,
                "bar": 2,
            })
        );
    }

    #[test_log::test(tokio::test)]
    async fn patch_add_root() {
        let resource_id = ModuleResourceId::from_u128(1);
        let data = json!({"foo": 1});

        let resources = [ResourceBuilder::new().id(resource_id).data(data.clone())];

        let storage = test_storage(resources).await;

        // Ensure the initial json value is correct
        let response = storage
            .get(ModuleResourceFilter::default().id(resource_id))
            .await
            .unwrap();
        assert_eq!(response.len(), 1);
        assert_eq!(response[0].data, data);

        // Replace the entire document with a new one
        let id_filter = ModuleResourceFilter::default().id(resource_id);
        let operations = vec![ModuleResourceOperation::Add {
            path: "".into(),
            value: json!({"bar": 2, "baz": 3}),
        }];
        let response = storage.patch(id_filter, operations).await.unwrap();

        assert_eq!(response.len(), 1);
        assert_eq!(
            response[0].data,
            json!({
                "bar": 2,
                "baz": 3,
            })
        );
    }

    #[test_log::test(tokio::test)]
    async fn patch_remove() {
        let resource_id = ModuleResourceId::from_u128(1);
        let data = json!({
            "foo": 1,
            "bar": 2,
        });

        let resources = [ResourceBuilder::new().id(resource_id).data(data.clone())];

        let storage = test_storage(resources).await;

        // Ensure the initial json value is correct
        let response = storage
            .get(ModuleResourceFilter::default().id(resource_id))
            .await
            .unwrap();
        assert_eq!(response.len(), 1);
        assert_eq!(response[0].data, data);

        // Remove the 'bar' field to the resource
        let id_filter = ModuleResourceFilter::default().id(resource_id);
        let operations = vec![ModuleResourceOperation::Remove {
            path: "/bar".into(),
        }];
        let response = storage.patch(id_filter, operations).await.unwrap();

        assert_eq!(response.len(), 1);
        assert_eq!(
            response[0].data,
            json!({
                "foo": 1,
            })
        );
    }

    #[test_log::test(tokio::test)]
    async fn patch_replace() {
        let resource_id = ModuleResourceId::from_u128(1);
        let data = json!({
            "foo": 1,
        });

        let resources = [ResourceBuilder::new().id(resource_id).data(data.clone())];

        let storage = test_storage(resources).await;

        // Ensure the initial json value is correct
        let response = storage
            .get(ModuleResourceFilter::default().id(resource_id))
            .await
            .unwrap();
        assert_eq!(response.len(), 1);
        assert_eq!(response[0].data, data);

        // Replace the value of the 'foo' field with  "bar"
        let id_filter = ModuleResourceFilter::default().id(resource_id);
        let operations = vec![ModuleResourceOperation::Replace {
            path: "/foo".into(),
            value: json!("bar"),
        }];
        let response = storage.patch(id_filter, operations).await.unwrap();

        assert_eq!(response.len(), 1);
        assert_eq!(
            response[0].data,
            json!({
                "foo": "bar",
            })
        );
    }

    #[test_log::test(tokio::test)]
    async fn patch_multiple_operations() {
        let resource_id = ModuleResourceId::from_u128(1);
        let data = json!({
            "foo": 1,
            "bar": 2,
        });

        let resources = [ResourceBuilder::new().id(resource_id).data(data.clone())];

        let storage = test_storage(resources).await;

        // Ensure the initial json value is correct
        let response = storage
            .get(ModuleResourceFilter::default().id(resource_id))
            .await
            .unwrap();
        assert_eq!(response.len(), 1);
        assert_eq!(response[0].data, data);

        // Remove the 'foo' field and add the 'baz' field
        let id_filter = ModuleResourceFilter::default().id(resource_id);
        let operations = vec![
            ModuleResourceOperation::Remove {
                path: "/foo".into(),
            },
            ModuleResourceOperation::Add {
                path: "/baz".into(),
                value: json!(3),
            },
        ];
        let response = storage.patch(id_filter, operations).await.unwrap();

        assert_eq!(response.len(), 1);
        assert_eq!(
            response[0].data,
            json!({
                "bar": 2,
                "baz": 3
            })
        );
    }

    #[test_log::test(tokio::test)]
    async fn patch_multiple_resources() {
        let data = json!({
            "foo": 1,
            "bar": 2,
        });

        let resources = [
            ResourceBuilder::new().data(data.clone()),
            ResourceBuilder::new().data(data.clone()),
        ];

        let storage = test_storage(resources).await;

        // Ensure the initial json value is correct
        let response = storage.get(ModuleResourceFilter::default()).await.unwrap();
        assert_eq!(response.len(), 2);
        assert_eq!(response[0].data, data);
        assert_eq!(response[1].data, data);

        // Remove the 'foo' field and add the 'baz' field
        let filter = ModuleResourceFilter::default();
        let operations = vec![
            ModuleResourceOperation::Remove {
                path: "/foo".into(),
            },
            ModuleResourceOperation::Add {
                path: "/baz".into(),
                value: json!(3),
            },
        ];
        let response = storage.patch(filter, operations).await.unwrap();

        assert_eq!(response.len(), 2);
        assert_eq!(
            response[0].data,
            json!({
                "bar": 2,
                "baz": 3
            })
        );

        assert_eq!(
            response[1].data,
            json!({
                "bar": 2,
                "baz": 3
            })
        );
    }
}
