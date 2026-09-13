// SPDX-License-Identifier: EUPL-1.2
// SPDX-FileCopyrightText: OpenTalk Team <mail@opentalk.eu>
/**
 * Fairness test
 * Spins up to 200 virtual users (VU) that join the same room, wait for the
 * ramp-up phase to complete, and then continuously send echo commands.
 * The goal is to detect whether the roomserver keeps RTTs comparable for
 * all users under sustained concurrent load.
 *
 * Metrics:
 * - echo_responses: Total successful echo replies per test window.
 * - echo_rtt: trend of per-request round-trip times in milliseconds.
 *
 * Both metrics are tagged with VU and iteration identifiers to allow comparing
 * results across the different VUs.
 */
import { sleep } from 'k6';
import exec from 'k6/execution';
import { Counter, Trend } from 'k6/metrics';

import { ClientBuilder } from '../lib/client.js';
import { getEnv, getRequiredEnv } from '../lib/environment.js';
import { shared_tags } from '../lib/metrics.js';

// Test metadata
const TEST_NAME = 'fairness';
const TEST_START_TIME = new Date();

// Test configuration
const BASE_URL = getRequiredEnv('BASE_URL');
const ROOM_ID = getEnv('ROOM_ID', '27c66df5-f6be-4d70-a167-abba2cf28a2a');

// Duration to reach number of VUs (default: 5 minutes)
const RAMP_UP_DURATION_SECONDS = getEnv('FAIRNESS_RAMP_UP_DURATION_SECONDS', 5 * 60);
// Duration of the fairness test (default: 10 minutes)
const TEST_DURATION_SECONDS = getEnv('FAIRNESS_TEST_DURATION_SECONDS', 10 * 60);
const USERS = getEnv('FAIRNESS_USERS', 200);

const ECHO_START_TIME = TEST_START_TIME.getTime() + RAMP_UP_DURATION_SECONDS;

// Custom metrics
const ECHO_RESPONSES_COUNTER = new Counter('fairness_echo_responses');
const ECHO_RTT = new Trend('fairness_echo_rtt', true);

export const options = {
  scenarios: {
    fairness_test: {
      executor: 'ramping-vus',
      startVUs: 0,
      stages: [
        // Ramp up VUs, make sure everyone joined
        { duration: `${RAMP_UP_DURATION_SECONDS}s`, target: USERS },
        // Keep the number VUs constant and test for fairness
        { duration: `${TEST_DURATION_SECONDS}s`, target: USERS },
      ],
    },
  },
};

export default async function () {
  let transactionId = 0;
  const client = await new ClientBuilder().connect(BASE_URL, ROOM_ID);

  // Wait until all clients have been connected before sending echo requests.
  // Sending echo requests during ramp-up would skew the results because some clients
  // would start sending requests much earlier than others.
  const sleepDuration = Math.max(0, (ECHO_START_TIME - Date.now()) / 1000); // sleep uses seconds
  console.info(`VU ${__VU} connected. Waiting to start echo commands. Sleeping for ${sleepDuration}s`);
  sleep(sleepDuration);

  console.info(`VU ${__VU} starting echo commands`);

  const testEndMS = (RAMP_UP_DURATION_SECONDS + TEST_DURATION_SECONDS) * 1000;
  try {
    // Run until the test duration has elapsed
    while (exec.instance.currentTestRunDuration < testEndMS) {
      // Using exec.instance.currentTestRunDuration to measure time instead of Date.now()
      // because Date.now() only supports millisecond precision which is not enough for RTT
      // measurement. exec.instance.currentTestRunDuration reports time in milliseconds but
      // as a floating point number with 17 digits of precision.
      const startTime = exec.instance.currentTestRunDuration;
      await client.sendEcho(transactionId++);

      // Record metrics
      const rtt = exec.instance.currentTestRunDuration - startTime;

      // Update metrics
      const tags = shared_tags(TEST_NAME, TEST_START_TIME);
      ECHO_RESPONSES_COUNTER.add(1, tags);
      ECHO_RTT.add(rtt, tags);
    }
    console.info(`VU ${__VU} completed test duration. Disconnecting.`);
  } catch (err) {
    console.error(`VU ${__VU} encountered an error: ${err.message}. Disconnecting.`);
  } finally {
    client.disconnect();
  }
}
