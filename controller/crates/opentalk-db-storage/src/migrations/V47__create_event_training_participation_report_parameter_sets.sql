CREATE TABLE event_training_participation_report_parameter_sets (
    event_id UUID PRIMARY KEY REFERENCES events(id) ON DELETE CASCADE NOT NULL,
    initial_checkpoint_delay_after BIGINT NOT NULL,
    initial_checkpoint_delay_within BIGINT NOT NULL,
    checkpoint_interval_after BIGINT NOT NULL,
    checkpoint_interval_within BIGINT NOT NULL
);
