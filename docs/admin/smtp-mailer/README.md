---
title: SMTP Mailer
---

# Administration guide for the {{ product_name }} SMTP Mailer

## General information

The purpose of the {{ product_name }} SMTP Mailer service is to inform users by mail when
a change in an {{ product_name }} instance affects them. Typical notifications are:

- A user has been invited to an event
- A user has been uninvited from an event
- An event has been updated
- An event has been deleted

- [Configuration](configuration.md)
- [Migration guide](migration.md)

## Interaction with other services

### Required services

- [RabbitMQ](rabbitmq.md): {{ product_name }} SMTP Mailer consumes messages from a
  RabbitMQ queue.

## State diagram

- The [State diagram](statediagram.md) describes the different states of the SMTP mailer.
