%define pkgname        opentalk-compose
%define pkgsummary     OpenTalk Compose Package. Compose and orchestrate OpenTalk services.
%define pkglicense     EUPL-1.2
## SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
## SPDX-License-Identifier: EUPL-1.2
%define pkgurl         https://opentalk.eu/
# These can be overridden: rpmbuild --define 'version 1.2.3' --define 'release 2'
%ifndef version
%define version 25.3.0
%endif
%ifndef release
%define release 1
%endif

Name:           %{pkgname}
Group:          Applications/Communications
Vendor:         OpenTalk GmbH
Packager:       OpenTalk GmbH <mail@opentalk.eu>
Version:        %{version}
Release:        %{release}%{?dist}
Summary:        %{pkgsummary}
License:        %{pkglicense}
URL:            %{pkgurl}
Source0:        %{name}-%{version}.tar.gz

BuildArch:      x86_64

# Runtime requirements (adjust as needed)
Requires:       bash
Requires:       rsync
Requires:       jq
Requires:       openssl
Requires:       tar
# Compose / Docker runtime (boolean dependencies where available)
Requires:       (docker-ce or docker-ee or docker)
Requires:       (docker-compose-plugin or docker-compose)
Provides:       %{pkgname}

%description
OpenTalk Orchestrierung inklusive Automation (Ansible), Wrapper-Skripten,
Konfigurationsstruktur und optional portablem Python Runtime.
Gebaut aus Commit: %{?git_commit}

%prep
%setup -q

%build
# Kein Build-Schritt nötig (Skripte / Python Runtime wird gestaged)

%install
rm -rf %{buildroot}

# Staging Script ausführen (nutzt Version Argument); git commit optional
bash extras/packaging/60-pkg-stage.sh "%{version}" || exit 1

# Kopiere gestagten Baum
cp -a build/pkgroot/. %{buildroot}/

%files
%license LICENSE* REUSE.toml
%doc README.md
/opt/opentalk-compose
/usr/bin/otctl
/usr/bin/ot-ansible
/usr/bin/ot-ansible-playbook
/usr/bin/ot-ansible-galaxy
/usr/share/opentalk-compose/examples
/usr/share/doc/opentalk-compose
%config(noreplace) /etc/opentalk-compose/settings.yml
%dir /etc/opentalk-compose
%dir /etc/opentalk-compose/secrets
%dir /etc/opentalk-compose/ssl

%post
if [ -x /opt/opentalk-compose/bin/pkg-postinst.sh ]; then
  /opt/opentalk-compose/bin/pkg-postinst.sh || echo "WARN: postinst failed"
fi

%postun
if [ $1 -eq 0 ]; then # final removal
  if [ -x /opt/opentalk-compose/bin/pkg-postrm.sh ]; then
    /opt/opentalk-compose/bin/pkg-postrm.sh || echo "WARN: postrm failed"
  fi
fi

%changelog
* Mon Oct 20 2025 Maintainer <maintainer@example.org> - %{version}-%{release}
- Initial RPM build spec (dynamic metadata)