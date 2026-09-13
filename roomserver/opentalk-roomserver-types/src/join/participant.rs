// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use std::collections::BTreeMap;

use opentalk_types_common::modules::ModuleId;
use opentalk_types_signaling::{ParticipantId, SignalingModulePeerFrontendData};
use serde::{Deserialize, Serialize};

use crate::{join::connection_info::ConnectionInfo, shared_json::SharedJson};

/// Status information about a participant
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Participant {
    /// The id of the participant
    pub id: ParticipantId,

    /// The connections of the participant
    pub connections: Vec<ConnectionInfo>,

    /// Module data for the participant
    pub module_data: BTreeMap<ModuleId, SharedJson>,
}

impl Participant {
    /// Gets the inner module data of a Participant
    pub fn get_module<T: SignalingModulePeerFrontendData>(
        &self,
    ) -> Result<Option<T>, serde_json::Error> {
        let Some(namespace) = T::NAMESPACE else {
            return Ok(None);
        };

        self.module_data
            .get(&namespace)
            .map(|m| serde_json::from_value(m.clone_inner()))
            .transpose()
    }
}

#[cfg(test)]
mod test {
    use opentalk_types_signaling::ParticipantId;

    use crate::{
        breakout::{BREAKOUT_MODULE_ID, module_data::BreakoutPeerModuleData},
        connection_id::ConnectionId,
        device_id::DeviceId,
        join::{connection_info::ConnectionInfo, participant::Participant},
    };

    #[test]
    fn serialize() {
        let mut participant = Participant {
            id: ParticipantId::from_u128(0x0001),
            connections: vec![ConnectionInfo {
                connection_id: ConnectionId::from_u128(0x0101),
                device_id: DeviceId::from_u128(0x0201),
            }],
            module_data: Default::default(),
        };

        participant.module_data.insert(
            BREAKOUT_MODULE_ID,
            serde_json::to_value(&BreakoutPeerModuleData {
                room: crate::room_kind::RoomKind::Main,
            })
            .unwrap()
            .into(),
        );

        // insta doesn't serialize correctly
        let raw = serde_json::to_string_pretty(&participant).unwrap();
        insta::assert_snapshot!(raw, @r#"
        {
          "id": "00000000-0000-0000-0000-000000000001",
          "connections": [
            {
              "connection_id": "00000000-0000-0000-0000-000000000101",
              "device_id": "00000000-0000-0000-0000-000000000201"
            }
          ],
          "module_data": {
            "breakout": {
              "room": {
                "kind": "main"
              }
            }
          }
        }
        "#);
    }
}
