---
sidebar_position: 3
---

# Frontend

The emails contain some links referring to specific parts of the frontend.

## Configuration

The section in the [configuration file](configuration.md) is called `frontend`.

| Field                 | Type     | Required | Default value                                   | Description                                                     |
| --------------------- | -------- | -------- | ----------------------------------------------- | --------------------------------------------------------------- |
| `base_url`            | `string` | no       | `"https://opentalk.example.org"`                | The base URL of the frontend.                                   |
| `data_protection_url` | `string` | no       | `"https://opentalk.example.org/dataprotection"` | The URL referring to the data protection hints                  |
| `quick_guide_urls`    | `table`  | no       | See below                                       | The URLs of the quick guide, keyed by the language of the email |

### Quick guide URLs

The emails contain a hint linking to the OpenTalk quick guide. The link can be configured per email language in the
`[frontend.quick_guide_urls]` table. The keys are the languages supported by the emails (`en-US` and `de-DE`), the
values are the URLs. Languages that are not configured use the default URL:

| Language | Default value                            |
| -------- | ---------------------------------------- |
| `en-US`  | `"https://opentalk.eu/en/quick-manual"`  |
| `de-DE`  | `"https://opentalk.eu/de/kurzanleitung"` |

### Example

```toml
[frontend]
base_url = "https://video.example.org"
data_protection_url = "https://video.example.org/dataprotection"

[frontend.quick_guide_urls]
"en-US" = "https://video.example.org/en/quick-guide"
"de-DE" = "https://video.example.org/de/kurzanleitung"
```
