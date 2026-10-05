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

If the release includes a PostgreSQL major upgrade (for example controller-db/keycloak-db image major bump), create dedicated database dumps before deploying:

```bash
otctl backup dump controller
otctl backup dump keycloak
```

Use the [restore workflow](1-quickstart-installation.md#27-restoring-backups) for rollback planning.

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
| [MinIO ↔ Garage](#3-migrating-between-minio-and-garage) | `otctl migrate minio-to-garage`, `otctl migrate garage-to-minio` | You switch the S3 storage provider |

> **Important:** Create a full backup before every migration, as described in [Create a Full Backup](#12-step-1-create-a-full-backup).

## 3. Migrating between MinIO and Garage

You can migrate S3 data in either direction — MinIO → Garage or Garage → MinIO — using the `otctl migrate` command. The command automates the full procedure: it installs `rclone` if needed, copies all objects between the two backends, updates `settings.yml`, and optionally runs a deployment to stop the source service.

> **Important:** Back up your data before starting the migration.
>
> ```bash
> otctl backup all
> ```

### 3.1 Enable both providers in parallel

For a successful migration, both S3 providers must be running at the same time. Add the following to `/etc/opentalk-compose/settings.yml` and deploy:

**MinIO → Garage** (Garage is not yet running):

```yaml
compose_service_minio_enabled: true
compose_service_garage_enabled: true
```

**Garage → MinIO** (MinIO is not yet running):

```yaml
compose_service_garage_enabled: true
compose_service_minio_enabled: true
```

Then run `otctl deploy` to start both services. A warning will appear at the end of the deployment indicating that both providers are active and a migration should be performed.

> **Note:** `s3_storage_provider` controls which S3 endpoint the controller uses. Leave it pointing at the current active provider until the migration is complete.

### 3.2 Run the migration

```bash
# Migrate from MinIO to Garage:
otctl migrate minio-to-garage

# Migrate from Garage to MinIO:
otctl migrate garage-to-minio
```

The command will:

1. Install `rclone` on the host if not already present (via `apt`, `dnf` or `zypper`).
2. Verify that both containers are running and healthy.
3. Stop the controller, so that no file is uploaded while the objects are copied.
4. Copy all objects from the source bucket to the destination bucket.
5. Verify that the object counts match.
6. Update `settings.yml`: set `s3_storage_provider` to the destination and set `compose_service_<source>_enabled: false`.
7. Run `otctl deploy` automatically to stop the source service and start the controller against the new backend.

> **Downtime:** OpenTalk is unavailable from step 3 until the deploy in step 7 has finished. Running meetings are interrupted, so plan the migration for a maintenance window.

If a step fails, the command restarts the controller on the unchanged source and leaves `settings.yml` untouched. Fix the cause and run the command again.

To skip the automatic deploy after migration (e.g. to verify manually first):

```bash
otctl migrate minio-to-garage --no-deploy
```

The controller then stays stopped until you run `otctl deploy`. Starting it earlier would make it write to the old backend again.

### 3.3 Verify

After migration, confirm the destination service is healthy and the controller is using the new backend:

```bash
# For Garage as destination:
otctl compose exec garage /garage bucket info opentalk

# For MinIO as destination:
otctl compose exec minio mc ls local/opentalk
```

### 3.4 Clean up (optional)

Once you have verified the migration, remove the old docker volumes to free disk space:

```bash
# After migrating away from MinIO:
otctl volumes rm opentalk_minio

# After migrating away from Garage:
otctl volumes rm opentalk_garage_data opentalk_garage_meta
```
