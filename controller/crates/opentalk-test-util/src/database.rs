// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use std::sync::Arc;

use diesel_async::{AsyncConnection, AsyncPgConnection, RunQueryDsl};
use opentalk_database::Db;
use opentalk_db_storage::migrations::migrate_from_url;
use opentalk_inventory::{InventoryProvider, NewRoom, NewUser, Room, User};
use opentalk_inventory_database::DatabaseConnectionPool;
use opentalk_types_common::{
    rooms::{GuestAccess, RoomId},
    tariffs::TariffStatus,
    tenants::TenantId,
    users::{GroupId, GroupName, UserId, UserTitle},
};
use snafu::{ResultExt, Whatever};
use testcontainers_modules::{
    postgres::Postgres,
    testcontainers::{ContainerAsync, ImageExt as _, runners::AsyncRunner as _},
};

/// User configured on the postgres testcontainer.
const POSTGRES_USER: &str = "postgres";
/// Password configured on the postgres testcontainer.
const POSTGRES_PASSWORD: &str = "postgres";
/// Image tag for the postgres testcontainer.
const POSTGRES_TAG: &str = "18-alpine";
/// Name of the database created for the test inside the dedicated container.
const TEST_DB_NAME: &str = "opentalk_test";
const POSTGRES_PORT: u16 = 5432;

/// Contains the [`Db`] as well as information about the test database
pub struct DatabaseContext {
    pub base_url: String,
    pub db_name: String,
    pub db: Arc<Db>,
    pub inventory_provider: Arc<DatabaseConnectionPool>,
    /// The postgres testcontainer backing this context.
    ///
    /// Dropping the [`DatabaseContext`] stops and removes the container (and with it the test database).
    _container: ContainerAsync<Postgres>,
}

impl DatabaseContext {
    /// Create a new [`DatabaseContext`]
    ///
    /// Starts a dedicated postgres [testcontainer] for this context and creates a migrated database inside it. Because
    /// every context gets its own container, tests are isolated from each other and can run in parallel. Running the
    /// tests requires a working docker (or compatible) environment.
    ///
    /// The container is owned by the returned [`DatabaseContext`] and is stopped and removed when it is dropped, so
    /// callers must keep the context alive for as long as they use its [`Db`] or any connection obtained from it.
    ///
    /// [testcontainer]: https://testcontainers.com/
    pub async fn new() -> Self {
        let container = Postgres::default()
            .with_tag(POSTGRES_TAG)
            .start()
            .await
            .expect("Failed to start postgres testcontainer");

        let host = container
            .get_host()
            .await
            .expect("Failed to get testcontainer host");
        let port = container
            .get_host_port_ipv4(POSTGRES_PORT)
            .await
            .expect("Failed to get testcontainer port");

        let base_url = format!("postgres://{POSTGRES_USER}:{POSTGRES_PASSWORD}@{host}:{port}");
        let db_name = TEST_DB_NAME.to_owned();

        let postgres_url = format!("{base_url}/postgres");
        let mut conn = AsyncPgConnection::establish(&postgres_url)
            .await
            .expect("Cannot connect to postgres database.");

        // Create the test database. The container is freshly started, so it cannot exist yet.
        diesel::sql_query(format!("CREATE DATABASE {db_name}"))
            .execute(&mut conn)
            .await
            .unwrap_or_else(|_| panic!("Could not create database {db_name}"));

        let db_url = format!("{base_url}/{db_name}");

        migrate_from_url(&db_url)
            .await
            .expect("Unable to migrate database");

        let db_conn = Arc::new(Db::connect_url(&db_url, 5).expect("Failed to connect to database"));

        let inventory_provider = Arc::new(DatabaseConnectionPool::new(db_conn.clone()));

        Self {
            base_url,
            db_name,
            db: db_conn,
            inventory_provider,
            _container: container,
        }
    }

    pub async fn create_test_user(&self, n: u32, groups: Vec<String>) -> Result<User, Whatever> {
        let mut connection = self
            .inventory_provider
            .get_inventory()
            .await
            .whatever_context("db connect failed")?;

        let tenant = connection
            .get_or_create_tenant_by_oidc_id(&"OpenTalkDefaultTenant".into())
            .await
            .whatever_context("get/create tenant failed")?;
        let tariff = connection
            .get_tariff_by_name("OpenTalkDefaultTariff")
            .await
            .unwrap();

        let user = connection
            .create_user(NewUser {
                oidc_sub: format!("oidc_sub{n}"),
                email: format!("opentalk_test_user{n}@example.org"),
                title: UserTitle::new(),
                firstname: "test".into(),
                lastname: "tester".into(),
                avatar_url: Some("https://example.com/avatar/abcdef".into()),
                display_name: "test tester".parse().expect("valid display name"),
                language: Some("en".parse().expect("valid language")),
                phone: None,
                tenant_id: tenant.id,
                tariff_id: tariff.id,
                tariff_status: TariffStatus::Default,
                timezone: None,
            })
            .await
            .whatever_context("create user failed")?;

        let groups: Vec<(TenantId, GroupName)> = groups
            .into_iter()
            .map(|name| (tenant.id, GroupName::from(name)))
            .collect();
        let groups = connection
            .get_or_create_groups_by_name(&groups)
            .await
            .whatever_context("create group failed")?
            .into_iter()
            .map(|g| g.id)
            .collect::<Vec<GroupId>>();
        connection
            .add_user_to_groups(user.id, &groups)
            .await
            .whatever_context("add user to group failed")?;

        Ok(user)
    }

    pub async fn create_test_room(
        &self,
        _room_id: RoomId,
        created_by: UserId,
        waiting_room: bool,
    ) -> Result<Room, Whatever> {
        let mut inventory = self
            .inventory_provider
            .get_inventory()
            .await
            .whatever_context("db connect failed")?;

        let tenant = inventory
            .get_or_create_tenant_by_oidc_id(&"OpenTalkDefaultTenant".into())
            .await
            .whatever_context("get or create tenant failed")?;

        let new_room = NewRoom {
            created_by,
            alias: None,
            password: None,
            waiting_room,
            guest_access: GuestAccess::default(),
            e2e_encryption: false,
            tenant_id: tenant.id,
        };

        let room = inventory
            .create_room(new_room)
            .await
            .whatever_context("creating room failed")?;

        Ok(room)
    }
}
