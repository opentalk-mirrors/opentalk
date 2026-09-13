# Meeting Reports

The {{ product_name }} Controller uses the [typst](https://typst.app/) format
to generate reports in meetings.

Each module can define its own set of reports that it can generate. Modules that
make use of the report functionality are for example:

- [Meeting Report](../../developer/signaling/meeting_report.md)
- [Legal Vote](../../developer/signaling/legalvote.md)
- [Training Participation Report](../../developer/signaling/training_participation_report.md)

The section in the [configuration file](configuration.md) is called `reports`.

## Typst

The {{ product_name }} Controller uses the [typst](https://typst.app/) format for report generation. The section in the [configuration file](configuration.md) is a subsection of `reports` and is called `typst`.

| Field           | Type   | Required | Default value               | Description                                  |
| --------------- | ------ | -------- | --------------------------- | -------------------------------------------- |
| `packages_path` | `Path` | no       | `/usr/share/typst/packages` | The location where typst looks for packages. |

### Typst Packages

Typst can be extended with packages, usually obtained through [universe](https://typst.app/universe/). The Controller templates require the [`@preview/linguify`](https://typst.app/universe/package/linguify) package for localized report generation. The container images already include the package at `/usr/share/typst/packages`.

## Localization

When generating a report in the Controller, the language for the report is determined in the following order:

1. The language configured by the room owner
2. The `user_language` configured in the [`defaults`](../advanced/defaults.md) section
3. English
