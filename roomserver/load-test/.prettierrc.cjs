/*
* SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
* SPDX-License-Identifier: EUPL-1.2
*/

/**
 * @see https://prettier.io/docs/en/configuration.html
 * @type {import("prettier").Config}
 */
const config = {
  singleQuote: true,
  trailingComma: 'es5',
  printWidth: 120,
  importOrder: ["^@core/(.*)$", "^@server/(.*)$", "^@ui/(.*)$", "^[./]"],
  importOrderSeparation: true,
  plugins: ["@trivago/prettier-plugin-sort-imports"]
};

module.exports = config;
