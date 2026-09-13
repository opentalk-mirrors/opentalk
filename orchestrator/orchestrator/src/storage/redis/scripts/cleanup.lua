-- SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
--
-- SPDX-License-Identifier: EUPL-1.2

local orchestrator_alive_key = KEYS[1]
local orchestrator_services_key = KEYS[2]
local orchestrators_set_key = KEYS[3]

local orchestrator_id = ARGV[1]

-- Skip if this orchestrator is still alive
if redis.call("EXISTS", orchestrator_alive_key) == 1 then
    return 1 -- still alive
end

-- Check that the given orchestrator ID exists in the global orchestrators set
if redis.call("SISMEMBER", orchestrators_set_key, orchestrator_id) == 0 then
    return 2 -- orchestrator not found
end

-- Clean up every service owned by this orchestrator
local service_ids = redis.call("SMEMBERS", orchestrator_services_key)

for _, service_id in ipairs(service_ids) do
    local instance_key = "ot-orchestrator:service:" .. service_id

    -- Remove resource ownership lookup keys before deleting the resource set
    local resources = redis.call("SMEMBERS", instance_key .. ":resources")
    for _, resource in ipairs(resources) do
        redis.call("DEL", "ot-orchestrator:resource:" .. resource .. ":owner")
    end

    -- Remove the service from its kind set
    local kind = redis.call("HGET", instance_key, "kind")
    if kind then
        redis.call("SREM", "ot-orchestrator:services:" .. kind, service_id)
    end

    -- Remove service instance, metrics, and resource keys
    redis.call("DEL", instance_key)
    redis.call("DEL", instance_key .. ":metrics")
    redis.call("DEL", instance_key .. ":resources")
end

-- Remove orchestrator tracking keys
redis.call("DEL", orchestrator_services_key)
redis.call("SREM", orchestrators_set_key, orchestrator_id)

return 0
