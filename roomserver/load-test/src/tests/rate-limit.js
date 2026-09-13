// SPDX-License-Identifier: EUPL-1.2
// SPDX-FileCopyrightText: OpenTalk Team <mail@opentalk.eu>
/**
 * WebSocket Rate Limit Test
 * Sends a fixed number of echo messages, faster than the rate limit.
 * Expected behavior is that messages will be delayed according to the rate limit.
 * No messages should be dropped.
 *
 * Metrics:
 * - rate_limit_sent: number of sent messages.
 * - rate_limit_received: number of received responses.
 */
import exec from 'k6/execution';
import { Counter } from 'k6/metrics';

import { ClientBuilder } from '../lib/client.js';
import { getEnv, getRequiredEnv } from '../lib/environment.js';
import { sendCommand } from '../lib/messaging.js';
import { shared_tags } from '../lib/metrics.js';

// Test metadata
const TEST_NAME = 'rate_limit';
const TEST_START_TIME = new Date();

// Room configuration
const BASE_URL = getRequiredEnv('BASE_URL');
const ROOM_ID = getEnv('ROOM_ID', '27c66df5-f6be-4d70-a167-abba2cf28a2a');

// Test configuration
const TOTAL_SEND = getEnv('RATE_LIMIT_TOTAL_SEND', 1000);
const TOKENS_PER_SECOND = getEnv('RATE_LIMIT_TOKENS_PER_SECOND', 10);
const TOKEN_BUCKET_SIZE = getEnv('RATE_LIMIT_TOKEN_BUCKET_SIZE', 15);

// Metrics
const SENT = new Counter('rate_limit_sent');
const RECEIVED = new Counter('rate_limit_received');

export const options = {
  scenarios: {
    rate_limit: {
      executor: 'constant-vus',
      vus: 1,
      duration: `${TOTAL_SEND / TOKENS_PER_SECOND}s`,
      gracefulStop: '0s',
    },
  },
};

export default async function () {
  const tags = shared_tags(TEST_NAME, TEST_START_TIME);
  let transactionId = 0;
  let pending = new Set();

  let client;
  try {
    client = await new ClientBuilder().withRateLimit(TOKENS_PER_SECOND, TOKEN_BUCKET_SIZE).connect(BASE_URL, ROOM_ID);
  } catch (err) {
    exec.test.abort(`Failed to connect: ${err}`);
    return;
  }

  client.ws.addEventListener('close', () => {
    exec.test.abort('WebSocket connection closed unexpectedly');
  });

  client.ws.onmessage = (event) => {
    const msg = JSON.parse(event.data);
    if (msg.transaction_id === undefined || !pending.has(msg.transaction_id)) return;

    pending.delete(msg.transaction_id);
    RECEIVED.add(1, tags);
  };

  for (let i = 0; i < TOTAL_SEND; i++) {
    const currentTransactionId = transactionId++;

    // Send an echo command without awaiting the response
    sendCommand(client.ws, 'echo', { action: 'ping' }, currentTransactionId);
    pending.add(currentTransactionId);
    SENT.add(1, tags);
  }

  // Not using k6's sleep() here, because it would suspend the VU, preventing it
  // from processing incoming messages.
  const waitMs = (TOTAL_SEND / TOKENS_PER_SECOND) * 1000;
  await new Promise((resolve) => setTimeout(resolve, waitMs));

  client.disconnect();
}
