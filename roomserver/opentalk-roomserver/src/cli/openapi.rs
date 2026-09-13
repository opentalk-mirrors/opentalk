// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use std::{
    fs::File,
    io::{BufWriter, Write},
    path::{Path, PathBuf},
};

use anyhow::{Context, ensure};
use clap::{Subcommand, ValueEnum};
use utoipa::{OpenApi, openapi::Server};
use yaml_rust2::{YamlEmitter, YamlLoader};

#[derive(Subcommand, Debug, Clone)]
#[clap(rename_all = "kebab_case")]
pub enum Command {
    /// Store the OpenAPI schema
    Dump {
        /// The output target
        #[clap(default_value = "-")]
        target: PathBuf,

        /// The export format
        #[clap(long, default_value = "yaml")]
        format: ExportFormat,
    },
}

#[derive(Default, ValueEnum, Debug, Clone)]
#[clap(rename_all = "kebab_case")]
pub enum ExportFormat {
    #[default]
    /// YAML output format
    Yaml,

    /// JSON output format
    Json,
}

pub(crate) fn handle_command(command: Command) -> anyhow::Result<()> {
    match command {
        Command::Dump { format, target } => {
            let mut outstream: Box<dyn Write> = if target == Path::new("-").to_path_buf() {
                Box::new(std::io::stdout().lock())
            } else {
                Box::new(BufWriter::new(
                    File::create(target).context("Failed to open file for writing")?,
                ))
            };

            let mut api = crate::api::ApiDoc::openapi();
            api.servers = Some(vec![Server::new("/v1")]);

            let openapi_json_string = api.to_pretty_json()?;

            let content = match format {
                ExportFormat::Yaml => {
                    // The builtin yaml export feature of the `utoipa` crate
                    // exports in an inferior yaml structure without a yaml
                    // document start marker, and with indentation that
                    // doesn't live up to the YAML format we prefer. Therefore
                    // we deserialize the JSON representation and serialize with
                    // a library that matches our expectations better. JSON is
                    // a subset of YAML, so we can simply load the serialized
                    // JSON back into a YAML representation and use that for
                    // generating our output.
                    let loaded_yaml = YamlLoader::load_from_str(&openapi_json_string)
                        .context("Failed to load serialized OpenAPI JSON")?;

                    ensure!(
                        loaded_yaml.len() == 1,
                        "Loaded YAML data should contain exactly one document"
                    );

                    let mut openapi_yaml_string = String::new();
                    let mut yaml_emitter = YamlEmitter::new(&mut openapi_yaml_string);
                    yaml_emitter.multiline_strings(true);
                    yaml_emitter.dump(
                        loaded_yaml
                            .first()
                            .expect("yaml to have at least one element"),
                    )?;
                    openapi_yaml_string
                }
                ExportFormat::Json => openapi_json_string,
            };

            write!(outstream, "{content}")?;

            Ok(())
        }
    }
}
