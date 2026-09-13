CREATE TYPE job_type AS ENUM ('adhoc_event_cleanup', 'event_cleanup', 'invite_cleanup', 'self_check');
CREATE TYPE job_status AS ENUM ('started', 'succeeded', 'failed');
CREATE TYPE log_level AS ENUM ('trace', 'debug', 'info', 'warn', 'error');

CREATE TABLE jobs (
    id BIGSERIAL PRIMARY KEY,
    name TEXT NOT NULL,
    kind job_type NOT NULL,
    parameters JSONB NOT NULL,
    timeout_secs INTEGER CHECK(timeout_secs > 0) NOT NULL,
    -- cron string
    recurrence TEXT NOT NULL
);

CREATE TABLE job_executions (
    id BIGSERIAL PRIMARY KEY,
    job_id BIGINT REFERENCES jobs(id) ON DELETE CASCADE NOT NULL,
    started_at TIMESTAMPTZ NOT NULL,
    ended_at TIMESTAMPTZ,
    job_status job_status NOT NULL
);

CREATE TABLE job_execution_logs (
    id BIGSERIAL PRIMARY KEY,
    execution_id BIGINT REFERENCES job_executions(id) ON DELETE CASCADE NOT NULL,
    logged_at TIMESTAMPTZ NOT NULL,
    log_level log_level NOT NULL,
    log_message TEXT NOT NULL
);
