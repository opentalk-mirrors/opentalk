# LiveKit

{{ product_name }} organizes video and audio streams through [LiveKit](https://livekit.io/).

## Configuration

The section in the [configuration file](configuration.md) is called `livekit`.

| Field         | Type     | Required | Default value | Description                                                                                                                                                                                                                         |
| ------------- | -------- | -------- | ------------- | ----------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| `api_key`     | `string` | yes      | -             | The name of the API key used by the {{ product_name }} controller to communicate with the LiveKit API.                                                                                                                              |
| `api_secret`  | `string` | yes      | -             | The API secret used by the {{ product_name }} controller to communicate with the LiveKit API.                                                                                                                                       |
| `public_url`  | `string` | yes      | -             | The URL under which the LiveKit server can be reached by the {{ product_name }} frontend. The {{ product_name }} controller will not use this URL, but just pass it on to the frontend ([see also](#using-built-in-livekit-proxy)). |
| `service_url` | `string` | yes      | -             | The URL under which the {{ product_name }} controller communicates with the LiveKit server.                                                                                                                                         |

Example:

```toml
[livekit]
api_key = "controller_key"
api_secret = "-secret-"
public_url = "https://controller-public.example.com/livekit"
service_url = "http://livekit-internal.lan"
```

- Ensure that `public_url` is reachable for all users connecting to LiveKit.
- The `service_url` should be the URL where the controller can access the LiveKit server, which can be an internally routed URL or IP.
- The `api_key` and `api_secret` must match the values configured in the LiveKit server.
- The recorder service will fetch the configuration from the controller

### Configuring LiveKit

- Use the container image `livekit/livekit-server:<version>` in combination with controller from the [corresponding release](https://docs.opentalk.eu/releases/).

- If you are using Docker, it is recommended to run LiveKit in `network_mode: host`. This is crucial because:
    - A large number of UDP ports are required for proper functionality.
    - Not using `network_mode: host` will result in a separate process being started for each mapped UDP port.

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

### Proxying LiveKit Signaling

#### Using built-in LiveKit proxy

In order to ensure that only partipants with an active controller connection can join the LiveKit room, the controller provides a proxy endpoint under `/livekit`.
Direct access to the LiveKit singnaling endpoint must be prevented (e.q. by firewall rules).

This endpoint is available since:

- {{ product_name }} 25.4.6 (controller 0.32.7)
- {{ product_name }} 25.3.3 (controller 0.31.4)

#### Reverse proxy configuration

Since the versions listed under [Using built-in LiveKit proxy](#using-built-in-livekit-proxy), LiveKit signaling should only be exposed via the controller proxy endpoint.
