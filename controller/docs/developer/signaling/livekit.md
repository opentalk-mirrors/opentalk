# LiveKit

The LiveKit Signaling Module provides information and access tokens for clients to connect to the LiveKit server.
It also provides updates and commands for moderators to grant and revoke permissions (microphone, screen sharing etc.).

## Joining the room

### JoinSuccess

<!-- This reflects the data structure found at [SignalingModule::FrontendData] -->

When joining a room, the `join_success` control event contains the module-specific fields described below.

| Field                          | Type                                                      | Mandatory | Description                                                          |
| ------------------------------ | --------------------------------------------------------- | --------- | -------------------------------------------------------------------- |
| `credentials`                  | [Credentials](#credentials)                               | Yes       | The current credentials for accessing a room on the LiveKit instance |
| `microphone_restriction_state` | [MicrophoneRestrictionState](#microphonerestrictionstate) | Yes       | The current state of microphone restrictions                         |

#### Credentials

| Field         | Type     | Mandatory | Description                                                                                                                               |
| ------------- | -------- | --------- | ----------------------------------------------------------------------------------------------------------------------------------------- |
| `room`        | `String` | Yes       | The room id for which the credential is valid                                                                                             |
| `token`       | `String` | Yes       | The access token for the controller and frontend                                                                                          |
| `public_url`  | `String` | Yes       | The URL to connect to the LiveKit server. This URL can be used by clients which might from any publicly routed network (e.g. the Web-App) |
| `service_url` | `String` | no        | The URL to connect to the LiveKit server. This URL can be used by clients that run in the same private network as the LiveKit server.     |

:::note `service_url` is only relevant for services running in the same environment as the LiveKit Server

The `service_url` is an alternative URL for the LiveKit server. It makes it possible to
access to the LiveKit server via an alternative route. This might be used in
situations where the client is deployed close to the LiveKit server and should bypass middleboxes.
This field is irrelevant for user clients (e.g. Web-App).

:::

#### MicrophoneRestrictionState

The content of this object depends on the value of the `type` field. The different variants are listed below.

##### Disabled

The force mute state is disabled, participants are allowed to unmute.

| Field  | Type   | Mandatory | Description        |
| ------ | ------ | --------- | ------------------ |
| `type` | `enum` | yes       | Must be `disabled` |

##### Enabled

The force mute state is enabled, only the participants included in `unrestricted_participants` are allowed to unmute.

| Field                       | Type       | Mandatory | Description                                         |
| --------------------------- | ---------- | --------- | --------------------------------------------------- |
| `type`                      | `enum`     | yes       | Must be `disabled`                                  |
| `unrestricted_participants` | `String[]` | yes       | List of  participant IDs that are allowed to unmute |

### Joined

<!-- This reflects the data structure found at [SignalingModule::PeerFrontendData] -->

When joining a room, the `joined` control event sent to all other participants contains no additional fields.

## Commands

<!-- This reflects the data structure found at [SignalingModule::Incoming] -->

### CreateNewAccessToken

Requests the creation of a new access token.

| Field    | Type   | Mandatory | Description                       |
| -------- | ------ | --------- | --------------------------------- |
| `action` | `enum` | yes       | Must be `create_new_access_token` |

#### Example

```json
{
    "action": "create_new_access_token"
}
```

#### Response

[Credential Event](#credentials_1)

### ForceMute

Force mutes participants.

| Field          | Type       | Mandatory | Description                                   |
| -------------- | ---------- | --------- | --------------------------------------------- |
| `action`       | `enum`     | yes       | Must be `force_mute`                          |
| `participants` | `String[]` | yes       | List of participant IDs which should be muted |

#### Example

```json
{
    "action": "force_mute",
    "participants": [
        "00000000-0000-0000-0000-000000000001",
        "00000000-0000-0000-0000-000000000002",
        "00000000-0000-0000-0000-000000000003"
    ]
}
```

#### Response

[ForceMuted Event](#forcemuted)

### GrantScreenSharePermission

Allows the specified participants to share their screens.

| Field          | Type       | Mandatory | Description                                                         |
| -------------- | ---------- | --------- | ------------------------------------------------------------------- |
| `action`       | `enum`     | yes       | Must be `grant_screen_share_permission`                             |
| `participants` | `String[]` | yes       | List of participant IDs that are granted screen sharing permissions |

#### Example

```json
{
    "action": "grant_screen_share_permission",
    "participants": [
        "00000000-0000-0000-0000-000000000001",
        "00000000-0000-0000-0000-000000000002",
        "00000000-0000-0000-0000-000000000003"
    ]
}
```

#### Response

The updated screen share permissions are communicated by the LiveKit server.
The moderator receives a [ScreenSharePermissionsUpdated](#screensharepermissionsupdated) event.

### RevokeScreenSharePermission

Revokes the permission of a participant to share their screen.

| Field          | Type       | Mandatory | Description                                                                   |
| -------------- | ---------- | --------- | ----------------------------------------------------------------------------- |
| `action`       | `enum`     | yes       | Must be `revoke_screen_share_permission`                                      |
| `participants` | `String[]` | yes       | List of participant IDs that who should not be allowed to share their screens |

#### Example

```json
{
    "action": "revoke_screen_share_permission",
    "participants": [
        "00000000-0000-0000-0000-000000000001",
        "00000000-0000-0000-0000-000000000002",
        "00000000-0000-0000-0000-000000000003"
    ]
}
```

#### Response

The updated screen share permissions are communicated by the LiveKit server.
The moderator receives a [ScreenSharePermissionsUpdated](#screensharepermissionsupdated) event.

### EnableMicrophoneRestrictions

Enables the microphone restriction state where only the participants that are part of the
`unrestricted_participants` are allowed to unmute themselves. This will mute
all participants who are not allowed to unmute themselves, but are currently not muted.

| Field                       | Type       | Mandatory | Description                                                               |
| --------------------------- | ---------- | --------- | ------------------------------------------------------------------------- |
| `action`                    | `enum`     | yes       | Must be `enable_microphone_restrictions`                                  |
| `unrestricted_participants` | `String[]` | yes       | List of participant IDs that are still allowed to enable their microphone |

#### Example

```json
{
    "action": "enable_microphone_restrictions",
    "unrestricted_participants": [
        "00000000-0000-0000-0000-000000000001",
        "00000000-0000-0000-0000-000000000002",
        "00000000-0000-0000-0000-000000000003"
    ]
}
```

#### Response

[MicrophoneRestrictionsEnabled Event](#microphonerestrictionsenabled)

### DisableMicrophoneRestrictions

Disable the microphone restriction state which will allow all participants to unmute their microphone again.

| Field    | Type   | Mandatory | Description                               |
| -------- | ------ | --------- | ----------------------------------------- |
| `action` | `enum` | yes       | Must be `disable_microphone_restrictions` |

#### Example

```json
{
    "action": "disable_microphone_restrictions"
}
```

#### Response

[MicrophoneRestrictionsDisabled Event](#microphonerestrictionsdisabled)

### RequestPopoutStreamAccessToken

Requests a special, more restrictive, livekit access token for the creation of a popout stream.

This access token is bound to the requesting participants identity. This token can only subscribe and is
hidden to other participants.

| Field    | Type   | Mandatory | Description                                  |
| -------- | ------ | --------- | -------------------------------------------- |
| `action` | `enum` | yes       | Must be `request_popout_stream_access_token` |

#### Example

```json
{
    "action": "request_popout_stream_access_token"
}
```

#### Response

[PopoutStreamAccessToken](#popoutstreamaccesstoken)

## Events

<!-- This reflects the data structure found at [SignalingModule::Outgoing] -->

### Credentials

The credentials for a client to use LiveKit

| Field         | Type     | Mandatory | Description                                                                                                                               |
| ------------- | -------- | --------- | ----------------------------------------------------------------------------------------------------------------------------------------- |
| `message`     | `enum`   | yes       | Is `credentials`                                                                                                                          |
| `room`        | `String` | Yes       | The room id for which the credential is valid                                                                                             |
| `token`       | `String` | Yes       | The access token for the controller and frontend                                                                                          |
| `public_url`  | `String` | Yes       | The URL to connect to the LiveKit server. This URL can be used by clients which might from any publicly routed network (e.g. the Web-App) |
| `service_url` | `String` | no        | The URL to connect to the LiveKit server. This URL can be used by clients that run in the same private network as the LiveKit server.     |

:::note `service_url` is only relevant for services running in the same environment as the LiveKit Server

The `service_url` is an alternative URL for the LiveKit server. It makes it possible to
access to the LiveKit server via an alternative route. This might be used in
situations where the client is deployed close to the LiveKit server and should bypass middleboxes.
This field is irrelevant for user clients (e.g. Web-App).

:::

### MicrophoneRestrictionsEnabled

The moderator enabled the microphone-restriction-state. Only participants listed in `unrestricted_participants` are able to unmute themselves.

| Field                       | Type       | Mandatory | Description                                                               |
| --------------------------- | ---------- | --------- | ------------------------------------------------------------------------- |
| `message`                   | `enum`     | yes       | Is `microphone_restrictions_enabled`                                      |
| `unrestricted_participants` | `String[]` | yes       | List of participant IDs that are still allowed to enable their microphone |

### MicrophoneRestrictionsDisabled

The moderator disabled the microphone-restriction-state. Participants are allowed to unmute themselves again.

| Field     | Type   | Mandatory | Description                           |
| --------- | ------ | --------- | ------------------------------------- |
| `message` | `enum` | yes       | Is `microphone_restrictions_disabled` |

### ForceMuted

The moderator has force muted the participant.

| Field       | Type     | Mandatory | Description                                                                |
| ----------- | -------- | --------- | -------------------------------------------------------------------------- |
| `message`   | `enum`   | yes       | Is `force_muted`                                                           |
| `moderator` | `String` | yes       | The participant ID of the moderator who muted the receiver of this message |

### PopoutStreamAccessToken

The user was granted a popout stream access token.

| Field     | Type     | Mandatory | Description                                                     |
| --------- | -------- | --------- | --------------------------------------------------------------- |
| `message` | `enum`   | yes       | Is `popout_stream_access_token`                                 |
| `token`   | `String` | yes       | A restrictive livekit access token for creating a popout stream |

### ScreenSharePermissionsUpdated

LiveKit screen share permissions of the participant have been updated.

| Field          | Type       | Mandatory | Description                                                                |
| -------------- | ---------- | --------- | -------------------------------------------------------------------------- |
| `message`      | `enum`     | yes       | Is `popout_stream_access_token`                                            |
| `grant`        | `bool`     | yes       | `true` if screen share permissions where granted, `false` otherwise.       |
| `participants` | `String[]` | yes       | The IDs of the participants who received a screen share permission update. |

### Error

Errors returned by the LiveKit Signaling Module.

#### LivekitUnavailable

The LiveKit server is not available.

| Field     | Type   | Mandatory | Description              |
| --------- | ------ | --------- | ------------------------ |
| `message` | `enum` | yes       | Is `error`               |
| `error`   | `enum` | yes       | Is `livekit_unavailable` |

#### InsufficientPermissions

The client has missing permissions for performing the action on the LiveKit server.

| Field     | Type   | Mandatory | Description                   |
| --------- | ------ | --------- | ----------------------------- |
| `message` | `enum` | yes       | Is `error`                    |
| `error`   | `enum` | yes       | Is `insufficient_permissions` |
