# SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
#
# SPDX-License-Identifier: EUPL-1.2

FROM debian:13-slim AS build

ENV DEBIAN_FRONTEND=noninteractive
ENV UV_VERSION=0.12.3

RUN apt-get update && apt-get install -y --no-install-recommends \
    git \
    curl \
    jq \
    bash \
    just \
    build-essential \
    make \
    ruby \
    ruby-dev \
    zlib1g-dev \
    ca-certificates \
    python3 \
    python3-dev \
    python3-pip \
    python3-venv \
    python3-requests \
    libssl-dev \
    libffi-dev \
    yamllint \
    yq \
    ansible \
    rsync \
    xz-utils \
    unzip \
    binutils \
    file \
    shellcheck \
    markdownlint \
    ansible-lint \
    rpm \
    cpio \
    gpg \
    dirmngr \
    gnupg \
    gnupg-l10n \
    gnupg-utils \
    gpg-agent \
    gpg-wks-client \
    gpgsm \
    gpgv \
    reuse \
    npm \
    locales \
    && rm -rf /var/lib/apt/lists/*

RUN npm install -g @commitlint/cli @commitlint/config-conventional
RUN gem install fpm
RUN echo "de_DE.UTF-8 UTF-8" >> /etc/locale.gen && locale-gen
RUN curl -L "https://github.com/astral-sh/uv/releases/download/${UV_VERSION}/uv-x86_64-unknown-linux-gnu.tar.gz" \
    | tar -xz -C /usr/local/bin --strip-components=1 uv-x86_64-unknown-linux-gnu/uv && \
    uv --version
RUN curl -L https://github.com/openkilt/openrepo/releases/download/v1.0.0/openrepo_cli_x86_64 -o /usr/local/bin/openrepo && \
    chmod +x /usr/local/bin/openrepo

WORKDIR /workspace