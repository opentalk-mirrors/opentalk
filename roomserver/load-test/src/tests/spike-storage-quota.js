// SPDX-License-Identifier: EUPL-1.2
// SPDX-FileCopyrightText: OpenTalk Team <mail@opentalk.eu>
/**
 * Spike Storage Quota Test
 * Creates multiple rooms by ramping up virtual users, each joining their own room.
 * While the rooms are active, a separate scenario sends HTTP POST requests to update
 * a user's storage quota. All VUs share the same user ID, so the corresponding `StorageQuotaChanged`
 * core event is sent to each room.
 *  A single measuring user joins its own room and continuously sends echo requests to track the
 * roomserver's responsiveness during the spike.
 * The goal is to observe how storage quota updates affect echo round-trip times.
 *
 * Metrics:
 * - spike_storage_quota_echo_rtt: gauge of per-request round-trip times in milliseconds
 *   of the measuring user.
 */
import { sleep } from 'k6';
import exec from 'k6/execution';
import http from 'k6/http';
import { Gauge } from 'k6/metrics';

import { generateJWT } from '../lib/auth.js';
import { ClientBuilder } from '../lib/client.js';
import { getEnv, getRequiredEnv } from '../lib/environment.js';
import { shared_tags } from '../lib/metrics.js';

// Test metadata
const TEST_NAME = 'spike_storage_quota';
const TEST_START_TIME = new Date();

// Test configuration
const BASE_URL = getRequiredEnv('BASE_URL');
const ECHO_INTERVAL = getEnv('SPIKE_STORAGE_QUOTA_ECHO_INTERVAL', 1); // seconds

const START_DELAY = getEnv('SPIKE_STORAGE_QUOTA_START_DELAY', 20); // seconds
const RAMP_UP_DURATION = getEnv('SPIKE_STORAGE_QUOTA_RAMP_UP_DURATION', 10); // seconds
const SPIKE_DURATION = getEnv('SPIKE_STORAGE_QUOTA_SPIKE_DURATION', 50); // seconds
const RAMP_DOWN_DURATION = getEnv('SPIKE_STORAGE_QUOTA_RAMP_DOWN_DURATION', 10); // seconds
const MAX_USERS = getEnv('SPIKE_STORAGE_QUOTA_MAX_USERS', 10);
const CONCURRENT_QUOTA_UPDATES = getEnv('SPIKE_STORAGE_QUOTA_CONCURRENT_QUOTA_UPDATES', 10);

const MEASURE_DURATION = START_DELAY + RAMP_UP_DURATION + SPIKE_DURATION + RAMP_DOWN_DURATION;

// Custom metrics
const ECHO_RTT = new Gauge('spike_storage_quota_echo_rtt');

export const options = {
  scenarios: {
    rooms: {
      exec: 'createAndHoldRoom',
      executor: 'ramping-vus',
      startVUs: 0,
      stages: [
        // Ramp up to MAX_USERS VUs
        { duration: `${RAMP_UP_DURATION}s`, target: MAX_USERS },
        // Keep at MAX_USERS VUs
        { duration: `${SPIKE_DURATION}s`, target: MAX_USERS },
        // Ramp down to 0 VUs
        { duration: `${RAMP_DOWN_DURATION}s`, target: 0 },
      ],
    },
    request: {
      exec: 'post_storage_quota',
      executor: 'constant-vus',
      vus: CONCURRENT_QUOTA_UPDATES,
      // Start sending requests after the start delay to allow rooms to be created first
      startTime: `${START_DELAY}s`,
      duration: `${SPIKE_DURATION}s`,
    },
    measure: {
      exec: 'measure',
      executor: 'constant-vus',
      vus: 1,
      duration: `${MEASURE_DURATION}s`,
    },
  },
};

export async function createAndHoldRoom() {
  let client;
  try {
    const roomId = crypto.randomUUID();
    client = await new ClientBuilder().connect(BASE_URL, roomId);
    console.info(`VU ${__VU} connected to room ${roomId}`);

    // Wait until k6 stops the VU (and cancels the sleep timer)
    sleep(31557600);

    console.info(`VU ${__VU} completed test duration. Disconnecting.`);
  } catch (err) {
    exec.test.abort(`VU ${__VU} encountered an error: ${err}`);
  } finally {
    client?.disconnect();
  }
}

export async function post_storage_quota() {
  const userId = '00000000-0000-0000-0000-0000000a11ce';
  const body = JSON.stringify({
    total: 2,
    used: 1,
  });
  const jwt = generateJWT();
  const params = {
    headers: {
      'Content-Type': 'application/json',
      Authorization: `Bearer ${jwt}`,
    },
  };
  const response = http.post(`${BASE_URL}/v1/user/${userId}/storage_quota`, body, params);
  if (response.status < 200 || response.status > 299) {
    console.error(
      `VU ${__VU} received unexpected status ${response.status} for storage quota request: ${response.body}`
    );
  }
}

export async function measure() {
  let transactionId = 0;
  const roomId = crypto.randomUUID();
  // Use a different user ID for the measuring client to ensure it does not receive the StorageQuotaChanged event
  const userId = '00000000-0000-0000-0000-000000ea500e';
  const client = await new ClientBuilder().connect(BASE_URL, roomId, userId);
  console.info(`VU ${__VU} connected for measuring`);

  try {
    const measureDurationMs = MEASURE_DURATION * 1000;
    while (exec.instance.currentTestRunDuration < measureDurationMs) {
      const startTime = exec.instance.currentTestRunDuration;
      await client.sendEcho(transactionId++);

      // Record metrics
      const rtt = exec.instance.currentTestRunDuration - startTime;
      ECHO_RTT.add(rtt, shared_tags(TEST_NAME, TEST_START_TIME));

      // Wait between echo requests
      sleep(ECHO_INTERVAL);
    }
    console.info(`VU ${__VU} completed measuring. Disconnecting.`);
  } catch (err) {
    exec.test.abort(`VU ${__VU} encountered an error during measuring: ${err}`);
  } finally {
    client.disconnect();
  }
}
