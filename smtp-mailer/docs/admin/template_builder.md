---
sidebar_position: 7
---

# Template builder

The emails contain some individual links whose destination URLs are generated using the templates listed below.
The `{base_url}` and `{*_id}` placeholders are replaced by the current values.

## Configuration

The section in the [configuration file](configuration.md) is called `template_builder`.

| Field                          | Type     | Required | Default value                              | Description                                                    |
| ------------------------------ | -------- | -------- | ------------------------------------------ | -------------------------------------------------------------- |
| `join_link_builder`            | `string` | no       | `{base_url}/room/{room_id}`                | The template used to generate room links                       |
| `dashboard_event_link_builder` | `string` | no       | `{base_url}/dashboard/meetings/{event_id}` | The template used to generate links to events in the dashboard |

### Example

```toml
[template_builder]
join_link_builder = "{base_url}/room/{room_id}"
dashboard_event_link_builder = "{base_url}/dashboard/meetings/{event_id}"
```
