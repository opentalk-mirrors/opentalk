# Development related docs

[[_TOC_]]

## OpenTalk Compose project sources

```bash
git clone ssh://git@git.opentalk.dev:222/opentalk/opentalk-devops/opentalk-compose.git && cd opentalk-compose
```

Currently, work is underway to replace the previous deployment approach using `cforge` with a new approach based on embedded Ansible. Until the `embedded-ansible` branch is merged into `main`, it should be considered the "main" branch. Therefore, branch off from `embedded-ansible` and set `embedded-ansible` as the target branch for merge requests.

## Building Packages

You can use `--deb` or `--rpm` to create packages in the respective format. If you omit these options, packages in both formats will be created.

### Building `.deb` Packages

```bash
extras/packaging/build-all.sh --deb --version 25.3.0 --release pre.$(date +"%y%m%d%H%M%S")
```

### Building `.rpm` Packages

```bash
extras/packaging/build-all.sh --rpm --version 25.3.0 --release pre.$(date +"%y%m%d%H%M%S")
```

## Setup local Dev environment for OpenTalk

[WIP]
