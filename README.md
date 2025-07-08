# Features

Supported modules

- [x] control
- [x] moderation
- [x] livekit
- [x] recording
- [ ] recording-service
- [ ] automod
- [ ] breakout
- [ ] chat
- [ ] e2ee
- [ ] legal-vote
- [ ] meeting-notes
- [ ] meeting-report
- [ ] polls
- [ ] shared-folder
- [ ] subroom-audio
- [ ] timer
- [ ] training-participant-report
- [ ] whiteboard

## Core

- [x] OpenID Connect authentication
- [x] Connect as authorized user
- [x] Connect as guest
- [x] Connect as call-in user
- [ ] Connect as recorder

## Control

### Commands

- [x] Join
- [x] Enter From Waiting Room
- [x] Raiser/Lower Hand
- [ ] Grant/Revoke Moderator role

### Events

- [x] JoinSuccess
- [ ] JoinBlocked
- [x] Participant Joined/Updated/Left
- [ ] TimeLimitQuotaElapsed
- [ ] Hand Raised/Lowered (Is this even needed?)
- [ ] RoleUpdated
- [ ] RoomDeleted
- [ ] Error
- [ ] ModeratorRoleGranted/Revoked

## Moderation

### Commands

- [ ] Kick
- [ ] Ban
- [ ] SendToWaitingRoom
- [ ] Debrief
- [ ] ChangeDisplayName
- [ ] Enable/DisableWaitingRoom
- [ ] Enable/DisableRaiseHands
- [ ] Accept (waiting room)
- [ ] ResetRaisedHands

### Events

- [ ] Kicked
- [ ] Banned
- [x] SentToWaitingRoom
- [x] SessionEnded
- [ ] DebriefingStarted
- [x] InWaitingRoom
- [ ] Joined/LeftWaitingRoom
- [ ] WaitingRoomEnabled/Disabled
- [ ] RaiseHandsEnabled/Disabled
- [x] Accepted
- [ ] DisplayNameChanged
- [ ] Error
- [ ] RaisedHandResetByModerator

## Livekit

### Commands

- [ ] CreateNewAccessToken
- [ ] ForceMute
- [ ] Grant/RevokeScreenSharePermission
- [ ] Enable/DisableMicrophoneRestriction
- [ ] RequestPopoutStreamAccessToken

### Events

- [ ] Credentials
- [x] MicrophoneRestrictionsEnabled/Disabled
- [x] ForceMuted
- [ ] PopoutStreamAccessToken
- [ ] ScreenSharePermissionsUpdated
- [ ] Error

## Recording

### Commands

- [x] SetConsent
- [ ] StartStream
- [ ] PauseStream
- [ ] StopStream

### Events

- [x] StreamUpdated
- [ ] Error
- [ ] RecorderError

## Recording Service

### Commands

- [ ] StartStreams
- [ ] PauseStreams
- [ ] StopStreams

### Events

- [ ] StreamUpdated
