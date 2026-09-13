// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use opentalk_database::DbConnection;
use opentalk_db_storage as db;
use opentalk_inventory as inventory;
use opentalk_test_util::{assert_eq, database::DatabaseContext, *};
use opentalk_types_common::{module_resources::ModuleResourceId, rooms::RoomId};
use serde_json::{Value, json};

async fn init_resource(json: Value) -> (DatabaseContext, ModuleResourceId, DbConnection) {
    let db_ctx = DatabaseContext::new().await;

    let mut db_conn = db_ctx.db.get_conn().await.unwrap();

    let user = db_ctx.create_test_user(0, vec![]).await.unwrap();

    let room = db_ctx
        .create_test_room(RoomId::from_u128(0), user.id, false)
        .await
        .unwrap();

    let tenant = db::queries::tenants::get_or_create_tenant_by_oidc_id(
        &mut db_conn,
        &db::tables::tenants::OidcTenantId::from("OpenTalkDefaultTenant"),
    )
    .await
    .unwrap();

    let m = db::tables::module_resources::NewModuleResource {
        tenant_id: tenant.id,
        created_by: user.id,
        room_id: room.id,
        namespace: "test".into(),
        tag: None,
        data: json,
    };

    let resource = db::queries::module_resources::create_module_resource(&mut db_conn, m)
        .await
        .unwrap();

    (db_ctx, resource.id, db_conn)
}

async fn init_empty_resource() -> (DatabaseContext, ModuleResourceId, DbConnection) {
    init_resource(json!({})).await
}

#[actix_rt::test]
async fn test_add_single() {
    let (_db_ctx, _id, mut db_conn) = init_empty_resource().await;

    let operations = vec![inventory::ModuleResourceOperation::Add {
        path: "/foo".into(),
        value: Value::String("bar".into()),
    }];

    let mut updated = db::queries::module_resources::patch_module_resources(
        &mut db_conn,
        db::queries::module_resources::types::Filter::new().with_namespace("test".into()),
        operations,
    )
    .await
    .unwrap();
    assert_eq!(updated.len(), 1);

    assert_eq!(
        json!(updated.remove(0).data),
        json!({
            "foo": "bar"
        })
    );
}

#[actix_rt::test]
async fn test_add_to_non_empty() {
    let (_db_ctx, _id, mut db_conn) = init_resource(json!({"foo": "bar"})).await;

    let operations = vec![inventory::ModuleResourceOperation::Add {
        path: "/baz".into(),
        value: Value::String("quux".into()),
    }];

    let mut updated = db::queries::module_resources::patch_module_resources(
        &mut db_conn,
        db::queries::module_resources::types::Filter::new().with_namespace("test".into()),
        operations,
    )
    .await
    .unwrap();
    assert_eq!(updated.len(), 1);

    assert_eq!(
        json!(updated.remove(0).data),
        json!({
            "foo": "bar",
            "baz": "quux"
        })
    );
}

#[actix_rt::test]
async fn test_empty_path() {
    let (_db_ctx, _id, mut db_conn) = init_empty_resource().await;

    let operations = vec![inventory::ModuleResourceOperation::Add {
        path: "".into(),
        value: Value::String("bar".into()),
    }];

    let mut updated = db::queries::module_resources::patch_module_resources(
        &mut db_conn,
        db::queries::module_resources::types::Filter::new().with_namespace("test".into()),
        operations,
    )
    .await
    .unwrap();

    assert_eq!(json!(updated.remove(0).data), json!("bar"));
}

#[actix_rt::test]
async fn test_root_path() {
    let (_db_ctx, _id, mut db_conn) = init_empty_resource().await;

    let operations = vec![inventory::ModuleResourceOperation::Add {
        path: "/".into(),
        value: Value::String("bar".into()),
    }];

    let mut updated = db::queries::module_resources::patch_module_resources(
        &mut db_conn,
        db::queries::module_resources::types::Filter::new().with_namespace("test".into()),
        operations,
    )
    .await
    .unwrap();

    assert_eq!(json!(updated.remove(0).data), json!({"": "bar"}));
}

