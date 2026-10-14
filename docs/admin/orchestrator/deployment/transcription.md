# Transcription scaling

The {{ product_name }} Transcription service transcribes spoken word for live subtitles. It uses GPUs for this and
requires scaling when a deployment reaches a certain size.

## Configuration

1. Add the `orchestrator` section to the `transcription.toml`:

   ```toml
   [orchestrator]
   # The API key of the orchestrator, one of the orchestrator's `http.api_keys`
   api_key = { id = "orchestrator", secret = "secret" }
   # The URL of the orchestrator
   url = "<orchestrator-url>"
   ```

2. Update the Controller's transcription configuration as shown in the [Controller's transcription](controller.md#transcription) section

3. Add the transcription API key to the `services.keys` section in the `orchestrator.toml` (See [Services](../services.md))

The transcription service will then register at the configured orchestrator on startup. Once registered, it is part of
the transcription cluster.
