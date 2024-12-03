# SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
#
# SPDX-License-Identifier: EUPL-1.2

FROM git.opentalk.dev:5050/opentalk/backend/containers/rust:1.83.0-bookworm AS builder

RUN apt-get update && apt-get install -y build-essential libclang-dev

WORKDIR /obelisk

COPY ./Cargo.toml ./Cargo.toml
COPY ./Cargo.lock ./Cargo.lock

# cache library build
# We need a proper main.rs to make the first cargo build run
# As we haven't copied any source files yet, this will only compile the dependencies
RUN mkdir src; \
    echo 'fn main() {println!("Hello World!");}' > src/main.rs; \
    cargo auditable build --release

# Now copy the real source over
COPY src src
COPY audio audio

# Cargo detects changes only by mtime
RUN touch src/main.rs; \
    cargo auditable build --release

FROM debian:bookworm

RUN apt-get update && apt-get upgrade -y && apt-get install libssl3 -y

WORKDIR /obelisk

COPY --from=builder /obelisk/target/release/opentalk-obelisk .

RUN apt-get install -y --no-install-recommends ca-certificates && update-ca-certificates

RUN ln -s opentalk-obelisk k3k-obelisk

ENTRYPOINT [ "./opentalk-obelisk" ]
