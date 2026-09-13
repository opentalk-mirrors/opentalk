-- SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
--
-- SPDX-License-Identifier: EUPL-1.2

local service_kind_key = KEYS[1]

local service_ids = redis.call("SMEMBERS", service_kind_key)

local result = {}

for i, service_id in ipairs(service_ids) do
    local api_key_ids = redis.call("HGET", "ot-orchestrator:service:" .. service_id, "api_key_ids")
    local metrics     = redis.call("HMGET", "ot-orchestrator:service:" .. service_id .. ":metrics", "load", "accepting_jobs")
    local resources   = redis.call("SMEMBERS", "ot-orchestrator:service:" .. service_id .. ":resources")

    result[i] = {
        service_id,
        api_key_ids or "",
        metrics[1] or "0",
        metrics[2] or "0",
        resources,
    }
end

return result
