// SPDX-License-Identifier: EUPL-1.2
// SPDX-FileCopyrightText: OpenTalk Team <mail@opentalk.eu>

use std::collections::{HashMap, HashSet};

use opentalk_roomserver_types::signaling::websocket::SignalingSocket;
use opentalk_types_common::{rooms::RoomId, users::UserId};

use crate::RoomTaskHandle;

#[derive(Debug)]
pub(super) struct RoomMap<Socket: SignalingSocket + 'static> {
    inner: HashMap<RoomId, RoomTaskHandle<Socket>>,
    rooms_by_creator: HashMap<UserId, HashSet<RoomId>>,
}

impl<S: SignalingSocket + 'static> Default for RoomMap<S> {
    fn default() -> Self {
        Self {
            inner: Default::default(),
            rooms_by_creator: Default::default(),
        }
    }
}

impl<S: SignalingSocket + 'static> RoomMap<S> {
    pub fn map(&self) -> &HashMap<RoomId, RoomTaskHandle<S>> {
        &self.inner
    }

    pub fn insert(&mut self, room_id: RoomId, created_by: UserId, handle: RoomTaskHandle<S>) {
        self.inner.insert(room_id, handle);
        self.rooms_by_creator
            .entry(created_by)
            .or_default()
            .insert(room_id);
    }

    pub fn remove(&mut self, room_id: RoomId) {
        self.inner.remove(&room_id);
        self.rooms_by_creator.retain(|_, room_ids| {
            room_ids.remove(&room_id);
            !room_ids.is_empty()
        });
    }

    pub fn handles_by_creator(&self, creator: UserId) -> Vec<(RoomId, RoomTaskHandle<S>)> {
        let Some(room_ids) = self.rooms_by_creator.get(&creator) else {
            return Vec::new();
        };

        self.inner
            .iter()
            .filter_map(|(id, handle)| {
                if room_ids.contains(id) {
                    Some((*id, handle.clone()))
                } else {
                    None
                }
            })
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use opentalk_types_common::{rooms::RoomId, users::UserId};
    use pretty_assertions::assert_eq;

    use crate::{RoomTaskHandle, mocking::socket::MockSocket, room_map::RoomMap};

    #[test]
    fn inserting_room_maps_creator() {
        let mut map = RoomMap::<MockSocket>::default();
        let created_by = UserId::from_u128(0x1);
        let room_id = RoomId::from_u128(0x1);

        map.insert(room_id, created_by, RoomTaskHandle::<MockSocket>::default());

        let room_ids = map
            .handles_by_creator(created_by)
            .iter()
            .map(|(id, _)| *id)
            .collect::<Vec<_>>();
        assert_eq!(room_ids, vec![room_id]);
    }

    #[test]
    fn removing_last_room_removes_creator() {
        let mut map = RoomMap::<MockSocket>::default();
        let created_by = UserId::from_u128(0x1);
        let handle = RoomTaskHandle::<MockSocket>::default();

        // Add two rooms for the same creator
        let room_1 = RoomId::from_u128(0x1);
        map.insert(room_1, created_by, handle.clone());
        let room_2 = RoomId::from_u128(0x2);
        map.insert(room_2, created_by, handle);

        // Removing one room removes it from the creator's set but keeps the creator
        map.remove(room_1);
        let room_ids = map
            .handles_by_creator(created_by)
            .iter()
            .map(|(id, _)| *id)
            .collect::<Vec<_>>();
        assert_eq!(room_ids, vec![room_2]);

        // Removing the last room removes the creator from the map
        map.remove(room_2);
        assert!(map.rooms_by_creator.is_empty());
    }
}
