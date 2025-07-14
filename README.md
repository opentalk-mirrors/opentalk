# OpenTalk Client (Signaling)

## Features

Supported modules

- ✅ chat
- ✅ control
- ✅ livekit
- ✅ moderation
- ✅ recording
- ✅ recording-service
- ❌ automod
- ❌ breakout
- ❌ e2ee
- ❌ legal-vote
- ❌ meeting-notes
- ❌ meeting-report
- ❌ polls
- ❌ shared-folder
- ❌ subroom-audio
- ❌ timer
- ❌ training-participant-report
- ❌ whiteboard

## Core

- ✅ Connect as authorized user
- ✅ Connect as call-in user
- ✅ Connect as guest
- ✅ Connect as recorder
- ✅ OpenID Connect authentication (ClientCredentials Flow)
- ✅ OpenID Connect authentication (Custom Flow)
- ✅ OpenID Connect authentication (DirectAccess Flow)
- ✅ OpenID Connect authentication (None)
- ❌ OpenID Connect authentication (Device Flow)

## Control

### Commands

- ✅ Enter From Waiting Room
- ✅ Join
- ✅ Raiser/Lower Hand
- ❌ Grant/Revoke Moderator role

### Events

- ✅ JoinSuccess
- ✅ Participant Joined/Updated/Left
- ❌ Error
- ❌ Hand Raised/Lowered (Is this even needed?)
- ❌ JoinBlocked
- ❌ ModeratorRoleGranted/Revoked
- ❌ RoleUpdated
- ❌ RoomDeleted
- ❌ TimeLimitQuotaElapsed

## Moderation

### Commands

- ❌ Accept (waiting room)
- ❌ Ban
- ❌ ChangeDisplayName
- ❌ Debrief
- ❌ Enable/DisableRaiseHands
- ❌ Enable/DisableWaitingRoom
- ❌ Kick
- ❌ ResetRaisedHands
- ❌ SendToWaitingRoom

### Events

- ✅ Accepted
- ✅ InWaitingRoom
- ✅ SentToWaitingRoom
- ✅ SessionEnded
- ❌ Banned
- ❌ DebriefingStarted
- ❌ DisplayNameChanged
- ❌ Error
- ❌ Joined/LeftWaitingRoom
- ❌ Kicked
- ❌ RaisedHandResetByModerator
- ❌ RaiseHandsEnabled/Disabled
- ❌ WaitingRoomEnabled/Disabled

## Livekit

### Commands

- ❌ CreateNewAccessToken
- ❌ Enable/DisableMicrophoneRestriction
- ❌ ForceMute
- ❌ Grant/RevokeScreenSharePermission
- ❌ RequestPopoutStreamAccessToken

### Events

- ✅ ForceMuted
- ✅ MicrophoneRestrictionsEnabled/Disabled
- ❌ Credentials
- ❌ Error
- ❌ PopoutStreamAccessToken
- ❌ ScreenSharePermissionsUpdated

## Recording

### Commands

- ✅ PauseStream
- ✅ SetConsent
- ✅ StartStream
- ✅ StopStream

### Events

- ✅ StreamUpdated
- ❌ Error
- ❌ RecorderError

## Recording Service

### Commands

- ✅ PauseStreams
- ✅ StartStreams
- ✅ StopStreams

### Events

- ❌ StreamUpdated
