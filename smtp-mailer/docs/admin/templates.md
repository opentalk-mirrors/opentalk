---
sidebar_position: 8
---

# Templates

The emails are generated from templates. For each kind of email, there exists one html template and one plaintext template. These templates can be replaced during runtime, before starting the SMTP mailer.

## Configuration

The section in the [configuration file](configuration.md) is called `templates`.

| Field                             | Type     | Required | Default value | Description                                                             |
| --------------------------------- | -------- | -------- | ------------- | ----------------------------------------------------------------------- |
| `registered_invite`               | `string` | no       | See below     | The template path setting for invites to registered users               |
| `unregistered_invite`             | `string` | no       | See below     | The template path setting for invites to unregistered users             |
| `external_invite`                 | `string` | no       | See below     | The template path setting for invites to external users                 |
| `registered_event_update`         | `string` | no       | See below     | The template path setting for event updates to registered users         |
| `unregistered_event_update`       | `string` | no       | See below     | The template path setting for event updates to unregistered users       |
| `external_event_update`           | `string` | no       | See below     | The template path setting for event updates to external users           |
| `registered_event_cancellation`   | `string` | no       | See below     | The template path setting for event cancellations to registered users   |
| `unregistered_event_cancellation` | `string` | no       | See below     | The template path setting for event cancellations to unregistered users |
| `external_event_cancellation`     | `string` | no       | See below     | The template path setting for event cancellations to external users     |
| `registered_uninvite`             | `string` | no       | See below     | The template path setting for uninvites to registered users             |
| `unregistered_uninvite`           | `string` | no       | See below     | The template path setting for uninvites to unregistered users           |
| `external_uninvite`               | `string` | no       | See below     | The template path setting for uninvites to external users               |

 To specify the templates, their set of paths must be provided in a certain format. The default template path settings accord with this pattern:

- `{html = "resources/templates/{field}.html", txt = "resources/templates/{field}.txt"}`

### Example

```toml
[templates]
registered_invite = "{html = "resources/templates/registered_invite.html", txt = "resources/templates/registered_invite.txt"}"
unregistered_invite = "{html = "resources/templates/unregistered_invite.html", txt = "resources/templates/unregistered_invite.txt"}"
external_invite = "{html = "resources/templates/external_invite.html", txt = "resources/templates/external_invite.txt"}"
<more lines>
```
