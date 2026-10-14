---
sidebar_position: 4
---

# Languages

The language of the emails can localized individually. If no culture is specified a fallback is used.

## Configuration

The section in the [configuration file](configuration.md) is called `languages`.

| Field              | Type     | Required | Default value | Description                                                      |
| ------------------ | -------- | -------- | ------------- | ---------------------------------------------------------------- |
| `default_language` | `string` | no       | "en-US"       | The default language to use if no explicit language is specified |

### Example

```toml
[languages]
default_language = "de-DE"
```
