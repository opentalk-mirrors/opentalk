---
sidebar_position: 5
---

# SMTP

{{ product_name }} SMTP Mailer sends emails via a SMTP server. These protocols are supported:

- SMTP cleartext
- SMTP with StartTLS
- SMTP with implicit TLS

## Configuration

The section in the [configuration file](configuration.md) is called `smtp`.

| Field         | Type     | Required | Default value            | Description                                                          |
| ------------- | -------- | -------- | ------------------------ | -------------------------------------------------------------------- |
| `smtp_server` | `string` | no       | "smtp://localhost:25"    | The URL of the SMTP server used for sending emails                   |
| `from_name`   | `string` | no       | "OpenTalk"               | The sender's name written to the `From` field of the emails          |
| `from_email`  | `string` | no       | "<no-reply@example.org>" | The sender's email address written to the `From` field of the emails |

Note: Encode the `smtp_server` URL properly:

- Significant characters (e.g. the '@' separating the user credentials from the domain) must be written literally.
- Special characters within the URL parts must be paraphrased (e.g. '@' replaced with '%40')

### Examples

#### SMTP cleartext to local server, explicit port, anonymous (i.e. without credentials)

```toml
[smtp]
smtp_server = "smtp://localhost:1025?disable_starttls=true"
from_name = "OpenTalk"
from_email = "no-reply@example.org"
```

#### SMTP with StartTLS to remote server, explicit port, using credentials

```toml
[smtp]
smtp_server = "smtp://user:pass@mailserver.example.org:1234"
from_name = "OpenTalk"
from_email = "no-reply@example.org"
```

#### SMTP with implicit TLS to remote server, default port, using credentials (with a username containing '@')

```toml
[smtp]
smtp_server = "smtps://user%40maildomain.example.org:pass@mailserver.example.org"
from_name = "OpenTalk"
from_email = "no-reply@example.org"
```
