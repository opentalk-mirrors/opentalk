-- SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
--
-- SPDX-License-Identifier: EUPL-1.2

local resource_lookup_key = KEYS[1]
local service_kind_key = KEYS[2]

local resource_value = ARGV[1]

local service_id = redis.call("GET", resource_lookup_key)

if service_id then
    -- found a service that manages the given resource
    local api_key_ids = redis.call("HGET", "ot-orchestrator:service:" .. service_id, "api_key_ids");
    local metrics =  redis.call("HMGET", "ot-orchestrator:service:" .. service_id .. ":metrics", "load", "accepting_jobs")

    return {service_id, api_key_ids, metrics}
end

-- select instance with lowest load
local services = redis.call("SMEMBERS", service_kind_key)

local lowest_load = tonumber("101")
local lowest_load_instance = nil
local lowest_load_metrics = nil
local lowest_load_instance_resource_count = nil

for i, service_id in ipairs(services) do
    local metrics = redis.call("HMGET", "ot-orchestrator:service:" .. service_id .. ":metrics", "load", "accepting_jobs")
    local resource_count = redis.call("SCARD", "ot-orchestrator:service:" .. service_id .. ":resources")

    if metrics ~= nil then
        local load = tonumber(metrics[1])
        local accepting_jobs = tonumber(metrics[2])

        if accepting_jobs == 1 then
            if load < lowest_load
                or (load == lowest_load and resource_count < lowest_load_instance_resource_count) then
                lowest_load = load
                lowest_load_instance = service_id
                lowest_load_metrics = metrics
                lowest_load_instance_resource_count = resource_count
            end
        end
    end
end

if not lowest_load_instance then
    return 1 -- no instance available
end

-- add resource to the lowest load instance
local service_id = lowest_load_instance

redis.call("SADD", "ot-orchestrator:service:".. service_id .. ":resources", resource_value)
redis.call("SET", resource_lookup_key, service_id)

local api_key_ids = redis.call("HGET", "ot-orchestrator:service:" .. service_id, "api_key_ids");

return {service_id, api_key_ids, lowest_load_metrics}
