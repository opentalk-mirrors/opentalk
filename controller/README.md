<!--
SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>

SPDX-License-Identifier: EUPL-1.2
-->

# OpenTalk Controller

See the [administration guide](docs/admin/README.md) for more information.

## Configuration

See the [configuration](docs/admin/configuration.md) chapter of the
administration guide for more information.

An example configuration is available in the
[`example/controller.toml`](example/controller.toml) file. It can be copied to the root directory:

```sh
cp ./example/controller.toml ./controller.toml
```

## Upgrading

See the [migration guide](docs/admin/migration.md) for information about upgrading.

## Build the container image

If you just want an image, that can be created with:

```bash
docker build . --tag <your tag>
```

If required, the image can be built with one of a small set of different base
images, a `Dockerfile-<baseimage>` is located in the root directory of this
project for each of them. `Dockerfile` is a symlink to the latest default image.

To build the image with a specific base image, execute in the root of the repository:

```bash
docker build -f Dockerfile-<baseimage> . --tag <your tag>
```

## Sub-crates

Inside the `crates` folder, the following crates can be found:

- [opentalk-cache](crates/opentalk-cache)
    - Redis-aware local in-memory cache layer used to reduce repeated remote lookups.
- [opentalk-controller](crates/opentalk-controller)
    - Main controller crate and application entry point
- [opentalk-controller-api-actix-web](crates/opentalk-controller-api-actix-web)
    - `actix-web` specific API integration layer (HTTP extraction/response helpers and OpenAPI-related web bindings) over the service facade.
- [opentalk-controller-api-authorization](crates/opentalk-controller-api-authorization)
    - Core Web API authorization interfaces and middleware-level abstractions.
- [opentalk-controller-api-authorization-database](crates/opentalk-controller-api-authorization-database)
    - Implementation of controller Web API authorization logic backed by database data.
- [opentalk-controller-core](crates/opentalk-controller-core)
    - Shared controller core domain logic and abstractions
- [opentalk-controller-service](crates/opentalk-controller-service)
    - Main application service layer implementing controller business workflows (rooms/users/auth/integrations), including cache, messaging, and external service clients.
- [opentalk-controller-service-facade](crates/opentalk-controller-service-facade)
    - Stable facade/types trait layer for the service crate, consumed by API and integration crates.
- [opentalk-controller-settings](crates/opentalk-controller-settings)
    - Typed configuration loading/validation for controller runtime settings.
- [opentalk-controller-utils](crates/opentalk-controller-utils)
    - Shared controller utility code.
- [opentalk-database](crates/opentalk-database)
    - Database foundation crate: async Diesel/PostgreSQL pool setup, connection handling, and DB-related telemetry/error plumbing.
- [opentalk-db-storage](crates/opentalk-db-storage)
    - Persistence/storage layer crate with Diesel models, schema-facing types, migration support and DB-backed repository utilities.
- [opentalk-inventory](crates/opentalk-inventory)
    - Inventory domain crate defining inventory model abstractions and related typed data structures shared in controller logic.
- [opentalk-inventory-common](crates/opentalk-inventory-common)
    - Minimal shared inventory primitives and error/common types used by inventory crates.
- [opentalk-inventory-database](crates/opentalk-inventory-database)
    - Database adapter/implementation for inventory domain operations, bridging inventory abstractions with DB storage.
- [opentalk-jobs](crates/opentalk-jobs)
    - Job execution system for maintenance tasks such as removing old meeting information according to GDPR legislation.
- [opentalk-log](crates/opentalk-log)
    - Shared logging crate centralizing logging-related setup/utilities used across controller crates.
- [opentalk-signaling-core](crates/opentalk-signaling-core)
    - Core signaling infrastructure.
- [opentalk-test-util](crates/opentalk-test-util)
    - Shared testing support crate (controller/database fixtures and helpers, optional DB-enabled test features).

## OpenTalk Controller API Specification

The <docs/developer/api.yaml> file contains the OpenTalk API specification formalized in OpenAPI
format.

### Checking the consistency with Spectral

[Stoplight Spectral](https://stoplight.io/open-source/spectral) is a linter tool
for structured data such as JSON and YAML. It contains built-in support to
ensure the consistency of an OpenAPI specification. These checks go far beyond
what most other linters detect, resulting in significantly higher consistency of
the OpenAPI specification.

#### Running the checks locally

##### Prerequisites

The subsequent commands assume that the project root is stored in the environment variable `PROJECT_ROOT` like this:

```bash
export PROJECT_ROOT="/path/to/opentalk/controller"
```

Alternatively if the project root is the current directory:

```bash
export PROJECT_ROOT="$(pwd)"
```

##### With `spectral` installed

```bash
spectral lint --ruleset "$PROJECT_ROOT"/ci/spectral/openapi.yml "$PROJECT_ROOT"/docs/developer/api.yaml
```

##### With the `stoplight/spectral` Docker image

```bash
docker run --rm -it -v "$PROJECT_ROOT":/tmp stoplight/spectral lint --ruleset /tmp/ci/spectral/openapi.yml /tmp/api/controller/frontend_api.yaml
```
