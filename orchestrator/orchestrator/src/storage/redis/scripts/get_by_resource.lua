-- SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
--
-- SPDX-License-Identifier: EUPL-1.2

local resource_lookup_key = KEYS[1]

local service_id = redis.call("GET", resource_lookup_key)

if not service_id then
    return 1 -- not found
end

local api_key_ids = redis.call("HGET", "ot-orchestrator:service:" .. service_id, "api_key_ids")
local metrics = redis.call("HMGET", "ot-orchestrator:service:" .. service_id .. ":metrics", "load", "accepting_jobs")

return {service_id, api_key_ids, metrics}