#[actix_rt::test]
async fn test_add_invalid() {
    let (_db_ctx, _id, mut db_conn) = init_empty_resource().await;

    let operations = vec![inventory::ModuleResourceOperation::Add {
        path: "/foo/bar/baz".into(),
        value: Value::String("bar".into()),
    }];

    match db::queries::module_resources::patch_module_resources(
        &mut db_conn,
        db::queries::module_resources::types::Filter::new().with_namespace("test".into()),
        operations,
    )
    .await
    {
        Err(db::queries::module_resources::types::JsonOperationError::JsonPatch { source }) => {
            assert_eq!(
                db::queries::module_resources::types::JsonPatchErrorCode::InvalidPath,
                source.error_code
            )
        }
        unexpected => panic!("Expected invalid_path error, got {unexpected:?}"),
    }
}

#[actix_rt::test]
async fn test_add_a_lot() {
    let (_db_ctx, _id, mut db_conn) = init_empty_resource().await;

    let mut json_compare = json!({});

    let mut operations = vec![];

    for i in 0..100 {
        let field = format!("foo{i}");

        operations.push(inventory::ModuleResourceOperation::Add {
            path: format!("/{field}"),
            value: Value::String("bar".into()),
        });

        if let Value::Object(hm) = &mut json_compare {
            hm.insert(field, "bar".into());
        }
    }

    let mut updated = db::queries::module_resources::patch_module_resources(
        &mut db_conn,
        db::queries::module_resources::types::Filter::new().with_namespace("test".into()),
        operations,
    )
    .await
    .unwrap();

    assert_eq!(updated.len(), 1);

    assert_eq!(json!(updated.remove(0).data), json_compare);
}

#[actix_rt::test]
async fn test_add_single_empty() {
    let (_db_ctx, _id, mut db_conn) = init_empty_resource().await;

    let operations = vec![inventory::ModuleResourceOperation::Add {
        path: "".into(),
        value: Value::String("bar".into()),
    }];

    let mut updated = db::queries::module_resources::patch_module_resources(
        &mut db_conn,
        db::queries::module_resources::types::Filter::new().with_namespace("test".into()),
        operations,
    )
    .await
    .unwrap();
    assert_eq!(updated.len(), 1);

    assert_eq!(json!(updated.remove(0).data), json!("bar"));
}

#[actix_rt::test]
async fn test_add_single_root() {
    let (_db_ctx, _id, mut db_conn) = init_empty_resource().await;

    let operations = vec![inventory::ModuleResourceOperation::Add {
        path: "/".into(),
        value: Value::String("bar".into()),
    }];

    let mut updated = db::queries::module_resources::patch_module_resources(
        &mut db_conn,
        db::queries::module_resources::types::Filter::new().with_namespace("test".into()),
        operations,
    )
    .await
    .unwrap();
    assert_eq!(updated.len(), 1);

    assert_eq!(json!(updated.remove(0).data), json!({"": "bar"}));
}

#[actix_rt::test]
async fn test_add_nested() {
    let (_db_ctx, _id, mut db_conn) = init_empty_resource().await;

    let operations = vec![
        inventory::ModuleResourceOperation::Add {
            path: "/foo".into(),
            value: json!({"bar": 1}),
        },
        inventory::ModuleResourceOperation::Add {
            path: "/foo/bar".into(),
            value: Value::Number(42.into()),
        },
    ];

    let mut updated = db::queries::module_resources::patch_module_resources(
        &mut db_conn,
        db::queries::module_resources::types::Filter::new().with_namespace("test".into()),
        operations,
    )
    .await
    .unwrap();
    assert_eq!(updated.len(), 1);

    assert_eq!(
        json!(updated.remove(0).data),
        json!({
            "foo": {
                "bar": 42
            }
        })
    );
}

