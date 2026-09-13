# Retention of Personal Data

This document describes how the {{ product_name }} service stores personal data, e.g. to cover GPDR compliance.

## What data is stored

The {{ product_name }} service collects the following types of data:

- Personal identification data (name, email address and phone number)
- Streaming keys
- Recordings
- Whiteboard data
- Votings and meeting-notes responses
- Logs, depending on the mode used

## How is your data collected

Users directly provide the service with their personal data. Data is collected when a user:

- Registers on the service
- Adds streaming information to their account
- Agrees to be part of a recording
- Participates in a voting/meeting-notes/whiteboard session

## Why is it stored

Personal data is stored for account management, communication and to enhance user experience.

Object Storage is used to provide access to recordings and whiteboard data.

## Where do we store your data

Personal identification data is stored in a postgres [database](./core/database.md).

Object Storage (recordings, whiteboard) is stored on a [MinIO](./core/minio.md) object storage instance.

Voting and meeting-notes responses are temporarily stored in a Redis database. Entries are automatically deleted after a meeting concludes, which occurs when the last participant leaves the meeting or upon service restarts

## How long is it stored

- Personal identification data is currently stored indefinitely due to the absence of a purge mechanism. We are actively developing a solution to address this issue
- Streaming keys are stored indefinitely unless deleted by the user
- Data in the object storage (e.g. recordings and whiteboard data) is stored until the associated event is deleted via a cleanup job or manual deletion
- Logs don't contain any personal data by default. Altering the log level to a more verbose variant beyond `INFO`, which we dont recommend, can lead to the inclusion of personal data in the logs

### Cleanup Jobs

These jobs ensure the deletion of data associated with their event types, with the threshold for deletion being configurable via a parameter. Currently, these jobs are not executed automatically and require manual setup by the administrator.

- The [`event-cleanup` job](./cli/jobs.md#job-event-cleanup) is for deleting non-recurring events after a certain duration.
- The [`adhoc-event-cleanup` job](./cli/jobs.md#job-adhoc-event-cleanup) is for deleting adhoc events created a certain duration ago.
