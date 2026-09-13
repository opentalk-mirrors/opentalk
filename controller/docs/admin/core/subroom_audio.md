# Subroom Audio

The Subroom Audio module allows participants to select other participants as their whisper partner. They can then talk
to each other without the rest of the conference hearing them.

The Subroom Audio module is disabled by default.

## Configuration

| Field            | Type   | Required | Default value | Description                               |
| ---------------- | ------ | -------- | ------------- | ----------------------------------------- |
| `enable_whisper` | `bool` | no       | false         | Enables the Subroom Audio whisper feature |

### Examples

#### Default Setup

```toml
[subroom_audio]
enable_whisper = false
```