#[actix_rt::test]
async fn test_add_to_array() {
    let initial_json = json!({
        "foo": ["a", "c"],
    });

    let (_db_ctx, _id, mut db_conn) = init_resource(initial_json).await;

    let operations = vec![inventory::ModuleResourceOperation::Add {
        path: "/foo/1".into(),
        value: Value::String("b".into()),
    }];

    let mut updated = db::queries::module_resources::patch_module_resources(
        &mut db_conn,
        db::queries::module_resources::types::Filter::new().with_namespace("test".into()),
        operations,
    )
    .await
    .unwrap();
    assert_eq!(updated.len(), 1);

    assert_eq!(
        json!(updated.remove(0).data),
        json!({
            "foo": ["a", "b", "c"]
        })
    );
}

#[actix_rt::test]
async fn test_add_array() {
    let (_db_ctx, _id, mut db_conn) = init_empty_resource().await;

    let operations = vec![inventory::ModuleResourceOperation::Add {
        path: "/foo".into(),
        value: Value::Array(vec![1.into(), 2.into(), 3.into()]),
    }];

    let mut updated = db::queries::module_resources::patch_module_resources(
        &mut db_conn,
        db::queries::module_resources::types::Filter::new().with_namespace("test".into()),
        operations,
    )
    .await
    .unwrap();
    assert_eq!(updated.len(), 1);

    assert_eq!(
        json!(updated.remove(0).data),
        json!({
            "foo": [1, 2 ,3]
        })
    );
}

#[actix_rt::test]
async fn test_add_multi() {
    let (_db_ctx, _id, mut db_conn) = init_empty_resource().await;

    let operations = vec![
        inventory::ModuleResourceOperation::Add {
            path: "/foo".into(),
            value: Value::Number(1.into()),
        },
        inventory::ModuleResourceOperation::Add {
            path: "/bar".into(),
            value: Value::Number(2.into()),
        },
    ];

    let mut updated = db::queries::module_resources::patch_module_resources(
        &mut db_conn,
        db::queries::module_resources::types::Filter::new().with_namespace("test".into()),
        operations,
    )
    .await
    .unwrap();
    assert_eq!(updated.len(), 1);

    assert_eq!(
        updated.remove(0).data,
        json!({
            "foo": 1,
            "bar": 2,
        })
    );
}

#[actix_rt::test]
async fn test_update_multi_add_with_update() {
    let (_db_ctx, _id, mut db_conn) = init_empty_resource().await;

    let operations = vec![
        inventory::ModuleResourceOperation::Add {
            path: "/foo".into(),
            value: Value::Number(1.into()),
        },
        inventory::ModuleResourceOperation::Add {
            path: "/bar".into(),
            value: Value::Number(2.into()),
        },
        inventory::ModuleResourceOperation::Add {
            path: "/foo".into(),
            value: Value::Number(42.into()),
        },
    ];

    let mut updated = db::queries::module_resources::patch_module_resources(
        &mut db_conn,
        db::queries::module_resources::types::Filter::new().with_namespace("test".into()),
        operations,
    )
    .await
    .unwrap();
    assert_eq!(updated.len(), 1);

    assert_eq!(
        json!(updated.remove(0).data),
        json!({
            "foo": 42,
            "bar": 2,
        })
    );
}

#[actix_rt::test]
async fn test_simple_remove() {
    let initial_json = json!({
        "foo": 1,
        "bar": 2
    });

    let (_db_ctx, _id, mut db_conn) = init_resource(initial_json).await;

    let operations = vec![inventory::ModuleResourceOperation::Remove {
        path: "/foo".into(),
    }];

    let mut updated = db::queries::module_resources::patch_module_resources(
        &mut db_conn,
        db::queries::module_resources::types::Filter::new().with_namespace("test".into()),
        operations,
    )
    .await
    .unwrap();
    assert_eq!(updated.len(), 1);

    assert_eq!(
        json!(updated.remove(0).data),
        json!({
            "bar": 2
        })
    );
}

