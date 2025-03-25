# README

## Setup OpenTalk local Dev environment

Clone the opentalk-compose git repo and change to the root directory of the repo. 
e.g

```bash
git clone ssh://git@git.opentalk.dev:222/opentalk/opentalk-devops/opentalk-compose.git && cd opentalk-compose
```

Copy the `config/default.env.sampl` to `config/default.env`.

```bash
cp config/default.env.sample config/default.env
```

Run the cforge container once to initilaze the local dev environment.
It generates the nessecary secrets. Find them in the file `config/secrets/secrets.yaml`, after the first run.

```bash
docker compose run --rm cforge --dev init
```

Start the OpenTalk components and the depending services.

```bash
docker compose up -d
```

## Postinstall tasks

Some configurations can only be made when speciffic services are up and running.

### Create a demo User

For testing purposes there is a task `demoUser` to quckly create a test account.
With the `--diff` option the credentials are printed to stdout. The default User name is `demo`.

```bash
docker compose run --rm cforge --diff task demoUser
```

### Run a specific task

Use the `--help` option with `task` to get informations about available tasks.

```
docker compose run --rm cforge task --help
usage: cforge task [--timestamp TIMESTAMP] [--help] [task_name]

positional arguments:
  task_name             Specify a post-install task. Use 'help' to see available tasks.

options:
  --timestamp TIMESTAMP
                        Specify a timestamp (YYYYMMDDHHMMSSZ), required for restore tasks
  --help, -h            Show this help message and exit

Available Post-Install Tasks:

                        User Management                         
┏━━━━━━━━━━━┳━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━┓
┃ Task Name ┃ Description                                      ┃
┡━━━━━━━━━━━╇━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━┩
│ demoUser  │ Creates a user in Keycloak for testing purposes. │
└───────────┴──────────────────────────────────────────────────┘
                                  Backup                                   
┏━━━━━━━━━━━━━━━━━━━━━━━━━━┳━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━┓
┃ Task Name                ┃ Description                                  ┃
┡━━━━━━━━━━━━━━━━━━━━━━━━━━╇━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━┩
│ createBackup             │ Creates full backup of all services          │
│ createBackupConfig       │ Creates backup of the config directory       │
│ createBackupControllerDB │ Creates backup of the Controller database    │
│ createBackupKeycloakDB   │ Creates backup of the Keycloak database      │
│ createBackupTraefik      │ Creates backup of the Traefik data directory │
│ createBackupMinio        │ Creates backup of MinIO bucket               │
└──────────────────────────┴──────────────────────────────────────────────┘
                                                  Restore                                                  
┏━━━━━━━━━━━━━━━━━━━━━━━━━━━┳━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━┓
┃ Task Name                 ┃ Description                                                                 ┃
┡━━━━━━━━━━━━━━━━━━━━━━━━━━━╇━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━┩
│ restoreBackup             │ Restore OpenTalk from a given timestamp. A full backup archive is required. │
│ restoreBackupConfig       │ Restores the config directory from a given timestamp.                       │
│ restoreBackupControllerDB │ Restores Controller database from a given timestamp.                        │
│ restoreBackupKeycloakDB   │ Restores Keycloak database from a given timestamp.                          │
│ restoreBackupTraefik      │ Restores Traefik data directory from a given timestamp.                     │
│ restoreBackupMinio        │ Restores MinIO bucket from a given timestamp.                               │
└───────────────────────────┴─────────────────────────────────────────────────────────────────────────────┘
```

## Login to OpenTalk webapp

By default the web interface on a local dev environment is accessible at [http://localhost:8080](http://localhost:8080).

### Web Interfaces of 3rd party services

Some 3rd party servicese provides web interfaces. Traefik creates routes to these HTTP endpoints.
You can access the web interfaces behind the paths as follows.

#### Traefik dashboard

- [http://localhost:8080/adm/traefik/dashboard/](http://localhost:8080/adm/traefik/dashboard/)

#### Keycloak dashboard

- [http://localhost:8080/auth/](http://localhost:8080/auth/)

#### RabbitMQ Management Web Interface

- [http://localhost:8080/adm/rabbitmq/](http://localhost:8080/adm/rabbitmq/)

#### MinIO Web Console

- [http://localhost:8080/adm/minio/](http://localhost:8080/adm/minio/)

#### MailCrab Web Interface

- [http://localhost:8080/adm/mailcrab/](http://localhost:8080/adm/mailcrab/)


## Configure and `apply` service configurations

To configure specific services you can edit the `config/setting.yaml` or `config/settings.override.yaml` files.
Ensure the YAML structure when uncommenting or overrriding default values.

Apply config changes with

```
docker compose run --rm cforge --diff apply
```
