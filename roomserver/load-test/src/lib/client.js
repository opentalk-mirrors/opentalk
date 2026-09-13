// SPDX-License-Identifier: EUPL-1.2
// SPDX-FileCopyrightText: OpenTalk Team <mail@opentalk.eu>
import exec from 'k6/execution';
import { WebSocket } from 'k6/experimental/websockets';
import { setTimeout, clearTimeout } from 'k6/timers';

import { requestRoomToken, getSignalingUrl } from './auth.js';
import { sendCommand } from './messaging.js';
import { createClientParameter, createRoomParameter } from './parameters.js';

export class ClientBuilder {
  /**
   * Configure the RoomServers rate limiting
   * @param {number} tokensPerSecond
   * @param {number} tokenBucketSize
   * @returns {ClientBuilder} The client builder
   */
  withRateLimit(tokensPerSecond, tokenBucketSize) {
    this.rateLimit = { tokens_per_second: tokensPerSecond, token_bucket_size: tokenBucketSize };
    return this;
  }

  /**
   * Open a WebSocket connection to the RoomServer and join the specified room
   * @param {string} baseUrl - Base URL of the RoomServer
   * @param {string} roomId - The ID of the room to join
   * @param {string} roomId - The ID of the user joining the room
   * @returns {Promise<Client>} The connected client
   */
  async connect(baseUrl, roomId, userId) {
    // Create parameters for this virtual user
    const roomParams = createRoomParameter(this.rateLimit);
    const clientParams = createClientParameter(__VU, __ITER);

    // Request room token
    const accessInfo = requestRoomToken(baseUrl, roomId, {
      room_parameters: roomParams,
      client_parameters: clientParams,
    });

    // Get WebSocket URL
    const signalingUrl = getSignalingUrl(accessInfo);

    // Connect websocket
    const ws = new WebSocket(signalingUrl);

    await new Promise((resolve, reject) => {
      // Fail if we don't receive the joined_lobby message in time
      const timerId = setTimeout(() => {
        const msg = 'Timeout waiting for joined_lobby message';
        reject(new Error(msg));
        exec.test.abort(msg);
      }, 1_000);

      ws.onmessage = (event) => {
        const message = JSON.parse(event.data);
        if (message.payload.message == 'joined_lobby') {
          clearTimeout(timerId);
          resolve();
        }
      };
    });

    const client = new Client(ws);
    let response = await client.sendEnterRoom(0);
    if (response.message !== 'join_success') {
      exec.test.abort('Failed to join room');
    }

    return client;
  }
}

export class Client {
  constructor(ws) {
    this.ws = ws;
    this.events = new Map();

    ws.onmessage = (event) => {
      if (this.events.size === 0) return;

      const message = JSON.parse(event.data);
      const resolve = this.events.get(message.transaction_id);
      if (resolve === undefined) return;

      resolve(message);
      this.events.delete(message.transaction_id);
    };

    ws.onerror = (error) => {
      console.error(`Received error from WebSocket: ${JSON.stringify(error)}`);
    };
  }

  /**
   * Sends an arbitrary command to the RoomServer
   * @param {string} namespace - The namespace of the command
   * @param {any} payload - The content of the command
   * @param {number} transactionId - The transaction ID of the command
   * @returns {Promise<any>} The corresponding event send by the RoomServer
   */
  sendCommand(namespace, payload, transactionId) {
    const self = this;
    const promise = new Promise((resolve, _reject) => {
      self.events.set(transactionId, resolve);
    });
    sendCommand(this.ws, namespace, payload, transactionId);
    return promise;
  }

  /**
   * Sends the core command to enter the room
   * @param {number} transactionId - The transaction ID of the command
   * @returns {Promise<any>} The corresponding event send by the RoomServer
   */
  sendEnterRoom(transactionId) {
    return this.sendCommand('core', { action: 'enter_room' }, transactionId);
  }

  /**
   * Sends a echo ping command
   * @param {number} transactionId - The transaction ID of the command
   * @returns {Promise<any>} The corresponding event send by the RoomServer
   */
  sendEcho(transactionId) {
    return this.sendCommand('echo', { action: 'ping' }, transactionId);
  }

  /**
   * @typedef {import('./chat-scope.js').ChatScope} ChatScope
   */

  /**
   * Sends a chat message
   * @param {string} content The content of the message
   * @param {ChatScope} scope
   * @param {number} transactionId The transaction ID of the command
   * @returns {Promise<any>} The corresponding event send by the RoomServer
   */
  sendChatMessage(content, scope, transactionId) {
    const cmd = { action: 'send_message', content, ...scope };
    return this.sendCommand('chat', cmd, transactionId);
  }

  /**
   * Disconnects the client, closing its WebSocket connection
   */
  disconnect() {
    this.ws.close();
  }
}