#[actix_rt::test]
async fn test_remove_document() {
    let initial_json = json!({
        "foo": 1,
        "bar": 2
    });

    let (_db_ctx, _id, mut db_conn) = init_resource(initial_json).await;

    let operations = vec![inventory::ModuleResourceOperation::Remove { path: "".into() }];

    let mut updated = db::queries::module_resources::patch_module_resources(
        &mut db_conn,
        db::queries::module_resources::types::Filter::new().with_namespace("test".into()),
        operations,
    )
    .await
    .unwrap();
    assert_eq!(updated.len(), 1);

    assert_eq!(json!(updated.remove(0).data), json!({}));
}

#[actix_rt::test]
async fn test_remove_by_index() {
    let initial_json = json!({
        "foo": ["a", "b", "c"],
        "bar": {
            "0": "baz",
            "qux": "biz"
        },
    });

    let (_db_ctx, _id, mut db_conn) = init_resource(initial_json).await;

    let operations = vec![
        inventory::ModuleResourceOperation::Remove {
            path: "/foo/0".into(),
        },
        inventory::ModuleResourceOperation::Remove {
            path: "/bar/0".into(),
        },
    ];

    let mut updated = db::queries::module_resources::patch_module_resources(
        &mut db_conn,
        db::queries::module_resources::types::Filter::new().with_namespace("test".into()),
        operations,
    )
    .await
    .unwrap();
    assert_eq!(updated.len(), 1);

    assert_eq!(
        json!(updated.remove(0).data),
        json!({
            "foo": ["b", "c"],
            "bar": {
                "qux": "biz"
            },
        })
    );
}

#[actix_rt::test]
async fn test_remove_and_add_with_numbers() {
    let initial_json = json!({
        "foo": ["a", "b", "c"],
        "bar": {
            "0": "baz",
            "qux": "biz"
        },
    });

    let (_db_ctx, _id, mut db_conn) = init_resource(initial_json).await;

    let operations = vec![
        inventory::ModuleResourceOperation::Remove {
            path: "/foo/0".into(),
        },
        inventory::ModuleResourceOperation::Remove {
            path: "/bar/0".into(),
        },
        inventory::ModuleResourceOperation::Add {
            path: "/bar/0".into(),
            value: Value::Number(42.into()),
        },
    ];

    let mut updated = db::queries::module_resources::patch_module_resources(
        &mut db_conn,
        db::queries::module_resources::types::Filter::new().with_namespace("test".into()),
        operations,
    )
    .await
    .unwrap();
    assert_eq!(updated.len(), 1);

    assert_eq!(
        json!(updated.remove(0).data),
        json!({
            "foo": ["b", "c"],
            "bar": {
                "0": 42,
                "qux": "biz"
            },
        })
    );
}

#[actix_rt::test]
async fn test_simple_move() {
    let initial_json = json!({
        "foo": { "qux": 1 },
        "bar": {
            "baz": 2,
        },
    });

    let (_db_ctx, _id, mut db_conn) = init_resource(initial_json).await;

    let operations = vec![inventory::ModuleResourceOperation::Move {
        from: "/foo/qux".into(),
        path: "/bar/qux".into(),
    }];

    let mut updated = db::queries::module_resources::patch_module_resources(
        &mut db_conn,
        db::queries::module_resources::types::Filter::new().with_namespace("test".into()),
        operations,
    )
    .await
    .unwrap();
    assert_eq!(updated.len(), 1);

    assert_eq!(
        json!(updated.remove(0).data),
        json!({
            "foo": { },
            "bar": {
                "baz": 2,
                "qux": 1
            },
        })
    );
}

