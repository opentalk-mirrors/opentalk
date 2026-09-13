// SPDX-License-Identifier: EUPL-1.2
// SPDX-FileCopyrightText: OpenTalk Team <mail@opentalk.eu>
/**
 * Break Join Test
 * Joins users in a constant rate. The users do nothing other than holding the connection open.
 */
import exec from 'k6/execution';

import { ClientBuilder } from '../lib/client.js';
import { getEnv, getRequiredEnv } from '../lib/environment.js';

// Room configuration
const BASE_URL = getRequiredEnv('BASE_URL');
const ROOM_ID = getEnv('ROOM_ID', '27c66df5-f6be-4d70-a167-abba2cf28a2a');
// Join configuration
const JOIN_RATE = getEnv('BREAK_JOIN_JOIN_RATE', 1); // users per JOIN_TIME_UNIT
const JOIN_TIME_UNIT = getEnv('BREAK_JOIN_JOIN_TIME_UNIT', 3); // seconds
const MAX_USERS = getEnv('BREAK_JOIN_MAX_USERS', 2000); // maximum number of users
// Test duration
const DURATION_SECONDS = getEnv('BREAK_JOIN_DURATION_SECONDS', 3 * 60 * 60); // 3 hours

export const options = {
  scenarios: {
    break_join: {
      executor: 'constant-arrival-rate',
      duration: `${DURATION_SECONDS}s`,
      rate: JOIN_RATE,
      timeUnit: `${JOIN_TIME_UNIT}s`,
      preAllocatedVUs: MAX_USERS,
    },
  },
};

export default async function () {
  let client;
  try {
    client = await new ClientBuilder().connect(BASE_URL, ROOM_ID);
  } catch (err) {
    exec.test.abort(`Failed to connect: ${err}`);
    return;
  }

  // Abort the test if a client gets disconnected
  client.ws.addEventListener('close', () => {
    exec.test.abort('WebSocket connection closed unexpectedly');
  });

  // Keep the connection open until k6 stops the VU.
  // Using a promise instead of sleep because sleep suspends the whole VU leading
  // to it not processing incoming messages and getting congested.
  // We don't run into connection timeouts because each new joined VU generates a
  // participant_connected event for all other connected VUs.
  await new Promise((resolve) => setTimeout(resolve, 31557600 * 1000));

  console.info(`VU ${__VU} completed test duration. Disconnecting.`);
  client.disconnect();
}
