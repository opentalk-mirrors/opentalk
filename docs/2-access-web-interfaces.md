# Access Web Interfaces

## 1. Table of Contents

- 1 [Access Web Interfaces](#1-access-web-interfaces)
- 2 [Login to OpenTalk Webapp](#2-login-to-opentalk-webapp)
- 3 [Web Interfaces of 3rd Party Services](#3-web-interfaces-of-3rd-party-services)
	- 3.1 [Keycloak Admin Console](#31-keycloak-admin-console)
	- 3.2 [Authentication for Endpoints Behind /adm/](#32-authentication-for-endpoints-behind-adm)
		- 3.2.1 [Traefik Dashboard](#321-traefik-dashboard)
		- 3.2.2 [RabbitMQ Management Web Interface](#322-rabbitmq-management-web-interface)
		- 3.2.3 [MinIO Web Console](#323-minio-web-console)
		- 3.2.4 [MailCrab Web Interface](#324-mailcrab-web-interface)

The OpenTalk web application provides a dashboard for managing OpenTalk meetings. In addition, there are several other web interfaces and consoles from third-party components that become accessible after installation and help administer the OpenTalk environment. The following section describes which interfaces are available and how to access them when using the default settings.

## 2. Login to OpenTalk Webapp

- e.g. `https://example.com`

The OpenTalk web application is the main component for users to access and use OpenTalk. For more information, refer to the [OpenTalk User documentation](https://docs.opentalk.eu/user/). If the default settings are kept, the web application is always available at the domain configured in the `OT_DOMAIN` environment variable.

## 3. Web Interfaces of 3rd Party Services

Some third-party services provide web interfaces for administration tasks or to display the current status of the service. With the default settings, Traefik is configured to create HTTP routes to the respective web interfaces. You can access the web interfaces behind the paths as follows.

### 3.1 Keycloak Admin Console

Accessible with default settings at: `/auth/`

- e.g. `http://example.com/auth/`

When default settings are kept the username is `admin`. The password is randomly generated during the initialization phase and can be found in `config/secrets/secrets.yaml` as the value for the key `KC_BOOTSTRAP_ADMIN_PASSWORD`.

Here you can create users for using OpenTalk. A step-by-step guide can be found [here](docs/3-managing-opentalk-users.md). Extensive configurations can also be made regarding user authentication. Further information on using the Keycloak Admin Console can be found in the official [Keycloak documentation](https://www.keycloak.org/docs/latest/server_admin/index.html#using-the-admin-console).


### 3.2 Authentication for Endpoints Behind `/adm/`

To protect certain services from unauthorized access, authentication is required. If the default settings are kept, the username is `admin`. The password is randomly generated during the initialization phase and can be found in `config/secrets/secrets.yaml` as the value for the key `TRAEFIK_DASHBOARD_ADMIN_PASSWORD`.
It is important not to forget the trailing slash `/` at the end of the URL to ensure proper routing.

#### 3.2.1 Traefik Dashboard

Accessible with default settings at: `/adm/traefik/dashboard/`

- e.g. `http://example.com/adm/traefik/dashboard/`

The Traefik dashboard primarily displays the status of routes for the respective services. For more information, refer to the official [Traefik documentation](https://doc.traefik.io/traefik/operations/dashboard/).

#### 3.2.2 RabbitMQ Management Web Interface

Accessible with default settings at: `/adm/rabbitmq/`

- e.g. `http://example.com/adm/rabbitmq/`

RabbitMQ’s management UI shows connections, channels, queues and consumers; use it to verify OpenTalk queues (`opentalk_mailer`, `opentalk_recorder`) and consumer health.

Login: pass Traefik BasicAuth (see 3.2), then sign in to RabbitMQ with user `ot` and the password from `RABBITMQ_PASSWORD` (configured via settings/secrets).

Find more information in the [RabbitMQ documentation](https://www.rabbitmq.com/docs/management).

#### 3.2.3 MinIO Web Console

Accessible with default settings at: `/adm/minio/`

- e.g. `http://example.com/adm/minio/`

MinIO Console to manage buckets, objects, policies and access keys used by OpenTalk (e.g., recordings, PDF documents).

Login: pass Traefik BasicAuth (see 3.2), then use MinIO credentials — username `minioadmin` (unless changed) and the password from `MINIO_ROOT_PASSWORD` (stored in secrets).

Find more information in the [MinIO documentation](https://docs.min.io/community/minio-object-store/administration/minio-console.html)

#### 3.2.4 Optional: MailCrab Web Interface

Accessible with default settings at: `/adm/mailcrab/`

- e.g. `http://example.com/adm/mailcrab/`

MailCrab is an optional dev/test SMTP sink to preview invites and notifications without sending real emails.
The service is not enabled by defualt and intendet to use in `dev` mode.

The MailCrab project is located [here](https://github.com/tweedegolf/mailcrab).
