# Log Output

The log output of the {{ product_name }} SMTP Mailer can be configured, allowing administrators to control the
verbosity and granularity of log messages.

## Log level

The verbosity of the log output of the {{ product_name }} SMTP Mailer is configured by setting a log level. The available log levels
are: "error", "warn", "info", "debug" and "trace". Next to those its also possible to set the log level to "off" which
completely turns of the logging. By default the log level of the {{ product_name }} SMTP Mailer is set to "warn". For more information
on these log levels see the [documentation of the log crate](https://docs.rs/log/latest/log/enum.Level.html).

## Setting the log level

The log level of the {{ product_name }} SMTP Mailer can be configured with the `RUST_LOG` environment variable. So for example, if you
want to set the global log level to "debug" you could start the {{ product_name }} SMTP Mailer with this command:

```sh
RUST_LOG=debug opentalk-smtp-mailer
```

Next to the global log level you can set the log level of specific modules. This is done by providing a key-value pair, where
the key is the name of the module and the key is the log level you want to set for this module. These pairs can be chained
together in a comma seperated list, if you want to set the log level for multiple modules. So for example, if you want to start
the {{ product_name }} SMTP Mailer with the global log level set to "info" and the log level of the `opentalk_smtp_mailer::settings`
module set to"warn" you could run the following command:

```sh
RUST_LOG=debug,opentalk_smtp_mailer::settings=warn opentalk-smtp-mailer
```

### Special cases

We discovered that some crates we depend on give very verbose output when the log level is set to "debug". This made the
log output very hard to navigate and thus we decided to limit the log level of those crates to the default("warn"), even when
the global log level is set to a more verbose level. The log level of those crates can still be set explicitly, though.

The crates that are currently handled this way are:

- [html5ever](https://crates.io/crates/html5ever)
- [selectors](https://crates.io/crates/selectors)

So for example to set the log level of the `selectors` crate to "debug", while having the global log level also on "debug"
you could start the {{ product_name }} SMTP Mailer with the following command:

```sh
RUST_LOG=debug,selectors=debug opentalk-smtp-mailer
```
