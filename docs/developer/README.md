# {{ product_name }} Developer Documentation

## Developer documentation for projects

- [{{ product_name }} Controller](controller/README.md)
- [{{ product_name }} SMTP-Mailer](smtp-mailer/README.md)

## Community

Even though we don't have collaboration processes established yet, we have
channels through which you can communicate with some of the developers. Please
be aware that there is no guarantee for an answer, because developers are
present in these channels purely on their own initiative.

- Matrix Chat Community Space [`#opentalk-community:matrix.org`](https://matrix.to/#/#opentalk-community:matrix.org)
- Matrix Chat Development Channel [`#opentalk-development:matrix.org`](https://matrix.to/#/#opentalk-development:matrix.org)
- [OpenTalk Development Mailing List](https://lists.opentalk.eu/mailman/listinfo/opentalk-devel)

## Source Code

OpenTalk is an Open Source project published under the
[`EUPL-1.2`](https://interoperable-europe.ec.europa.eu/collection/eupl/eupl-text-eupl-12) license.
The source code is publicly available in `git` repositories under the
[OpenTalks GitLab instance](https://git.opentalk.dev/opentalk/) and
[OpenTalk organization on OpenCoDE](https://gitlab.opencode.de/opentalk).

Development of the project happens on our [public OpenTalk GitLab instance](https://git.opentalk.dev/opentalk/).
The source code on OpenCoDE is just a mirror of that code which gets
synchronized for every commit that gets merged into the `main` or release
branches of the internal repositories. It has yet to be decided how
contributions can be accepted from the community. Therefore opening merge
requests on GitLab or OpenCoDE is currently not possible.

### Source code on other platforms

#### crates.io

OpenTalk works towards publishing all open source code written in
[Rust](https://rust-lang.org/) on the crates.io community crate registry.
The code is published by the
[OpenTalk-Bot](https://crates.io/users/OpenTalk-Bot) crates.io user. Please be
aware that not all code is published yet, and we need to remove some blockers by
e.g. restructuring or publishing internal crates first.