#[actix_rt::test]
async fn test_move_empty() {
    let initial_json = json!({
        "foo": { "qux": 1 },
        "bar": {
            "baz": 2,
        },
    });

    let (_db_ctx, _id, mut db_conn) = init_resource(initial_json).await;

    let operations = vec![inventory::ModuleResourceOperation::Move {
        from: "".into(),
        path: "/bar/qux".into(),
    }];

    match db::queries::module_resources::patch_module_resources(
        &mut db_conn,
        db::queries::module_resources::types::Filter::new().with_namespace("test".into()),
        operations,
    )
    .await
    {
        Err(db::queries::module_resources::types::JsonOperationError::JsonPatch { source }) => {
            assert_eq!(
                db::queries::module_resources::types::JsonPatchErrorCode::InvalidFromPath,
                source.error_code
            )
        }
        unexpected => panic!("Expected invalid_from_path error, got {unexpected:?}"),
    }
}

#[actix_rt::test]
async fn test_move_root() {
    let initial_json = json!({
        "foo": { "qux": 1 },
        "bar": {
            "baz": 2,
        },
    });

    let (_db_ctx, _id, mut db_conn) = init_resource(initial_json).await;

    let operations = vec![inventory::ModuleResourceOperation::Move {
        from: "/".into(),
        path: "/bar/qux".into(),
    }];

    match db::queries::module_resources::patch_module_resources(
        &mut db_conn,
        db::queries::module_resources::types::Filter::new().with_namespace("test".into()),
        operations,
    )
    .await
    {
        Err(db::queries::module_resources::types::JsonOperationError::JsonPatch { source }) => {
            assert_eq!(
                db::queries::module_resources::types::JsonPatchErrorCode::InvalidFromPath,
                source.error_code
            )
        }
        unexpected => panic!("Expected invalid_from_path error, got {unexpected:?}"),
    }
}

#[actix_rt::test]
async fn test_move_null_value() {
    let initial_json = json!({
        "foo": null,
        "bar": {
            "baz": 2,
        },
    });

    let (_db_ctx, _id, mut db_conn) = init_resource(initial_json).await;

    let operations = vec![inventory::ModuleResourceOperation::Move {
        from: "/foo".into(),
        path: "/bar/qux".into(),
    }];

    let mut updated = db::queries::module_resources::patch_module_resources(
        &mut db_conn,
        db::queries::module_resources::types::Filter::new().with_namespace("test".into()),
        operations,
    )
    .await
    .unwrap();

    assert_eq!(
        json!(updated.remove(0).data),
        json!({
            "bar": {
                "baz": 2,
                "qux": null,
            },
        })
    );
}

#[actix_rt::test]
async fn test_move_into_child() {
    let initial_json = json!({
        "foo": {
            "bar": 42
        },
    });

    let (_db_ctx, _id, mut db_conn) = init_resource(initial_json).await;

    let from_path: String = "/foo".into();
    let to_path: String = "/foo/bar".into();
    let operations = vec![inventory::ModuleResourceOperation::Move {
        from: from_path,
        path: to_path,
    }];

    match db::queries::module_resources::patch_module_resources(
        &mut db_conn,
        db::queries::module_resources::types::Filter::new().with_namespace("test".into()),
        operations,
    )
    .await
    {
        Err(db::queries::module_resources::types::JsonOperationError::JsonPatch { source }) => {
            assert_eq!(
                db::queries::module_resources::types::JsonPatchErrorCode::InvalidFromPath,
                source.error_code
            )
        }
        unexpected => panic!("Expected invalid_source_path error, got {unexpected:?}"),
    }
}

#[actix_rt::test]
async fn test_move_value_inside_array() {
    let initial_json = json!({ "foo": [ "all", "grass", "cows", "eat" ] });

    let (_db_ctx, _id, mut db_conn) = init_resource(initial_json).await;

    let operations = vec![inventory::ModuleResourceOperation::Move {
        from: "/foo/1".into(),
        path: "/foo/3".into(),
    }];

    let mut updated = db::queries::module_resources::patch_module_resources(
        &mut db_conn,
        db::queries::module_resources::types::Filter::new().with_namespace("test".into()),
        operations,
    )
    .await
    .unwrap();
    assert_eq!(updated.len(), 1);

    assert_eq!(
        json!(updated.remove(0).data),
        json!({ "foo": [ "all", "cows", "eat", "grass" ] })
    );
}

