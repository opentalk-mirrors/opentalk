# Quickstart Guide to Self-Host OpenTalk

OpenTalk Compose is a set of tools and utilities for running the OpenTalk video conferencing solution on your own server. This quickstart guide covers the minimal installation steps to get the necessary services up and running. It assumes a standard scenario with a publicly accessible server where all components run on a single host, and SSL/TLS certificates are obtained via Let's Encrypt.
For customized setups or special conditions, refer to the "OpenTalk Administration Guide" in the documentation to find options that suit your requirements. The essentials are explained in this Quickstart Guide.

## Table of Contents

- [1. Preparations](#1-preparations)
	- [1.1 Minimal System Requirements](#11-minimal-system-requirements)
	- [1.2 Supported Operating Systems](#12-supported-operating-systems)
	- [1.3 Firewall Configuration](#13-firewall-configuration)
	- [1.4 DNS Domain Record](#14-dns-domain-record)
	- [1.5 Docker as Container Engine is Required](#15-docker-as-container-engine-is-required)
- [2. Installation Procedure](#2-installation-procedure)
	- [2.1 Add the OpenTalk Package Repository](#21-add-the-opentalk-package-repository)
	- [2.2 Install the opentalk-compose Package](#22-install-the-opentalk-compose-package)
	- [2.3 Set the Server's DNS Domain Record](#23-set-the-servers-dns-domain-record)
	- [2.4 Initialize the Deployment Environment](#24-initialize-the-deployment-environment)
	- [2.5 Optional: Edit Settings in settings.yml](#25-optional-edit-settings-in-settingsyml)
	- [2.6 Run OpenTalk Application Stack](#26-run-opentalk-application-stack)
	- [2.8 Optional: Enable Demo User](#28-optional-enable-demo-user)

## 1. Preparations

### 1.1 Minimal System Requirements

The target server for OpenTalk installation should fulfill the following minimum hardware specifications:

- **CPU:** 2 cores
- **RAM:** 4 GB
- **Storage:** 20 GB HDD
- **Network:** a public network interface
- **DNS:** a domain record pointing to the public network interface

### 1.2 Supported Operating Systems

Currently, releases are provided as DEB and RPM packages. The packages are neither architecture-dependent nor tied to a specific Linux distribution.

We recommend using one of the following distributions:

| Distributions using `.deb`     | Distributions using `.rpm`      |
|--------------------------------|--------------------------------------|
| Debian 12 "Bookworm"           | RHEL 9                               |
| Debian 13 "Trixie"             | RHEL 10                              |
| Ubuntu 22.04 LTS "Jammy Jellyfish" | AlmaLinux 9                     |
| Ubuntu 24.04 LTS "Noble Numbat"    | AlmaLinux 10                    |
|                                | Rocky Linux 9                        |
|                                | Rocky Linux 10                       |
|                                | openSUSE Leap 15                   |
|                                | SLES 15                           |

### 1.3 Firewall Configuration

The following firewall ports should be accessible for the standard deployment scenario. Installation scenarios behind restrictive networks are discussed elsewhere.

- **80, 443/tcp** (HTTP, HTTPS)
- **40000-49999/udp** (RTP Obelisk)
- **5060/udp** (SIP Obelisk)
- **50000-64535/udp** (RTP LiveKit)
- **7881/tcp** (WebRTC ICE over TCP, LiveKit)
- **3478/udp** (LiveKit TURN)

### 1.4 DNS Domain Record

Make sure that an appropriate DNS domain record for OpenTalk is configured and points to the public IPv4 and IPv6 addresses of your server.
It is the domain name used to access the OpenTalk web application, e.g., `https://example.com`.

### 1.5 Docker as Container Engine is Required

Docker and the Docker Compose plugin must be installed before you start with the `opentalk-compose` installation. If this is not yet the case, follow the [official Docker installation documentation](https://docs.docker.com/engine/install/debian/#install-using-the-repository).

## 2. Installation Procedure

### 2.1 Add the OpenTalk Package Repository

**Debian and Ubuntu-based Distributions:**

```bash
apt update && apt install -y curl gnupg
curl http://anfi.opentalk.run:7376/deb-testing/public.gpg | gpg --yes --dearmor -o /usr/share/keyrings/openrepo-deb-testing.gpg
echo "deb [arch=any signed-by=/usr/share/keyrings/openrepo-deb-testing.gpg] http://anfi.opentalk.run:7376/deb-testing/ stable main" > /etc/apt/sources.list.d/openrepo-deb-testing.list
```

**RHEL and SuSE-based Distributions:**

```bash
echo """
[rpm-testing]
name=rpm-testing
baseurl=http://anfi.opentalk.run:7376/rpm-testing
enabled=1
repo_gpgcheck=1
gpgkey=http://anfi.opentalk.run:7376/rpm-testing/public.gpg
""" > /etc/yum.repos.d/rpm-testing.repo
```

### 2.2 Install the `opentalk-compose` Package

**Debian and Ubuntu-based Distributions:**

```bash
apt update && apt install opentalk-compose -y
```

**RHEL-based Distributions:**

```bash
dnf install opentalk-compose -y
```

**SuSE-based Distributions:**

```bash
zypper install opentalk-compose -y
```

### 2.3 Set the Server's DNS Domain Record

Set the environment variable `OT_DOMAIN` in the `default.env` file to your desired DNS domain for OpenTalk.
You can edit `/opt/opentalk-compose/config/default.env` manually or use the following command:

```bash
sed -i 's/^OT_DOMAIN=.*/OT_DOMAIN="example.com"/' /opt/opentalk-compose/config/default.env
```

This means that the dashboard of the OpenTalk web application will later be accessible, for example, at `https://example.com`.

### 2.4 Initialize the Deployment Environment

Use the `cforge` command to create directories, secrets, and configuration files required for the installation.
A default configuration is derived from the OT_DOMAIN variable.

```bash
cforge init
```

### 2.5 Optional: Edit Settings in `settings.yml`

If the default configuration does not fit your needs and you prefer different settings, you can change the relevant parameters here. However, be careful: incorrect syntax or unmet prerequisites for some configuration options may cause certain services to fail to start, and OpenTalk may not function properly.

### 2.6 Run OpenTalk Application Stack

Use `docker compose` to pull the container images and start all required services for your OpenTalk instance.

```bash
cd /opt/opentalk-compose && docker compose up -d
```

It can take several minutes for all services to start. The OpenTalk web application should now be reachable at the domain you configured in [2.3 Set the Server's DNS Domain Record](#23-set-the-servers-dns-domain-record).

### 2.8 Optional: Enable Demo User

By default, there is no user available to log in to the OpenTalk dashboard. For testing purposes, you can enable the demo user. You can find the username **demo** and the corresponding **password** in the output when you run `cforge` with the `--diff` argument.
Otherwise, users are created in the Keycloak management console. Use this to create general user accounts.

```bash
cforge --diff task demoUser
```
