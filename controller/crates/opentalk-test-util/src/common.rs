// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use std::collections::BTreeMap;

use opentalk_controller_api_authorization::authorization::Authorizer;
use opentalk_controller_api_authorization_database::OpenTalkAuthorizerBackend;
use opentalk_controller_settings::test_util::settings_provider_from_example_raw_settings;
use opentalk_inventory::User;
use opentalk_types_common::{rooms::RoomId, users::DisplayName};
use opentalk_types_signaling::ParticipantId;
use snafu::{ResultExt, Whatever};
use tokio::sync::broadcast::Sender;

use crate::database::DatabaseContext;

#[derive(Debug)]
pub struct TestUser {
    pub n: u32,
    pub participant_id: ParticipantId,
    pub name: &'static str,
}

impl TestUser {
    pub fn display_name(&self) -> DisplayName {
        DisplayName::from_str_lossy(self.name)
    }
}

pub const ROOM_ID: RoomId = RoomId::from_u128(2000);

pub const USER_1: TestUser = TestUser {
    n: 1,
    participant_id: ParticipantId::from_u128(1),
    name: "user1",
};

pub const USER_2: TestUser = TestUser {
    n: 2,
    participant_id: ParticipantId::from_u128(2),
    name: "user2",
};

pub const USERS: [TestUser; 2] = [USER_1, USER_2];

/// The [`TestContext`] provides access to redis & postgres for tests
pub struct TestContext {
    pub db_ctx: DatabaseContext,
    pub authorizer: Authorizer,
    pub shutdown: Sender<()>,
}

impl TestContext {
    /// Creates a new [`TestContext`]
    pub async fn new() -> Self {
        let _ = setup_logging();

        let db_ctx = DatabaseContext::new().await;

        let (shutdown, _) = tokio::sync::broadcast::channel(10);

        // Tests don't exercise tariff-gated module feature checks, so an empty
        // module feature map is sufficient for the database-backed authorizer.
        let authorizer = Authorizer::new(OpenTalkAuthorizerBackend::new(
            db_ctx.inventory_provider.clone(),
            settings_provider_from_example_raw_settings(),
            BTreeMap::new(),
        ));

        TestContext {
            db_ctx,
            authorizer,
            shutdown,
        }
    }

    pub async fn default() -> Self {
        Self::new().await
    }
}

pub fn setup_logging() -> Result<(), Whatever> {
    fern::Dispatch::new()
        .format(|out, message, record| {
            out.finish(format_args!(
                "[{}][{}] {}",
                record.target(),
                record.level(),
                message
            ))
        })
        .level(log::LevelFilter::Info)
        .chain(std::io::stdout())
        .apply()
        .whatever_context("Failed to setup logging utility")
}

/// Creates a new tuple with two users
pub async fn setup_users(test_ctx: &TestContext) -> (User, User) {
    let waiting_room = false;

    let user1 = test_ctx
        .db_ctx
        .create_test_user(USER_1.n, vec![])
        .await
        .unwrap();
    let user2 = test_ctx
        .db_ctx
        .create_test_user(USER_2.n, vec![])
        .await
        .unwrap();

    let _room = test_ctx
        .db_ctx
        .create_test_room(ROOM_ID, user1.id, waiting_room)
        .await
        .unwrap();

    (user1, user2)
}
