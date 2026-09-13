// SPDX-License-Identifier: EUPL-1.2
// SPDX-FileCopyrightText: OpenTalk Team <mail@opentalk.eu>

/**
 * @typedef {Object} GlobalScope
 */

/**
 * Create a global scope descriptor for chat commands.
 * @returns {GlobalScope}
 */
export function createGlobalScope() {
  return Object.freeze({ scope: 'global' });
}

/**
 * @typedef {Object} BreakoutScope
 * @property {'breakout'} scope
 * @property {number} target
 */

/**
 * Create a breakout scope descriptor for chat commands.
 * @param {number} breakoutId
 * @returns {BreakoutScope}
 */
export function createBreakoutScope(breakoutId) {
  return Object.freeze({ scope: 'breakout', target: breakoutId });
}

/**
 * @typedef {Object} PrivateScope
 * @property {'private'} scope
 * @property {string} target
 */

/**
 * Create a private scope descriptor for chat commands.
 * @param {string} participantId
 * @returns {PrivateScope}
 */
export function createPrivateScope(participantId) {
  return Object.freeze({ scope: 'private', target: participantId });
}

/**
 * @typedef {GlobalScope | BreakoutScope | PrivateScope} ChatScope
 */
