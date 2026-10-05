<!--
SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>

SPDX-License-Identifier: EUPL-1.2
-->

# Upgrades and Migrations

This chapter describes how to keep an OpenTalk Compose installation up to date and how to migrate data when a release or a configuration change requires it. Read it before every upgrade.

## 1. Updating OpenTalk Compose

Keeping your OpenTalk Compose installation current ensures you receive security fixes, performance improvements, and new features. Updates are designed to be safe and idempotent. The recommended workflow mirrors the initial installation and consists of these main steps: create a full backup, update the package from the repository, run any migration the release requires, and redeploy the stack.

> **Maintenance Window:** Perform updates during a low-usage window to avoid interrupting active meetings. A typical update (package + redeploy) usually completes within a few minutes.

### 1.1 Overview

High-level process:

1. Create a full backup (databases, config, volumes)
2. Refresh repository metadata and upgrade the `opentalk-compose` package
3. Check for package-provided config update files and merge new options
4. Check with `otctl migrate status` whether the release needs a migration, and run it
5. Run `otctl deploy` to render any updated templates and restart services if needed

### 1.2 Step 1: Create a Full Backup

Use the bundled backup facility before every update:

```bash
otctl backup all
otctl backup ls   # (optional) verify backup presence
```

Backups are stored under `/var/opt/opentalk-compose/backups`. Keep sufficient disk space free. For more granular commands see [Backups](1-quickstart-installation.md#26-backups-optional-but-recommended).

`otctl backup all` includes SQL dumps of the controller and Keycloak databases. Use the [restore workflow](1-quickstart-installation.md#27-restoring-backups) for rollback planning.

### 1.3 Step 2: Update Repository & Package

Ensure your repository configuration is still present (see [Add the Package Repository](1-quickstart-installation.md#21-add-the-package-repository-and-install-opentalk-compose)). Then update according to your distribution family.

#### 1.3.1 Debian / Ubuntu

```bash
apt update
apt install --only-upgrade opentalk-compose
```

#### 1.3.2 RHEL-based

```bash
dnf makecache
dnf upgrade -y opentalk-compose
```

#### 1.3.3 SUSE-based

```bash
zypper refresh
zypper update -y opentalk-compose
```

### 1.4 Step 3: Merge new configuration options from package updates

After package upgrade, check whether the package manager kept your existing settings file and wrote an updated template alongside it:

```bash
ls -l /etc/opentalk-compose/settings.yml.dpkg-dist /etc/opentalk-compose/settings.yml.rpmnew
```

If one of these files exists, compare it with your active configuration and merge relevant new options into `/etc/opentalk-compose/settings.yml`.

- Debian/Ubuntu commonly uses `/etc/opentalk-compose/settings.yml.dpkg-dist`
- RPM-based systems commonly use `/etc/opentalk-compose/settings.yml.rpmnew`

Pay special attention to newly introduced storage options. If you are moving between S3 backends (MinIO/Garage), follow the [migration workflow](#3-migrating-between-minio-and-garage) before or during the upgrade window.

### 1.5 Step 4: Run Required Migrations

Some releases change how data is stored, for example a new PostgreSQL major version. Check whether the installed release needs a migration:

```bash
otctl migrate status
```

If a migration is listed, run the command it names before you deploy. [Migrations](#2-migrations) explains how they work. The package upgrade also prints a reminder to run `otctl migrate status`.

### 1.6 Step 5: Redeploy Services

After upgrading the package, redeploy to apply any new templates, configuration changes, or image updates:

```bash
otctl deploy
```

The deploy is idempotent: unchanged services remain running; changed components are recreated or restarted automatically.

### 1.7 Verification & Rollback Notes

Verify versions and health:

```bash
otctl version
otctl status
```

Check logs of critical services (e.g. controller, keycloak, livekit-server) for startup issues:

```bash
otctl logs controller
otctl logs livekit-server
otctl logs keycloak
```

If problems occur:

- Re-run deploy: `otctl deploy`
- Compare configs: use previous config backup snapshot
- Restore from backup using the artifacts created in Step 1

> **Tip:** Maintain at least the last two successful full backups to facilitate rollback in rare edge cases.

## 2. Migrations

Some changes cannot be applied by `otctl deploy` alone because existing data has to be moved or converted. Each of these is handled by a dedicated `otctl migrate` command. The command checks the preconditions, copies the data, verifies the result and keeps the source data until you remove it yourself.

You do not have to remember when a migration is needed:

- `otctl migrate status` inspects the deployment (Docker volumes and `settings.yml`) and lists every migration that is still pending, with the reason and the command to run.
- `otctl deploy` performs the same check first. While a migration is pending it stops before it changes anything and prints the same list. This prevents a deploy from starting a service on empty storage next to your existing data.

Every migration follows the same pattern:

1. It shows a plan: what is copied, which services are stopped, and what happens afterwards.
2. It asks for confirmation. Type `migrate` to start, or pass `--yes` in scripts and other non-interactive sessions.
3. It copies and verifies the data while the services that write to it are stopped.
4. It switches the deployment to the new data and runs `otctl deploy`.
5. It keeps the old data, so you can roll back. Remove it yourself once you have verified the result.

Show all migration commands:

```bash
otctl migrate --help
```

> **Note:** If you have checked the state yourself and must deploy anyway, `otctl deploy -e migration_check_enabled=false` skips the check. The deploy may then start services on empty storage.

| Migration | Command | When it is needed |
| --- | --- | --- |
| [PostgreSQL major upgrade](#4-upgrading-postgresql-to-a-new-major-version) | `otctl migrate postgres` | A release ships a new PostgreSQL major version, for example 26.3 with PostgreSQL 18 |
| [MinIO ↔ Garage](#3-migrating-between-minio-and-garage) | `otctl migrate minio-to-garage`, `otctl migrate garage-to-minio` | You switch the S3 storage provider |

> **Important:** Create a full backup before every migration, as described in [Create a Full Backup](#12-step-1-create-a-full-backup).

## 3. Migrating between MinIO and Garage

You can migrate S3 data in either direction — MinIO → Garage or Garage → MinIO — using the `otctl migrate` command. The command automates the full procedure: it starts the destination, installs `rclone` if needed, copies all objects between the two backends, updates `settings.yml`, and runs a deployment to stop the source service.

> **Important:** Back up your data before starting the migration.
>
> ```bash
> otctl backup all
> ```

### 3.1 Run the migration

```bash
# Migrate from MinIO to Garage:
otctl migrate minio-to-garage

# Migrate from Garage to MinIO:
otctl migrate garage-to-minio
```

The command will:

1. Show the plan and ask for confirmation.
2. Run a deploy that starts the destination next to the source. You do not have to enable it in `settings.yml` first.
3. Install `rclone` on the host if not already present (via `apt`, `dnf` or `zypper`).
4. Verify that both containers are running.
5. Stop the controller, so that no file is uploaded while the objects are copied.
6. Copy all objects from the source bucket to the destination bucket and verify that the object counts match.
7. Update `settings.yml`: set `s3_storage_provider` and `compose_service_<destination>_enabled: true` for the destination, and `compose_service_<source>_enabled: false`.
8. Run `otctl deploy` to stop the source service and start the controller against the new backend.

> **Downtime:** OpenTalk is unavailable from step 5 until the deploy in step 8 has finished. Running meetings are interrupted, so plan the migration for a maintenance window.

If a step fails, the command restarts the controller on the unchanged source and leaves `settings.yml` untouched. Fix the cause and run the command again.

To skip the automatic deploy after migration (e.g. to verify manually first):

```bash
otctl migrate minio-to-garage --no-deploy
```

The controller then stays stopped until you run `otctl deploy`. Starting it earlier would make it write to the old backend again.

### 3.2 Verify

After migration, confirm the destination service is healthy and the controller is using the new backend:

```bash
# For Garage as destination:
otctl compose exec garage /garage bucket info opentalk

# For MinIO as destination:
otctl compose exec minio mc ls local/opentalk
```

### 3.3 Clean up (optional)

Once you have verified the migration, remove the old docker volumes to free disk space:

```bash
# After migrating away from MinIO:
otctl volumes rm opentalk_minio

# After migrating away from Garage:
otctl volumes rm opentalk_garage_data opentalk_garage_meta
```

## 4. Upgrading PostgreSQL to a new major version

The controller and Keycloak each keep their data in a PostgreSQL database. A new PostgreSQL major version cannot read the data files of the previous one, so a release that moves to a new major needs a migration. OpenTalk Compose 26.3 moves from PostgreSQL 17 to 18.

From PostgreSQL 18 on, each major version gets its own Docker volume, for example `opentalk_controller-db-pg18`. The migration copies the data into the new volume and leaves the old one untouched. Up to PostgreSQL 17 the volumes are called `opentalk_controller-db` and `opentalk_keycloak-db`.

> **Downtime:** The controller and Keycloak are stopped during the migration, so OpenTalk is unavailable. How long it takes depends on the size of the databases. Plan it for a maintenance window.

### 4.1 Detect the migration

After the package upgrade, `otctl migrate status` lists the databases that still need the migration:

```text
2 migration(s) pending. 'otctl deploy' stays blocked until they are done.

* PostgreSQL 18 upgrade of the controller database
  Its data is still in the volume opentalk_controller-db, written by an older PostgreSQL major. PostgreSQL 18 cannot start on it.
  Next step: Run 'otctl migrate postgres'.
```

Until the migration is done, `otctl deploy` stops with the same message. Your installation keeps running on the old version in the meantime.

### 4.2 Run the migration

```bash
otctl backup all
otctl migrate postgres
```

The command first checks and shows a plan, then asks you to type `migrate`. The checks confirm that each old volume was written by the PostgreSQL version this release migrates from, that the database password files exist, and that there is enough free disk space. The new volumes and the SQL dumps each need about as much space as the old data, twice that if both are on the same filesystem.

After the confirmation, for each database the command:

1. Stops the application, then its database.
2. Starts the old PostgreSQL version on the old volume, without network access, counts the rows of every table and writes an SQL dump to `/var/opt/opentalk-compose/backups/dumps/pg-migrate-<database>-17-to-18-<timestamp>.sql`.
3. Initialises the new PostgreSQL version on a new volume with the existing user, database name and password, and restores the dump.
4. Compares the row count of every table with the old database and refreshes the planner statistics.

Finally it runs `otctl deploy`, which starts both databases on the new volumes and the services on top of them.

If any step fails, the command removes the new volumes, starts the previous containers again and stops with the reason. OpenTalk then runs as before on the old data, and you can run the command again once the cause is fixed.

Options:

- `--yes` skips the confirmation, for scripts and other non-interactive sessions.
- `--no-deploy` skips the final deploy. The services then stay stopped until you run `otctl deploy`.

### 4.3 Verify

```bash
otctl status
otctl migrate status
```

Log in to OpenTalk and open an existing meeting room to confirm that the data is there. `otctl migrate status` reports no pending migration and lists the old volumes it kept.

### 4.4 Roll back

The old volumes stay untouched, so you can go back to the previous release:

1. Reinstall the previous `opentalk-compose` package.
2. Run `otctl deploy`. It uses the old volumes again.

> **Note:** A rollback returns to the data as it was at the moment of the migration. Anything changed after that, such as new meetings or users, exists only in the new volumes. To keep it, restore a database dump made after the migration (see the [restore workflow](1-quickstart-installation.md#27-restoring-backups)).

### 4.5 Clean up

Once you are sure you will not roll back, remove the old volumes to free disk space:

```bash
otctl volumes rm opentalk_controller-db opentalk_keycloak-db
```

The SQL dumps of the migration stay in `/var/opt/opentalk-compose/backups/dumps` until you delete them.