#[actix_rt::test]
async fn test_move_array_value() {
    let initial_json = json!({
        "foo": ["a", "b", "c"],
        "bar": {
            "baz": ["d"],
            "qux": 1,
        },
    });

    let (_db_ctx, _id, mut db_conn) = init_resource(initial_json).await;

    let operations = vec![inventory::ModuleResourceOperation::Move {
        from: "/foo/0".into(),
        path: "/bar/baz/1".into(),
    }];

    let mut updated = db::queries::module_resources::patch_module_resources(
        &mut db_conn,
        db::queries::module_resources::types::Filter::new().with_namespace("test".into()),
        operations,
    )
    .await
    .unwrap();
    assert_eq!(updated.len(), 1);

    assert_eq!(
        json!(updated.remove(0).data),
        json!({
            "foo": ["b", "c"],
            "bar": {
                "baz": ["d", "a"],
                "qux": 1,
            },
        })
    );
}

#[actix_rt::test]
async fn test_simple_copy() {
    let initial_json = json!({
        "foo": { "qux": 1 },
        "bar": {
            "baz": 2,
        },
    });

    let (_db_ctx, _id, mut db_conn) = init_resource(initial_json).await;

    let operations = vec![inventory::ModuleResourceOperation::Copy {
        from: "/foo/qux".into(),
        path: "/bar/qux".into(),
    }];

    let mut updated = db::queries::module_resources::patch_module_resources(
        &mut db_conn,
        db::queries::module_resources::types::Filter::new().with_namespace("test".into()),
        operations,
    )
    .await
    .unwrap();
    assert_eq!(updated.len(), 1);

    assert_eq!(
        json!(updated.remove(0).data),
        json!({
            "foo": { "qux": 1 },
            "bar": {
                "baz": 2,
                "qux": 1,
            },
        })
    );
}

#[actix_rt::test]
async fn test_simple_test() {
    let initial_json = json!({
        "foo": 1,
        "bar": 2,
    });

    let (_db_ctx, _id, mut db_conn) = init_resource(initial_json).await;

    let operations = vec![inventory::ModuleResourceOperation::Test {
        path: "/foo".into(),
        value: Value::Number(1.into()),
    }];

    let mut updated = db::queries::module_resources::patch_module_resources(
        &mut db_conn,
        db::queries::module_resources::types::Filter::new().with_namespace("test".into()),
        operations,
    )
    .await
    .unwrap();
    assert_eq!(updated.len(), 1);

    assert_eq!(
        json!(updated.remove(0).data),
        json!({
            "foo": 1,
            "bar": 2,
        })
    );
}

#[actix_rt::test]
async fn test_simple_test_error() {
    let initial_json = json!({
        "foo": 1,
        "bar": 2,
    });

    let (_db_ctx, _id, mut db_conn) = init_resource(initial_json).await;

    let operations = vec![inventory::ModuleResourceOperation::Test {
        path: "/foo".into(),
        value: Value::Number(99.into()),
    }];

    match db::queries::module_resources::patch_module_resources(
        &mut db_conn,
        db::queries::module_resources::types::Filter::new().with_namespace("test".into()),
        operations,
    )
    .await
    {
        Err(db::queries::module_resources::types::JsonOperationError::JsonPatch { source }) => {
            assert_eq!(
                db::queries::module_resources::types::JsonPatchErrorCode::ValueNotEqual,
                source.error_code
            )
        }
        unexpected => panic!("Expected failed compare error, got {unexpected:?}"),
    }
}

