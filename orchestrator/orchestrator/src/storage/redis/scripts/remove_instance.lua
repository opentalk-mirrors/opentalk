-- SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
--
-- SPDX-License-Identifier: EUPL-1.2

local instance_key = KEYS[1]
local service_kind_key = KEYS[2]
local service_instance_key = KEYS[3]

local service_id = ARGV[1]

if redis.call("EXISTS", instance_key) == 0 then
    return 1 -- instance not found
end

-- Get all resource keys associated with the instance
local resource_keys = redis.call("SMEMBERS", instance_key .. ":resources")

for i, resource in ipairs(resource_keys) do
    -- Remove the resource lookup key for each resource
    redis.call("DEL", "ot-orchestrator:resource:" .. resource .. ":owner")
end

-- Remove the keys related to the instance
redis.call("DEL", instance_key)
redis.call("DEL", instance_key .. ":metrics")
redis.call("DEL", instance_key .. ":resources")
redis.call("SREM", service_kind_key, service_id)
redis.call("SREM", service_instance_key, service_id)

return 0 -- Ok
