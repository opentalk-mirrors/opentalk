// SPDX-License-Identifier: EUPL-1.2
// SPDX-FileCopyrightText: OpenTalk Team <mail@opentalk.eu>

const ENV_PREFIX = 'LOAD_TEST_';

/**
 * Get an environment variable with the common prefix
 * @param {string} key - The variable name without prefix
 * @param {string} defaultValue - Default value if not set
 * @returns {string} The environment variable value
 */
export function getEnv(key, defaultValue = '') {
  const fullKey = `${ENV_PREFIX}${key}`;
  const value = __ENV[fullKey];

  if (value === undefined || value === null || value === '') {
    if (defaultValue === '') {
      console.warn(`Environment variable ${fullKey} is not set and no default provided`);
    }
    return defaultValue;
  }

  // Try to match the type of the default value
  if (typeof defaultValue === 'boolean') {
    return /^(true|1|yes|on)$/i.test(String(value).trim());
  }

  const parse = defaultValue?.constructor;
  if (typeof parse === 'function') {
    return parse(value);
  }

  return value;
}

/**
 * Get a required environment variable with the common prefix
 * Throws an error if the variable is not set
 * @param {string} key - The variable name without prefix
 * @returns {string} The environment variable value
 */
export function getRequiredEnv(key) {
  const fullKey = `${ENV_PREFIX}${key}`;
  const value = __ENV[fullKey];

  if (value === undefined || value === null || value === '') {
    throw new Error(`Required environment variable ${fullKey} is not set`);
  }

  return value;
}
