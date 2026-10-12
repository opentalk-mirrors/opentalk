# Recorder

The {{ product_name }} Recorder renders recordings of {{ product_name }} meetings. It uses GPUs for this and requires
scaling when a deployment reaches a certain size.

## Configuration

1. Add the `orchestrator` section to the `recorder.toml`:

   ```toml
   [orchestrator]
   # The API key of the orchestrator, one of the orchestrator's `http.api_keys`
   api_key = { id = "orchestrator", secret = "secret" }
   # The URL of the orchestrator
   url = "<orchestrator-url>"
   ```

2. Update the Controller's recorder configuration as shown in the [Controller's recorder](controller.md#recorder) section

3. Add the Recorder's API key to the `services.keys` section in the `orchestrator.toml` (See [Services](../services.md))

The Recorder will then register at the configured Orchestrator on startup. Once registered, it is part of the
Recorder cluster.
