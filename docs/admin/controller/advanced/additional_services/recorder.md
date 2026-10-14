# Meeting Recorder

The {{ product_name }} Controller has the ability to establish communication with the
{{ product_name }} Recorder and initiate recording sessions. This interaction between the
{{ product_name }} Recorder and the {{ product_name }} Controller is facilitated through RabbitMQ.

## Configuration

To activate the recording functionality, it is necessary to configure the
`recording_task_queue` setting within the `rabbit_mq` section. Without this
configuration, the controller will not possess the capability to commence
recording sessions.

The section in the [configuration file](../../core/configuration.md) is called `rabbit_mq`.

In addition to the configuration file, Keycloak needs to be configured to allow the recorder to access meetings.

### Keycloak configuration

!!! note

    The Keycloak user interface changed in the past and because of that it's safe to assume
    that it will continue to change moving forward. Instead of screenshots we describe what needs to be
    done, and link to the Keycloak documentation where needed. These links
    reference a specific version of Keycloak. If those settings are outdated, please refer to the
    [Keycloak documentation archive](https://www.keycloak.org/documentation-archive.html)
    and find the corresponding section there.

The recorder requires access to the controller API. For that we need to create a
client inside Keycloak and configure the recorder with the client secret. The client
has to be assigned to the `opentalk-recorder` role to gain access to the controller API.

!!! warning

    The following configuration needs to be changed in the configuration file of the __recording service__.

```toml
[auth]
issuer = "http://localhost:8080/auth/realms/OPENTALK"
client_id = "Recorder"
client_secret = "the-client-secret"
```

1. Create an [OpenID Connect client](https://www.keycloak.org/docs/latest/server_admin/index.html#proc-creating-oidc-client_server_administration_guide).
   - The **Client ID** will be used in the field `auth.client_id` of the configuration field (e.g. `Recorder`).
   - Enable **Service account roles** in the [Capability Config](https://www.keycloak.org/docs/latest/server_admin/index.html#capability-config).
2. Create [client credentials](https://www.keycloak.org/docs/latest/server_admin/index.html#_client-credentials).
   - Use the Client Authenticator **Client Id and Secret** .
   - The **Client secret** will be used in the field `auth.client_secret` of the configuration field.
3. Set the correct issuer URL in `auth.issuer`
   - Replace the domain and realm placeholders with your specific values: `http://<Keycloak domain>/auth/realms/<{{ product_name }} realm>`
4. Grant the Recorder-Client access to the Controller API
   - [Create a realm role](https://www.keycloak.org/docs/latest/server_admin/index.html#proc-creating-realm-roles_server_administration_guide) with the id `opentalk-recorder`
   - Assign the role to the service account of the recorder client

### Controller configuration

!!! warning

    The following configurations needs to be changed in the configuration file of the __controller__.

#### Disabling recording capability

If you wish to disable the recording capability for the controller, simply
refrain from setting the `recording_task_queue` parameter in the configuration
file. Here's an example configuration without this setting:

```toml
[rabbit_mq]

# Other RabbitMQ settings ...

#recording_task_queue = "opentalk_recorder"
```

#### Enabling Recording Capability

To enable the recording capability for the controller, configure the
`recording_task_queue` parameter. This parameter should be set to the same value
as used for the recording process itself. Here's an example configuration that
enables recording:

```toml
[rabbit_mq]

# Other RabbitMQ settings ...

recording_task_queue = "opentalk_recorder"
```
