# Transcription

Configuration of the transcription engine itself.

## Configuration

The section in the [configuration file](README.md) is called `transcription`.

| Field      | Type     | Required | Default value                     | Description                            |
| ---------- | -------- | -------- | --------------------------------- | -------------------------------------- |
| `model`    | `string` | no       | `models/ggml-large-v3-turbo.bin`  | Path to the whisper.cpp `ggml` model.  |
| `dump_dir` | `string` | no       | -                                 | Directory to write recognized speech segments as WAV files together with their transcribed text, for debugging. Disabled when absent. |

The model file must be a whisper.cpp `ggml` model. See
[Hardware Acceleration](../hardware_acceleration.md) for how the model is
executed on different backends.

### Example

```toml
[transcription]
model = "models/ggml-large-v3-turbo.bin"
```
