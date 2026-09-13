// SPDX-License-Identifier: EUPL-1.2
// SPDX-FileCopyrightText: OpenTalk Team <mail@opentalk.eu>
/**
 * Break Chat Test
 * Joins VUs in a constant rate. The VUs send chat messages at a fixed interval, with some jitter.
 * Joins a single VU that sends chat messages and measures the round-trip time and success rate of the messages.
 *
 * Metrics:
 * - break_chat_message_rtt: trend of per-message round-trip times in milliseconds.
 * - break_chat_message_success: rate of success responses.
 *
 * Notes:
 * - Chat rate limiting should be disabled for this test.
 * - Because this test is intended to run over a long period of time, prometheus will allocate a lot of memory to
 *   store the metrics. It is recommended to either decrease the push interval by setting the
 *   `K6_PROMETHEUS_RW_PUSH_INTERVAL` environment variable or to run prometheus on a separate machine with sufficient
 *   memory.
 */
import { SharedArray } from 'k6/data';
import exec from 'k6/execution';
import { Rate, Trend } from 'k6/metrics';

import { createGlobalScope } from '../lib/chat-scope.js';
import { ClientBuilder } from '../lib/client.js';
import { getEnv, getRequiredEnv } from '../lib/environment.js';
import { test_name_tag } from '../lib/metrics.js';

// Room configuration
const BASE_URL = getRequiredEnv('BASE_URL');
const ROOM_ID = getEnv('ROOM_ID', '27c66df5-f6be-4d70-a167-abba2cf28a2a');
// Join configuration
const JOIN_RATE = getEnv('BREAK_CHAT_JOIN_RATE', 1); // users per JOIN_TIME_UNIT
const JOIN_TIME_UNIT = getEnv('BREAK_CHAT_JOIN_TIME_UNIT', 3); // seconds
const MAX_USERS = getEnv('BREAK_CHAT_MAX_USERS', 2000); // maximum number of users
// Message sending configuration
const MESSAGE_INTERVAL = getEnv('BREAK_CHAT_MESSAGE_INTERVAL', 1); // seconds
const MESSAGE_JITTER = getEnv('BREAK_CHAT_MESSAGE_JITTER', 0.5); // seconds
const MESSAGE_CONTENT = getEnv('BREAK_CHAT_MESSAGE_CONTENT', 'generating load');
// Test duration
const DURATION_SECONDS = getEnv('BREAK_CHAT_DURATION_SECONDS', 3 * 60 * 60); // 3 hours
const DURATION_MS = DURATION_SECONDS * 1000;

// Custom metrics
const SUCCESS_RATE = new Rate('break_chat_message_success');
const MESSAGE_RTT = new Trend('break_chat_message_rtt', true);

export const options = {
  scenarios: {
    stress: {
      exec: 'stress',
      executor: 'constant-arrival-rate',
      duration: `${DURATION_SECONDS}s`,
      rate: JOIN_RATE,
      timeUnit: `${JOIN_TIME_UNIT}s`,
      preAllocatedVUs: MAX_USERS,
    },
    measure: {
      exec: 'measure',
      executor: 'constant-vus',
      vus: 1,
      duration: `${DURATION_SECONDS}s`,
    },
  },
};

// Initialization code is shared across VUs
// Allocating resources here, so they can be shared
const shared = new SharedArray('shared_data', function () {
  const data = Object.freeze({
    scope: createGlobalScope(),
    message: MESSAGE_CONTENT,
    tags: test_name_tag('break_chat', new Date()),
    command: JSON.stringify({
      namespace: 'chat',
      payload: {
        action: 'send_message',
        content: MESSAGE_CONTENT,
        ...createGlobalScope(),
      },
    }),
  });
  return [data];
});

/**
 * Sends chat messages at a fixed interval with some jitter.
 */
export async function stress() {
  const client = await connect();
  const duration = DURATION_SECONDS * 1000;
  try {
    while (exec.instance.currentTestRunDuration < duration) {
      client.ws.send(shared[0].command);
      const delay = randomFloatBetween(MESSAGE_INTERVAL - MESSAGE_JITTER, MESSAGE_INTERVAL + MESSAGE_JITTER);
      // Using a promise instead of sleep because sleep suspends the whole VU,
      // leading to it not processing incoming messages and getting congested.
      await new Promise((resolve) => setTimeout(resolve, delay * 1000));
    }
  } catch (err) {
    console.error('Encountered an error: ', err);
    exec.test.abort(err);
  } finally {
    client.disconnect();
  }
}

/**
 * Sends chat messages every second and measures the round-trip time and success rate.
 */
export async function measure() {
  const client = await connect();

  let transactionId = 0;
  const duration = DURATION_SECONDS * 1000;
  try {
    while (exec.instance.currentTestRunDuration < duration) {
      const startTime = exec.instance.currentTestRunDuration;
      const response = await client.sendChatMessage(shared[0].message, shared[0].scope, transactionId++);

      // Record metrics
      const rtt = exec.instance.currentTestRunDuration - startTime;
      const isSuccess = response?.payload?.message === 'message_sent';

      // Update metrics
      MESSAGE_RTT.add(rtt, shared[0].tags);
      SUCCESS_RATE.add(isSuccess, shared[0].tags);

      // Wait 1 second before sending the next message
      await new Promise((resolve) => setTimeout(resolve, 1000));
    }
  } catch (err) {
    console.error('Encountered an error: ', err);
    exec.test.abort(err);
  } finally {
    client.disconnect();
  }
}

/**
 * @typedef {import('../lib/client.js').Client} Client
 */

/**
 * Connects to the WebSocket server and returns the client.
 * Sets up an event listener to abort the test if the connection is closed.
 * @returns {Promise<Client>} The connected WebSocket client
 */
async function connect() {
  try {
    const client = await new ClientBuilder().connect(BASE_URL, ROOM_ID);

    // Abort only if the socket closes before the intended test duration.
    // During normal shutdown, k6 can interrupt VUs and close sockets.
    client.ws.addEventListener('close', () => {
      if (exec.instance.currentTestRunDuration < DURATION_MS) {
        exec.test.abort('WebSocket connection closed unexpectedly');
      }
    });

    return client;
  } catch (err) {
    exec.test.abort(`Failed to connect: ${err}`);
    throw err;
  }
}

/**
 * Returns a random float between `min` and `max`
 * @param {number} min - Minimum value
 * @param {number} max - Maximum value
 * @returns {number} Random float between min and max
 */
function randomFloatBetween(min, max) {
  return Math.random() * (max - min) + min;
}
