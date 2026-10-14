---
sidebar_position: 6
---

# Support contact

The emails optionally can contain information about a support contact.

## Configuration

The section in the [configuration file](configuration.md) is called `support_contact`.

| Field                  | Type     | Required  | Default value | Description                              |
| ---------------------- | -------- | --------- | ------------- | ---------------------------------------- |
| `phone`                | `string` | See below | -             | The phone number of the support contact  |
| `mail`                 | `string` | See below | -             | The email address of the support contact |

The `support_contact` section can be omitted as a whole. If it exists, its `phone` and `mail` fields must exist as well.

### Example

```toml
[support_contact]
phone = "+49123321123"
mail = "support@example.org"
```
