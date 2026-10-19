---
sidebar_position: 100
title: Migrate to LiveKit
---

# Migrate to LiveKit

A general [LiveKit configuration documentation section](../core/livekit.md) is also available, this page describes the relevant information when migrating from an installation with [Janus](https://janus.conf.meetecho.com/) to LiveKit.

Migrating from Janus to LiveKit involves several key steps to ensure a smooth transition. Follow the steps outlined below to successfully migrate your services.

1. **Shutdown and Remove Janus Resources**
    Begin by shutting down your existing Janus services and removing any associated resources to prevent conflicts during the migration process.

2. **Configure and Start LiveKit Server**
    - Use the matching container images for the release.
        - For controller version `0.27.0`, use `livekit/livekit-server:v1.8`
        - For later versions, look at the [releases page](https://docs.opentalk.eu/releases/), or use what is specified in the corresponding version tag on [ot-setup](https://gitlab.opencode.de/opentalk/ot-setup).
    - If you are using Docker, it is recommended to run LiveKit in `network_mode: host`. This is crucial because:
        - A large number of UDP ports are required for proper functionality.
        - Not using `network_mode: host` will result in a separate process being started for each mapped UDP port.
        - Ensure that the UDP port range (e.g. 20000-25000 or the range that was previously used for janus) is reachable for all users.

    - Node ip selection:
        - Add `--node-ip <ip>` to the container command to specify the IP of the host which will be advertised to clients.

          Example `command` entry in a compose file:

          ```yaml
          command: --config /livekit.yaml --node-ip 198.51.100.23
          ```

        - Alternatively, you can set `rtc.use_external_ip` to true to allow LiveKit to discover the true IP using STUN in cloud environments where the host has a public IP address but is not directly exposed.

    - Configure the access tokens used by the controller and livekit. In the livekit configuration, add an entry in the format `<NAME>: <SECRET>`, ensuring it is in sync with the controller settings for `api_key` and `api_secret`.

    Here is a sample configuration for the LiveKit server:

    ```yaml
    ---
    port: 7880
    rtc:
      tcp_port: 7881
      port_range_start: 20000
      port_range_end: 25000
      use_external_ip: true
    keys:
      controller_key: -secret-
    logging:
      json: false
      level: info
    ```

3. **Adjust Reverse Proxy Configuration**
    If you plan to host LiveKit behind a reverse proxy, ensure that:
    - The reverse proxy handles the TLS encryption.
    - Port 7880 (or the configured port) is set as the target for the reverse proxy.

4. **Adjust Controller Configuration**
    - Remove the `[room_server]` configuration from your controller settings.
    - Set up LiveKit in your controller configuration as follows:

    ```toml
    [livekit]
    api_key = "controller_key"
    api_secret = "-secret-"
    public_url = "https://livekit-public.example.com"
    service_url = "https://livekit-internal.example.com"
    ```

    - Ensure that `public_url` is reachable for all users connecting to LiveKit.
    - The `service_url` should be the URL where the controller can access the LiveKit server, which can be an internally routed URL or IP.
    - The `api_key` and `api_secret` must match the values configured in the LiveKit server.
    - The recorder service will fetch the configuration from the controller
