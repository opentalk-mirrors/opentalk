---
sidebar_position: 3
---

# Frontend

The emails contain some links referring to specific parts of the frontend.

## Configuration

The section in the [configuration file](configuration.md) is called `frontend`.

| Field                 | Type     | Required | Default value                                   | Description                                    |
| --------------------- | -------- | -------- | ----------------------------------------------- | ---------------------------------------------- |
| `base_url`            | `string` | no       | `"https://opentalk.example.org"`                | The base URL of the frontend.                  |
| `data_protection_url` | `string` | no       | `"https://opentalk.example.org/dataprotection"` | The URL referring to the data protection hints |

### Example

```toml
[frontend]
base_url = "https://video.example.org"
data_protection_url = "https://video.example.org/dataprotection"
```
