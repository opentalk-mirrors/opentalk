These are the templates we currently support. These can be overwritten by setting new paths for each in the config.toml

# Macros

We have a tera macro for creating links. The output is an `a` tag.

# Templates

The following templates are currently supported.

## Registered Invite

Files: _invite.txt, invite.html_

Variables:

| Variable name              | Type           | Description                                       |
| -------------------------- | -------------- | ------------------------------------------------- |
| language                   | string         |                                                   |
| invitee                    | object         | The person which got invited to the event         |
| invitee.email              | string         | Email of the invitee                              |
| invitee.title              | string         | Title of the invitee                              |
| invitee.first_name         | string         | first name of the invitee                         |
| invitee.last_name          | string         | last name of the invitee                          |
| invitee.language           | string         | language of the invitee                           |
| inviter                    | object         | The person which invited the invitee to the event |
| inviter.email              | string         | Email of the inviter                              |
| inviter.title              | string         | Title of the inviter                              |
| inviter.first_name         | string         | first name of the inviter                         |
| inviter.last_name          | string         | last name of the inviter                          |
| inviter.language           | string         | language of the inviter                           |
| event.id                   | string         | id of the event                                   |
| event.name                 | string         | Name of the event                                 |
| event.start_time           | object or null | Start time. _Optional_                            |
| event.start_time?.time     | string         | Time part of the start time                       |
| event.start_time?.timezone | string         | Timezone of the start time                        |
| event.end_time             | object or null | End time. _Optional_                              |
| event.end_time?.time       | string         | Time part of the end time                         |
| event.end_time?.timezone   | string         | Timezone of the end time                          |
| event.rrule                | string or null | Event RRULE _Optional_                            |
| event.description          | string         | Event description                                 |
| event.room                 | object         | The room the event takes place in                 |
| event.room.id              | string         | The rom id                                        |
| event.room.password        | string         | The password, can be empty                        |
| event.call_in              | object or null | The Sip Call In information. _Optional_           |
| event.call_in.sip_tel      | string         | Telephone number to call                          |
| event.call_in.sip_id       | string         | Id of the room to enter                           |
| event.call_in.sip_password | string         | Password of the room to enter                     |
| join_link                  | string         | Link href to the meeting room                     |
| event_link                 | string         | Link href to the dashboard event/meeting page     |
| support                    | object or null | Support contact information                       |
| support.phone              | string         | The support phone number                          |
| support.mail               | string         | The support email address                         |

## Unregistered Invite

Files: _invite.txt, invite.html_

Variables:

| Variable name              | Type           | Description                                       |
| -------------------------- | -------------- | ------------------------------------------------- |
| language                   | string         |                                                   |
| invitee                    | object         | The person which got invited to the event         |
| invitee.email              | string         | Email of the invitee                              |
| invitee.first_name         | string         | first name of the invitee                         |
| invitee.last_name          | string         | last name of the invitee                          |
| inviter                    | object         | The person which invited the invitee to the event |
| inviter.email              | string         | Email of the inviter                              |
| inviter.title              | string         | Title of the inviter                              |
| inviter.first_name         | string         | first name of the inviter                         |
| inviter.last_name          | string         | last name of the inviter                          |
| inviter.language           | string         | language of the inviter                           |
| event.id                   | string         | id of the event                                   |
| event.name                 | string         | Name of the event                                 |
| event.start_time           | object or null | Start time. _Optional_                            |
| event.start_time?.time     | string         | Time part of the start time                       |
| event.start_time?.timezone | string         | Timezone of the start time                        |
| event.end_time             | object or null | End time. _Optional_                              |
| event.end_time?.time       | string         | Time part of the end time                         |
| event.end_time?.timezone   | string         | Timezone of the end time                          |
| event.rrule                | string or null | Event RRULE _Optional_                            |
| event.description          | string         | Event description                                 |
| event.room                 | object         | The room the event takes place in                 |
| event.room.id              | string         | The rom id                                        |
| event.room.password        | string         | The password, can be empty                        |
| event.call_in              | object or null | The Sip Call In information. _Optional_           |
| event.call_in.sip_tel      | string         | Telephone number to call                          |
| event.call_in.sip_id       | string         | Id of the room to enter                           |
| event.call_in.sip_password | string         | Password of the room to enter                     |
| join_link                  | string         | Link href to the meeting room                     |
| event_link                 | string         | Link href to the dashboard event/meeting page     |
| support                    | object or null | Support contact information                       |
| support.phone              | string         | The support phone number                          |
| support.mail               | string         | The support email address                         |

## External Invite

Files: _invite.txt, invite.html_

Variables:

| Variable name              | Type           | Description                                       |
| -------------------------- | -------------- | ------------------------------------------------- |
| language                   | string         |                                                   |
| invitee                    | object         | The person which got invited to the event         |
| invitee.email              | string         | Email of the invitee                              |
| inviter                    | object         | The person which invited the invitee to the event |
| inviter.email              | string         | Email of the inviter                              |
| inviter.title              | string         | Title of the inviter                              |
| inviter.first_name         | string         | first name of the inviter                         |
| inviter.last_name          | string         | last name of the inviter                          |
| inviter.language           | string         | language of the inviter                           |
| event.id                   | string         | id of the event                                   |
| event.name                 | string         | Name of the event                                 |
| event.start_time           | object or null | Start time. _Optional_                            |
| event.start_time?.time     | string         | Time part of the start time                       |
| event.start_time?.timezone | string         | Timezone of the start time                        |
| event.end_time             | object or null | End time. _Optional_                              |
| event.end_time?.time       | string         | Time part of the end time                         |
| event.end_time?.timezone   | string         | Timezone of the end time                          |
| event.rrule                | string or null | Event RRULE _Optional_                            |
| event.description          | string         | Event description                                 |
| event.room                 | object         | The room the event takes place in                 |
| event.room.id              | string         | The rom id                                        |
| event.room.password        | string         | The password, can be empty                        |
| event.call_in              | object or null | The Sip Call In information. _Optional_           |
| event.call_in.sip_tel      | string         | Telephone number to call                          |
| event.call_in.sip_id       | string         | Id of the room to enter                           |
| event.call_in.sip_password | string         | Password of the room to enter                     |
| join_link                  | string         | Link href to the meeting room                     |
| support                    | object or null | Support contact information                       |
| support.phone              | string         | The support phone number                          |
| support.mail               | string         | The support email address                         |
