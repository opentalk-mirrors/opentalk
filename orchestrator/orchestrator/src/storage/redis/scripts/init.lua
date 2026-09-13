-- SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
--
-- SPDX-License-Identifier: EUPL-1.2

local orchestrator_alive_key = KEYS[1]
local orchestrators_set_key = KEYS[2]

local orchestrator_id = ARGV[1]
local ttl = tonumber(ARGV[2])

-- Guard against the case of a UUID collision
if redis.call("EXISTS", orchestrator_alive_key) == 1 then
    return 1 -- UUID collision
end

-- Register this orchestrator with an expiring alive key (heartbeat target)
redis.call("SET", orchestrator_alive_key, orchestrator_id, "EX", ttl)

-- Track this orchestrator in the global set
redis.call("SADD", orchestrators_set_key, orchestrator_id)

return 0
