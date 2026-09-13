// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use serde::Serialize;

#[derive(Default, Debug, Copy, Clone, PartialEq, Eq, Serialize)]
pub enum ShareType {
    User,
    Group,
    #[default]
    PublicLink,
    Email,
    FederatedCloudShare,
    Circle,
    TalkConversation,
}

impl From<ShareType> for u8 {
    fn from(value: ShareType) -> Self {
        match value {
            ShareType::User => 0,
            ShareType::Group => 1,
            ShareType::PublicLink => 3,
            ShareType::Email => 4,
            ShareType::FederatedCloudShare => 6,
            ShareType::Circle => 7,
            ShareType::TalkConversation => 10,
        }
    }
}
