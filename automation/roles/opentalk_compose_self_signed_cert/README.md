<!--
SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>

SPDX-License-Identifier: EUPL-1.2
-->

opentalk_compose_self_signed_cert
=================================

Generates a self-signed root CA and a server certificate (full chain + key) for the OpenTalk deployment and (optionally) installs the generated CA certificate into the system trust store.

Features
--------

- Idempotent certificate generation (skips if all expected target files already exist)
- SAN support (DNS + IP) with automatic port stripping
- Separate validity periods and key sizes for CA and server cert
- Optional system trust anchor installation (Debian / RedHat / openSUSE & SLES families)

Requirements
------------

Requires the Ansible collections `community.crypto`.

Directory Layout
----------------

Certificates are written to `custom_certificates_certs_dir` (default `/etc/opentalk-compose/ssl`).

Generated files (defaults):

- CA: `ca.crt`
- Server full chain: `fullchain.crt` (server cert + CA)
- Private key: `privkey.key`

Role Variables (selection)
--------------------------

`self_signed_certificates_generate` (bool, default: false)
: Generate certificates when true.

`self_signed_certificates_install_system_trust` (bool, default: false)
: If true, copy the generated CA cert into the OS trust store and trigger trust update.

`self_signed_certificates_system_trust_name` (string, default: `opentalk-selfsigned-ca.crt`)
: Filename used in the system trust anchor directory.

`self_signed_certificates_system_trust_dest_override` (string, default: empty)
: If set, overrides the automatically detected destination path (useful for non-standard trust store locations or future platforms).

`custom_certificates_certs_dir` (path, default: `/etc/opentalk-compose/ssl`)
: Target directory for final cert/key artifacts.

`self_signed_certificates_ssl_certificate_ca_file` (default: `ca.crt`)
`self_signed_certificates_ssl_certificate_fullchain_file` (default: `fullchain.crt`)
`self_signed_certificates_ssl_certificate_key_file` (default: `privkey.key`)

`self_signed_certificates_server_san_dns` / `self_signed_certificates_server_san_ip`
: Lists for SAN entries (port numbers are stripped automatically).

Trust Installation Behavior
---------------------------

When `self_signed_certificates_install_system_trust` is enabled:

- Debian family: CA copied to `/usr/local/share/ca-certificates/<self_signed_certificates_system_trust_name>` then `update-ca-certificates` executed.
- RedHat family: CA copied to `/etc/pki/ca-trust/source/anchors/<self_signed_certificates_system_trust_name>` then `update-ca-trust extract` executed.
- Suse family (openSUSE / SLES): CA copied to `/usr/share/pki/trust/anchors/<self_signed_certificates_system_trust_name>` then `update-ca-certificates` executed.

The handler only runs if the CA file copy produced a change.

Example Playbook
----------------

```yaml
- hosts: opentalk_hosts
  become: true
  roles:
    - role: opentalk_compose_self_signed_cert
      vars:
        ot_domain: "example.local"
        self_signed_certificates_generate: true
        self_signed_certificates_install_system_trust: true
        self_signed_certificates_server_san_dns:
          - "example.local"
          - "*.example.local"
        self_signed_certificates_server_san_ip:
          - "127.0.0.1"
          - "::1"
```

License
-------

EUPL-1.2

Notes
-----

Alpine support for trust installation can be added later (currently would behave like Debian if the path exists and update command is available).

Troubleshooting
---------------

If you encounter `dest is required` in the trust task:

1. Run with `-vv` to see the debug message `Computed trust destination: ...`.
2. Check `ansible_os_family` fact (e.g. `ansible -m setup -a 'filter=ansible_os_family' localhost`).
3. Set `self_signed_certificates_system_trust_dest_override` explicitly if your distribution reports an unexpected family value.
4. Ensure `ca-certificates` package is installed; role attempts installation per OS family.

If you previously saw an error like `'> not supported between instances of int and _AnsibleTaggedStr'` this was due to a Jinja comparison on a tagged string in a single complex expression. The logic was refactored into two simpler `set_fact` tasks (initialize + conditional override) to avoid this edge case.

Immediate Trust Update
----------------------

The role flushes handlers right after copying the CA certificate to ensure the system trust store is updated in the same play segment (no waiting until end of play). Remove the `meta: flush_handlers` task if you prefer deferred update.
