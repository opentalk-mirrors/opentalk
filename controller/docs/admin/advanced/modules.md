# Modules

Modules provide functionality that can be used inside meetings.

<!-- begin:fromfile:modules/module-features-documentation.md -->

## Module `automod`

Handles auto-moderation functionality such as the talking stick

### Features

This module does not provide any configurable features.

### Example

```toml
[automod]
```

## Module `breakout`

Handles breakout room functionality

### Features

This module does not provide any configurable features.

## Module `chat`

Handles room chat functionality

### Features

This module does not provide any configurable features.

### Example

```toml
[chat.rate_limit]
slow_down_threshold = 0.75
token_bucket_size = 30
tokens_per_second = 10
```

## Module `core`

Handles core room functionality such as joining and leaving

### Features

The following features can be configured for the module. All features are enabled by default and can be disabled either [by configuration](https://docs.opentalk.eu/admin/controller/advanced/defaults/) or [by tariff](https://docs.opentalk.eu/admin/controller/advanced/tariffs/).

#### `core::call_in`

Enables telephone call-in functionality for meetings.

#### `core::guests_allowed`

Allow guest access. With this feature enabled, guests are allowed to join meetings through invite links. When turned off, no invite links are generated, and invite links that existed before will be invalid.

#### `core::storage_upgradable`

Communicates to the frontend that the user's storage can be upgraded. Frontend will then show the corresponding link to the account management if the user's storage is close to the limit. This feature is usually configured differently the tariffs. If a user has a tariff which already provides the maximum available storage space, then that feature should be disabled. For all other tariffs it should be on.

## Module `e2ee`

Handles end-to-end encryption functionality

### Features

This module does not provide any configurable features.

### Example

```toml
[e2ee]
```

## Module `echo`

Used for internal connection checking and development

### Features

This module does not provide any configurable features.

### Example

```toml
[echo]
```

## Module `excalidraw`

Handles excalidraw whiteboard integration. Excalidraw is a collaborative drawing board.

### Features

This module does not provide any configurable features.

### Example

```toml
[excalidraw]
```

## Module `legal_vote`

Handles the legal-vote functionality

### Features

This module does not provide any configurable features.

### Example

```toml
[legal_vote]
```

## Module `livekit`

Handles Livekit media streams coordination and integration

### Features

This module does not provide any configurable features.

### Example

```toml
[livekit]
api_key = "devkey"
api_secret = "secret"
public_url = "http://localhost:7880"
service_url = "http://localhost:7880/"
```

## Module `meeting_notes`

Handles meeting note editing and viewing functionality

### Features

This module does not provide any configurable features.

### Example

```toml
[meeting_notes]
api_key = "secret"
base_url = "http://localhost:9001/"
```

## Module `meeting_report`

Handles generation of meeting reports, e.g. participant list export

### Features

This module does not provide any configurable features.

### Example

```toml
[meeting_report]
```

## Module `moderation`

Handles moderation functionality

### Features

This module does not provide any configurable features.

### Example

```toml
[moderation]
```

## Module `polls`

Handles meeting polls functionality

### Features

This module does not provide any configurable features.

### Example

```toml
[polls]
```

## Module `raise_hands`

Handles the raising hand functionality

### Features

This module does not provide any configurable features.

### Example

```toml
[raise_hands]
```

## Module `reaction`

Handles emoji reactions

### Features

This module does not provide any configurable features.

### Example

```toml
[reaction]
```

## Module `recording`

Handles recording functionality.

### Features

The following features can be configured for the module. All features are enabled by default and can be disabled either [by configuration](https://docs.opentalk.eu/admin/controller/advanced/defaults/) or [by tariff](https://docs.opentalk.eu/admin/controller/advanced/tariffs/).

#### `recording::record`

Allows creation of recordings for meetings

#### `recording::stream`

Allows streaming meetings to streaming services

### Example

```toml
[recording]
url = "http://localhost:5555/"

[recording.api_key]
id = "recorder"
secret = "secret"
```

## Module `shared_folder`

Handles shared folder integration. This allows automatic creation of shares on a NextCloud instance using the [OCS API](https://docs.nextcloud.com/server/latest/developer_manual/client_apis/OCS/ocs-api-overview.html) or on an OpenCloud instance using its [Graph API](https://owncloud.dev/libre-graph-api/) and [WebDAV](https://docs.opencloud.eu/docs/dev/apis/webdav) endpoints.

### Features

This module does not provide any configurable features.

### Example

```toml
[shared_folder]
```

## Module `subroom_audio`

Handles sub-room audio, allowing participants to talk to each other in a separate audio group.

### Features

This module does not provide any configurable features.

### Example

```toml
[subroom_audio]
```

## Module `timer`

Handles timer functionality including the coffee-break timer.

### Features

This module does not provide any configurable features.

### Example

```toml
[timer]
```

## Module `training_participation_report`

Handles training participation report functionality. Participants are asked to confirm their presence repeatedly at pre-configured time intervals. These confirmations are documented in the training participation report which is created automatically at the end of the meeting.

### Features

This module does not provide any configurable features.

### Example

```toml
[training_participation_report.autostart.checkpoint_interval]
after = 300
within = 400

[training_participation_report.autostart.initial_checkpoint_delay]
after = 100
within = 200
```

## Module `transcription`

Live transcription for meetings

### Features

The following features can be configured for the module. All features are enabled by default and can be disabled either [by configuration](https://docs.opentalk.eu/admin/controller/advanced/defaults/) or [by tariff](https://docs.opentalk.eu/admin/controller/advanced/tariffs/).

#### `transcription::transcription`

Allows to create transcriptions for meetings

### Example

```toml
[transcription]
url = "http://localhost:11413/"

[transcription.api_key]
id = "transcription"
secret = "secret"
```

## Module `whiteboard`

Handles whiteboard integration. The whiteboard is a collaborative drawing board that can be used during the meeting.

### Features

This module does not provide any configurable features.

### Example

```toml
[whiteboard]
api_key = "secret"
base_url = "http://localhost:9666/"
```

<!-- end:fromfile:modules/module-features-documentation.md -->

## `opentalk-controller modules` subcommand

This command outputs all modules available in the {{ product_name }} controller, including
the features that can be enabled or disabled:

```text
opentalk-controller modules list
```

Example output:

<!-- begin:fromfile:cli-usage/opentalk-controller-modules-list.md -->

```text
automod: []
breakout: []
chat: []
core: ["call_in", "guests_allowed", "storage_upgradable"]
e2ee: []
echo: []
excalidraw: []
legal_vote: []
livekit: []
meeting_notes: []
meeting_report: []
moderation: []
polls: []
raise_hands: []
reaction: []
recording: ["record", "stream"]
shared_folder: []
subroom_audio: []
timer: []
training_participation_report: []
transcription: ["transcription"]
whiteboard: []
```

<!-- end:fromfile:cli-usage/opentalk-controller-modules-list.md -->
