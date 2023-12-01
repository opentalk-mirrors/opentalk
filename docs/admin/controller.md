---
sidebar_position: 3
---

# Controller

## Configuration

The section in the [configuration file](configuration.md) is called `controller`.

| Field      | Type     | Required | Default value | Description                                                              |
| ---------- | -------- | -------- | ------------- | ------------------------------------------------------------------------ |
| `domain`   | `string` | yes      | -             | The host and optional port of the controller in the format `host[:port]` |
| `insecure` | `bool`   | no       | false         | `true` to disable transport security to the controller                   |

### Example

```toml
[controller]
domain = "localhost:11311"
insecure = true
```