#[actix_rt::test]
async fn test_value_is_kept_on_error() {
    let initial_json = json!({
        "foo": 1,
        "bar": 2,
    });

    let (_db_ctx, id, mut db_conn) = init_resource(initial_json.clone()).await;

    let operations = vec![
        inventory::ModuleResourceOperation::Add {
            path: "/baz".into(),
            value: Value::Number(3.into()),
        },
        inventory::ModuleResourceOperation::Remove {
            path: "/bar".into(),
        },
        inventory::ModuleResourceOperation::Test {
            path: "/foo".into(),
            value: Value::Number(99.into()),
        },
    ];

    match db::queries::module_resources::patch_module_resources(
        &mut db_conn,
        db::queries::module_resources::types::Filter::new().with_namespace("test".into()),
        operations,
    )
    .await
    {
        Err(db::queries::module_resources::types::JsonOperationError::JsonPatch { source }) => {
            assert_eq!(
                db::queries::module_resources::types::JsonPatchErrorCode::ValueNotEqual,
                source.error_code
            )
        }
        unexpected => panic!("Expected failed compare error, got {unexpected:?}"),
    }

    let resource = db::queries::module_resources::get_module_resource(
        &mut db_conn,
        db::queries::module_resources::types::Filter::new().with_id(id),
    )
    .await
    .unwrap()
    .remove(0);

    // resource should not be changed
    assert_eq!(initial_json, resource.data);
}

#[actix_rt::test]
async fn test_object_value() {
    let initial_json = json!({
    "foo": {
        "baz": 1,
        "bar": 2,
    }});

    let (_db_ctx, _id, mut db_conn) = init_resource(initial_json).await;

    let operations = vec![inventory::ModuleResourceOperation::Test {
        path: "/foo".into(),
        value: json!({
            "baz": 1,
            "bar": 2,
        }),
    }];

    let mut updated = db::queries::module_resources::patch_module_resources(
        &mut db_conn,
        db::queries::module_resources::types::Filter::new().with_namespace("test".into()),
        operations,
    )
    .await
    .unwrap();
    assert_eq!(updated.len(), 1);

    assert_eq!(
        json!(updated.remove(0).data),
        json!({
            "foo": {
                "baz": 1,
                "bar": 2,
            }
        }
        )
    );
}

#[actix_rt::test]
async fn test_simple_replace() {
    let initial_json = json!({
        "foo": 1,
        "bar": 2,
    });

    let (_db_ctx, _id, mut db_conn) = init_resource(initial_json).await;

    let operations = vec![inventory::ModuleResourceOperation::Replace {
        path: "/bar".into(),
        value: Value::Number(99.into()),
    }];

    let mut updated = db::queries::module_resources::patch_module_resources(
        &mut db_conn,
        db::queries::module_resources::types::Filter::new().with_namespace("test".into()),
        operations,
    )
    .await
    .unwrap();
    assert_eq!(updated.len(), 1);

    assert_eq!(
        json!(updated.remove(0).data),
        json!({
            "foo": 1,
            "bar": 99,
        })
    );
}

#[actix_rt::test]
async fn test_replace_array_index() {
    let initial_json = json!({
        "foo": ["x", "b", "c"],
    });

    let (_db_ctx, _id, mut db_conn) = init_resource(initial_json).await;

    let operations = vec![inventory::ModuleResourceOperation::Replace {
        path: "/foo/0".into(),
        value: Value::String("a".into()),
    }];

    let mut updated = db::queries::module_resources::patch_module_resources(
        &mut db_conn,
        db::queries::module_resources::types::Filter::new().with_namespace("test".into()),
        operations,
    )
    .await
    .unwrap();
    assert_eq!(updated.len(), 1);

    assert_eq!(
        json!(updated.remove(0).data),
        json!({
            "foo": ["a", "b", "c"],
        })
    );
}

