-- SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
--
-- SPDX-License-Identifier: EUPL-1.2

local orchestrators_set_key = KEYS[1]

local orchestrator_ids = redis.call("SMEMBERS", orchestrators_set_key)

local stale_orchestrators = {}

for _, id in ipairs(orchestrator_ids) do
    if redis.call("EXISTS", "ot-orchestrator:orchestrator:" .. id .. ":alive") == 0 then
        table.insert(stale_orchestrators, id)
    end
end

return stale_orchestrators
