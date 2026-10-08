# Services

The {{ product_name }} Orchestrator forwards requests to the service instances that registered at it, e.g. RoomServers,
recorders or transcription services. The `services` section configures the API keys that the orchestrator uses to
authorize these requests.

## Configuration

The section in the [configuration file](README.md) is called `services`.

| Field  | Type                 | Required | Default value | Description                                                  |
| ------ | -------------------- | -------- | ------------- | ------------------------------------------------------------ |
| `keys` | Key/value pair array | yes      | -             | The API keys used to authorize requests towards the services |

### Keys

When a service instance registers with the orchestrator, it sends the ids of the API keys it has configured. The
orchestrator accepts the registration only if it knows at least one of these key ids. Requests towards that instance
are then authorized with the first matching key.

The key ids and secrets must therefore match the API keys configured in the `http.api_keys` setting of the respective
RoomServer, recorder or transcription service. Use a different secret for each service in production.

An API key can be configured as string (`"<key_id>:<key_secret>"`) or as key/value pair
(`{ id = "<key_id>", secret = "<key_secret>" }`).

!!! important

    When multiple orchestrators are deployed, all of them must be configured with the same keys. Otherwise services
    cannot register with every orchestrator.

When set through the environment, the keys are provided as a comma-separated list:

```sh
OT_ORCHESTRATOR_SERVICES__KEYS="roomserver:secret1,recorder:secret2,transcription:secret3"
```

### Example

```toml
[services]
keys = [
    { id = "roomserver", secret = "secret1" },
    { id = "recorder", secret = "secret2" },
    { id = "transcription", secret = "secret3" },
]
```
