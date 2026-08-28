// SPDX-License-Identifier: EUPL-1.2
// SPDX-FileCopyrightText: OpenTalk Team <mail@opentalk.eu>

/**
 * Create a default room parameter configuration
 * @param {object} rateLimit - The rate limit configuration
 * @param {string} userId - The user who creates the room
 * @return {object} The room parameters
 */
export function createRoomParameter(rateLimit, userId) {
  return {
    created_by: {
      id: userId || '00000000-0000-0000-0000-0000000a11ce',
      email: 'alice@example.com',
      title: 'M.Sc.',
      firstname: 'Alice',
      lastname: 'Aal',
      display_name: 'Alice the angry',
      avatar_url: 'https://example.com/avatar-of-alice',
      timezone: 'Europe/Berlin',
    },
    password: null,
    guest_access: true,
    waiting_room: 'disabled',
    call_in: null,
    event: null,
    tariff: {
      id: '2da2b825-6db9-4dc4-b9e6-b4fd64e66a16',
      name: 'Starter tariff',
      quotas: {},
      used_quota: {},
      disabled_features: [],
    },
    streaming_targets: [],
    show_meeting_details: true,
    e2e_encryption: false,
    module_settings: {
      livekit: {
        api_key: 'devkey',
        api_secret: 'secret',
        public_url: 'http://localhost:7880',
        service_url: 'http://localhost:7880',
      },
      automod: {},
      chat: {},
      e2ee: {},
      echo: {},
      meeting_report: {},
      moderation: {},
      polls: {},
      raise_hands: {},
      shared_folder: {},
      subroom_audio: {},
      timer: {},
      meeting_notes: {},
    },
    asset_storage: {
      type: 'controller',
      url: 'http://localhost:8000',
      secret: 'secret2',
    },
    preferred_language: 'de',
    fallback_language: 'de',
    ws_rate_limit: rateLimit,
    allowed_origins: ['*'],
    room_idle_timeout: 60 * 1000, // 1 minute
  };
}

/**
 * Create client parameters for a virtual user
 * @param vu - Virtual user number
 * @param iter - Iteration number
 * @param role - User role (default: "moderator")
 */
export function createClientParameter(vu, iter, role = 'moderator') {
  const userId = `00000000-0000-0000-0000-${String(vu).padStart(6, '0')}${String(iter).padStart(6, '0')}`;

  return {
    device_secret: `v3rys3cr3tD3v1ce5tr1ng-user-${vu}-${iter}`,
    kind: 'registered',
    role: role,
    profile: {
      id: userId,
      email: `user${vu}-${iter}@example.com`,
      title: 'M.Sc.',
      firstname: 'User',
      lastname: `${vu}`,
      display_name: `VU ${vu} Iteration ${iter}`,
      avatar_url: `https://example.com/avatar-${vu}`,
      timezone: 'Europe/Berlin',
    },
  };
}
