# OpenTalk RoomServer

This project contains the crates that implement the OpenTalk RoomServer.

- [OpenTalk RoomServer](opentalk-roomserver/src/main.rs) – The crate which produces the actual RoomServer executable. It implements the OpenTalk RoomServer Web API.
- [OpenTalk RoomServer Client](./opentalk-roomserver-client/src/lib.rs) – HTTP-Client which can be used to communicate with the OpenTalk RoomServer using the _OpenTalk RoomServer Web API_
- [OpenTalk RoomServer Common](opentalk-roomserver-common/src/lib.rs) – Types that are shared between the OpenTalk RoomServer and Signaling Modules
- [OpenTalk RoomServer Crypto Provider](./opentalk-roomserver-crypto-provider/src/lib.rs) – Crypto provider setup for the OpenTalk RoomServer
- [OpenTalk RoomServer DUI](./opentalk-roomserver-dui/src/main.rs) – Graphical interface for the RoomServer signaling, used for testing and development
- [OpenTalk RoomServer Modules](./opentalk-roomserver-modules/src/lib.rs) – Initializes the signaling module registrie with all modules
- [OpenTalk RoomServer Room](./opentalk-roomserver-room/src/lib.rs) – Contains code for room management and signaling
- [OpenTalk RoomServer Signaling](./opentalk-roomserver-signaling/src/lib.rs) – Types required to develop signaling modules. Most prominently the `SignalingModule` trait.
- [OpenTalk RoomServer Types](./opentalk-roomserver-types/src/lib.rs) – Types that are sent over the wire. These types are used in the HTTP-API and shared between the RoomServer crates.
- [OpenTalk RoomServer Web API](./opentalk-roomserver-web-api/src/lib.rs) – HTTP-API endpoints written with Axum. This crate defines the API but does not implement any logic. All requests are forwarded to trait implementations.

## Developer UI

The RoomServer DUI is used to debug the signaling interface.

You can execute it using:

```bash
just run-dui
```

## Typst Packages

Some modules require the [linguify](https://github.com/typst-community/linguify) typst package for localized report generation. This package is included in the container of the RoomServer. For some tests, it is also necessary to have this package present locally. It can be obtained/updated by running `just install-latest-typst-packages`.

## Documentation

Documentation for the RoomServer is build in [the unified documentation project](https://git.opentalk.dev/opentalk/backend/docs).
The content is copied from `<roomserver-project-root>/docs` and placed in the documentation project automatically.

### Copy files outside of `docs/`

The documentation contains verbatim copies of files outside the `docs/` folder
(e.g. `example/roomserver.toml`). The [`ci-docs-updater`](https://git.opentalk.dev/opentalk/tools/opentalk-ci-doc-updater) is used to ensure that the documentation stays in sync with the original files.

You can update the documentation running:

```bash
just update-docs
```
