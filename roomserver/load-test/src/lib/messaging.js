// SPDX-License-Identifier: EUPL-1.2
// SPDX-FileCopyrightText: OpenTalk Team <mail@opentalk.eu>

/**
 * Send a command message through the WebSocket connection
 * @param ws - The WebSocket connection
 * @param namespace - The module namespace (e.g., 'echo', 'core', 'chat')
 * @param payload - The command payload
 * @param transactionId - Optional transaction ID for tracking responses
 */
export function sendCommand(ws, namespace, payload, transactionId) {
  const command = {
    namespace,
    payload,
    ...(transactionId !== undefined && { transaction_id: transactionId }),
  };
  ws.send(JSON.stringify(command));
}
