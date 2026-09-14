# OpenTalk Obelisk

SIP bridge for the OpenTalk conference system.

## Setup guide (local development)

### 1. Prerequisites

- A Rust toolchain (install via [rustup](https://rustup.rs/)).
- A C/C++ compiler and `libclang`. These are required because the
  `webrtc-sys` and `openh264-sys2` crates compile native code and generate
  bindings with `bindgen`.
- OpenSSL development headers and `pkg-config`.

Install the system dependencies:

```bash
# Fedora
sudo dnf install gcc gcc-c++ clang clang-devel pkgconf-pkg-config openssl-devel zlib-devel

# Debian / Ubuntu
sudo apt-get install build-essential libclang-dev pkg-config libssl-dev zlib1g-dev
```

### 2. Build

From the repository root:

```bash
cargo build -p opentalk-obelisk
```

The resulting binary is written to `target/debug/opentalk-obelisk`
(or `target/release/opentalk-obelisk` when building with `--release`).

### 3. Configure

Obelisk reads its configuration from a TOML file. Copy the bundled example and
adjust it to your environment:

```bash
cp obelisk/extra/example.toml obelisk.toml
```

At minimum you need to set the `[controller]` URL/API key and the `[sip]`
credentials. Obelisk looks for `obelisk.toml` in the current directory (see
[docs/admin/configuration.md](docs/admin/configuration.md) for all locations and
the full list of options). Individual values can be overridden with environment
variables, e.g.:

```bash
OPENTALK_OBLSK_CONTROLLER__URL=http://localhost:8000
```

### 4. Run

```bash
cargo run -p opentalk-obelisk
```

## Build the container image

The `Dockerfile` is located in `ci/Dockerfile`.

To build the image, execute in the root of the repository:

```bash
 docker build -f ci/Dockerfile . --tag <your tag>
```
