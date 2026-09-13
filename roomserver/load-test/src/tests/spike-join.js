// SPDX-License-Identifier: EUPL-1.2
// SPDX-FileCopyrightText: OpenTalk Team <mail@opentalk.eu>
/**
 * Spike Join Test
 * Joins a single user that sends echo requests in a fixed interval.
 * Then joins many users in a short amount of time to create a spike in load.
 * The goal is to observe how much the echo RTT increases and how quickly it recovers.
 *
 * * Metrics:
 * - spike_join_echo_rtt: gauge of per-request round-trip times in milliseconds of
 *  the measuring user.
 */
import { sleep } from 'k6';
import exec from 'k6/execution';
import { Gauge } from 'k6/metrics';

import { ClientBuilder } from '../lib/client.js';
import { getEnv, getRequiredEnv } from '../lib/environment.js';
import { shared_tags } from '../lib/metrics.js';

// Test metadata
const TEST_NAME = 'spike_join';
const TEST_START_TIME = new Date();

// Test configuration
const BASE_URL = getRequiredEnv('BASE_URL');
const ROOM_ID = getEnv('ROOM_ID', '27c66df5-f6be-4d70-a167-abba2cf28a2a');

const SPIKE_ECHO_INTERVAL = getEnv('SPIKE_JOIN_ECHO_INTERVAL', 1); // seconds
const START_DELAY = getEnv('SPIKE_JOIN_START_DELAY', 30); // seconds
const SPIKE_RAMP_UP_DURATION = getEnv('SPIKE_JOIN_RAMP_UP_DURATION', 10); // seconds
const SPIKE_DURATION = getEnv('SPIKE_JOIN_DURATION', 30); // seconds
const SPIKE_RAMP_DOWN_DURATION = getEnv('SPIKE_JOIN_RAMP_DOWN_DURATION', 10); // seconds
const MAX_USERS = getEnv('SPIKE_JOIN_MAX_USERS', 800);

const SPIKE_END = START_DELAY + SPIKE_RAMP_UP_DURATION + SPIKE_DURATION;
const MEASURE_DURATION = START_DELAY + SPIKE_RAMP_UP_DURATION + SPIKE_DURATION + SPIKE_RAMP_DOWN_DURATION;

// Custom metrics
const ECHO_RTT = new Gauge('spike_join_echo_rtt');

export const options = {
  scenarios: {
    measure: {
      exec: 'measure',
      executor: 'constant-vus',
      vus: 1,
      duration: `${MEASURE_DURATION}s`,
    },
    spike: {
      exec: 'spike',
      executor: 'ramping-vus',
      startVUs: 0,
      // Start the spike after START_DELAY seconds to allow measuring baseline RTT first
      startTime: `${START_DELAY}s`,
      stages: [
        // Increase the number of VUs
        { duration: `${SPIKE_RAMP_UP_DURATION}s`, target: MAX_USERS },
        // Keep the number of VUs stable
        { duration: `${SPIKE_DURATION}s`, target: MAX_USERS },
        // Decrease the number of VUs
        { duration: `${SPIKE_RAMP_DOWN_DURATION}s`, target: 0 },
      ],
      gracefulRampDown: '0s',
    },
  },
};

export async function measure() {
  let transactionId = 0;
  const client = await new ClientBuilder().connect(BASE_URL, ROOM_ID);
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
      sleep(SPIKE_ECHO_INTERVAL);
    }
    console.info(`VU ${__VU} completed measuring. Disconnecting.`);
  } catch (err) {
    exec.test.abort(`VU ${__VU} encountered an error during measuring: ${err}`);
  } finally {
    client.disconnect();
  }
}

export async function spike() {
  // Do not join when the spike phase is over
  if (exec.instance.currentTestRunDuration >= SPIKE_END) {
    return;
  }

  let client;
  try {
    client = await new ClientBuilder().connect(BASE_URL, ROOM_ID);
    console.info(`VU ${__VU} connected`);

    // Wait until k6 stops the VU (and cancels the sleep timer)
    sleep(31557600);

    console.info(`VU ${__VU} completed test duration. Disconnecting.`);
  } catch (err) {
    exec.test.abort(`VU ${__VU} encountered an error during spike: ${err}`);
  } finally {
    if (client) {
      client.disconnect();
    }
  }
}