#[actix_rt::test]
async fn test_replace_invalid_array_index() {
    let initial_json = json!({
        "foo": ["a", "b", "c"],
    });

    let (_db_ctx, _id, mut db_conn) = init_resource(initial_json).await;

    let operations = vec![inventory::ModuleResourceOperation::Replace {
        path: "/foo/3".into(),
        value: Value::String("d".into()),
    }];

    match db::queries::module_resources::patch_module_resources(
        &mut db_conn,
        db::queries::module_resources::types::Filter::new().with_namespace("test".into()),
        operations,
    )
    .await
    {
        Err(db::queries::module_resources::types::JsonOperationError::JsonPatch { source }) => {
            assert_eq!(
                db::queries::module_resources::types::JsonPatchErrorCode::InvalidPath,
                source.error_code
            )
        }
        unexpected => panic!("Expected invalid path error, got {unexpected:?}"),
    }
}

#[actix_rt::test]
async fn test_replace_nonexistent_field() {
    let initial_json = json!({
        "foo": 1,
    });

    let (_db_ctx, _id, mut db_conn) = init_resource(initial_json).await;

    let operations = vec![inventory::ModuleResourceOperation::Replace {
        path: "/bar".into(),
        value: Value::String("2".into()),
    }];

    match db::queries::module_resources::patch_module_resources(
        &mut db_conn,
        db::queries::module_resources::types::Filter::new().with_namespace("test".into()),
        operations,
    )
    .await
    {
        Err(db::queries::module_resources::types::JsonOperationError::JsonPatch { source }) => {
            assert_eq!(
                db::queries::module_resources::types::JsonPatchErrorCode::InvalidPath,
                source.error_code
            )
        }
        unexpected => panic!("Expected invalid path error, got {unexpected:?}"),
    }
}

#[actix_rt::test]
async fn test_filter() {
    let db_ctx = opentalk_test_util::database::DatabaseContext::new().await;

    let mut db_conn = db_ctx.db.get_conn().await.unwrap();

    let user = db_ctx.create_test_user(0, vec![]).await.unwrap();

    let room = db_ctx
        .create_test_room(RoomId::from_u128(0), user.id, false)
        .await
        .unwrap();

    let tenant = db::queries::tenants::get_or_create_tenant_by_oidc_id(
        &mut db_conn,
        &db::tables::tenants::OidcTenantId::from("OpenTalkDefaultTenant"),
    )
    .await
    .unwrap();

    let new_resource = db::tables::module_resources::NewModuleResource {
        tenant_id: tenant.id,
        created_by: user.id,
        room_id: room.id,
        namespace: "test".into(),
        tag: Some("something".into()),
        data: json! {
            {
                "foo": "a",
                "bar": "b"
            }
        },
    };

    let module_resource =
        db::queries::module_resources::create_module_resource(&mut db_conn, new_resource)
            .await
            .unwrap();

    let resources = db::queries::module_resources::get_module_resource(
        &mut db_conn,
        db::queries::module_resources::types::Filter::new().with_namespace("not-test".into()),
    )
    .await
    .unwrap();

    assert!(resources.is_empty());

    let resources = db::queries::module_resources::get_module_resource(
        &mut db_conn,
        db::queries::module_resources::types::Filter::new().with_tag("nothing".into()),
    )
    .await
    .unwrap();

    assert!(resources.is_empty());

    let resources = db::queries::module_resources::get_module_resource(
        &mut db_conn,
        db::queries::module_resources::types::Filter::new().with_json(json! {{"baz": "c"}}),
    )
    .await
    .unwrap();

    assert!(resources.is_empty());

    let filter = db::queries::module_resources::types::Filter::new()
        .with_id(module_resource.id)
        .with_created_by(user.id)
        .with_namespace("test".into())
        .with_tag("something".into())
        .with_json(json! {
            {
                "foo": "a"
            }
        });

    let resources = db::queries::module_resources::get_module_resource(&mut db_conn, filter)
        .await
        .unwrap();

    assert!(resources.len() == 1);
    assert_eq!(module_resource, resources[0])
}
