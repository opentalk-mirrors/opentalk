# Database (Postgres)

The {{ product_name }} Controller uses PostgreSQL database.

## Configuration

The section in the [configuration file](./configuration.md) is called `database`.

| Field             | Type     | Required | Default value | Description                                                                    |
| ----------------- | -------- | -------- | ------------- | ------------------------------------------------------------------------------ |
| `url`             | `string` | yes      | -             | Database URL connection, with specified username, password, and database name  |
| `max_connections` | `uint`   | no       | 100           | Maximum number of connection allowed to the database server                    |

### Examples

#### Default with URL

```toml
[database]
url = "postgres://postgres:password123@localhost:5432/opentalk"
```

#### With maximum connections

```toml
[database]
url = "postgres://postgres:password123@localhost:5432/opentalk"
max_connections = 50
```

## `opentalk-controller migrate-db` subcommand

This subcommand is used to migrate the database to the currently installed controller version.

The following two examples show how to manually migrate the database:

```shell
opentalk-controller migrate-db
```

Help output looks like this:

<!-- begin:fromfile:cli-usage/opentalk-controller-migrate-db-help.md -->

```text
Migrate the db. This is done automatically during start of the controller, but can be done without starting the controller using this command

Usage: opentalk-controller migrate-db

Options:
  -h, --help  Print help
```

<!-- end:fromfile:cli-usage/opentalk-controller-migrate-db-help.md -->
