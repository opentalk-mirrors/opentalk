# Memory-profiling {{ product_name }} SMTP-Mailer

With a few commands, the memory usage of the SMTP-Mailer can be recorded using
some tooling that is easily available on most widespread Linux distributions.

## Prerequisites

- `amqp-tools`, `heaptrack` and optionally `heaptrack-gui` should be installed,
  e.g. on Debian using `apt install amqp-tools heaptrack heaptrack-gui`.
- RabbitMQ service must be set up, either in a container or directly on the
  host.
- SMTP-Mailer must be able to run, e.g. by having the `config.toml` in place so
  that it picks its tasks from the RabbitMQ service and hands the outgoing mail
  to some SMTP host (use e.g. [MailCrab](https://github.com/tweedegolf/mailcrab)).
  if you don't want to send out real mail messages).
- A rust development environment must be available, see <https://rustup.rs> for
  the instructions.

!!! warning

    Make sure to use a sandbox E-Mail setup, such as
    [MailCrab](https://github.com/tweedegolf/mailcrab). Sending a lot of automated
    E-Mails in a short period of time can get you banned on your E-Mail provider.

## Profiling the memory usage

Export the example message files using this command:
`cargo run --package opentalk-mail-worker-protocol --features="utoipa"`.
This will write a set of example files in the current directory. Pick any of
these that you would like to use for generating the mail, we will use
`registered_event_invite.json` below.

Change into the `opentalk-smtp-mailer` directory and build the executable with
the `release` profile:

```text
cd crates/opentalk-smtp-mailer
cargo build --release
```

Now run the SMTP-Mailer service with `heaptrack`. If you'd like to see some
output on the command-line, set the `RUST_LOG` environment variable to `info`.

```text
RUST_LOG=info heaptrack ../../target/release/opentalk-smtp-mailer --config ../../config.toml
```

In a different terminal, push the desired number of messages into the RabbitMQ
queue:

```texd
for i in $(seq 1 300); do amqp-publish -r opentalk_mailer < ../../registered_event_invite.json; done
```

Wait for all the mail to be processed.

Interrupting the `opentalk-smtp-mailer` process using `CTRL+C` will also abort
`heaptrack`, so we need to kill the process manually.

Find the process id, e.g. with `ps aux | grep opentalk-smtp-mailer`. The first
column is the user id, the second is the process id. Kill the process using
`kill <process_id>`. If you are certain that you have only one
`opentalk-smtp-mailer` process running and the `killall` command is available on
your system, you could instead run `killall opentalk-smtp-mailer` instead.

If you have `heaptrack-gui` installed, it will be opened automatically, showing
the recorded memory consumption.
