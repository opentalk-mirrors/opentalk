# SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
#
# SPDX-License-Identifier: EUPL-1.2

FROM git.opentalk.dev:5050/opentalk/backend/containers/rust:alpine3.19 AS Builder

RUN apk add --no-cache glib-dev openssl-dev gstreamer-dev gst-plugins-base-dev gst-plugins-bad-dev

WORKDIR /obelisk

COPY ./Cargo.toml ./Cargo.toml
COPY ./Cargo.lock ./Cargo.lock
COPY recorder recorder

# cache library build
# We need a proper main.rs to make the first cargo build run
# As we haven't copied any source files yet, this will only compile the dependencies
RUN mkdir src; \
    echo 'fn main() {println!("Hello World!");}' > src/main.rs; \
    RUSTFLAGS=-Ctarget-feature=-crt-static cargo build --release

# Now copy the real source over
COPY src src
COPY audio audio

# Cargo detects changes only by mtime
RUN touch src/main.rs; \
    RUSTFLAGS=-Ctarget-feature=-crt-static cargo build --release

FROM alpine:3.19

RUN apk add --no-cache glib libssl3 libcrypto3 \
    gstreamer gst-plugins-base gst-plugins-good gst-plugins-bad libnice-gstreamer gst-libav \
    openh264 ca-certificates

WORKDIR /obelisk

COPY --from=Builder /obelisk/target/release/opentalk-obelisk .
RUN ln -s opentalk-obelisk k3k-obelisk

ENTRYPOINT [ "./opentalk-obelisk" ]
