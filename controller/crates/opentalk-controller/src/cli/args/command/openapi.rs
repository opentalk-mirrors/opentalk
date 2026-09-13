// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use std::{
    fs::File,
    io::{BufWriter, Write},
    path::{Path, PathBuf},
};

use clap::{Args, Subcommand, ValueEnum};
use itertools::Itertools as _;
use snafu::{ResultExt, ensure_whatever};
use utoipa::{OpenApi as _, openapi::Server};
use yaml_rust2::{YamlEmitter, YamlLoader};

use crate::Result;

#[derive(Subcommand, Debug, Clone)]
#[clap(rename_all = "kebab_case")]
pub enum Command {
    /// Dump the OpenAPI specification
    Dump(DumpArguments),
}
impl Command {
    pub fn exec(self) -> Result<()> {
        match self {
            Command::Dump(args) => args.exec(),
        }
    }
}

#[derive(Args, Debug, Clone)]
#[clap(rename_all = "kebab_case")]
pub struct DumpArguments {
    /// The output target (either a filename, or `-` for stdout)
    #[clap(default_value = "-")]
    target: PathBuf,

    /// The export format
    #[clap(long, default_value = "yaml")]
    format: ExportFormat,
}

impl DumpArguments {
    fn exec(self) -> Result<()> {
        let DumpArguments { format, target } = self;

        let mut outstream: Box<dyn Write> = if target == Path::new("-").to_path_buf() {
            Box::new(std::io::stdout().lock())
        } else {
            Box::new(BufWriter::new(
                File::create(target).whatever_context("Failed to open file for writing")?,
            ))
        };

        let mut api = opentalk_controller_core::ApiDoc::openapi();
        api.servers = Some(vec![Server::new("/v1")]);

        let openapi_json_string = api
            .to_pretty_json()
            .whatever_context("Failed to serialize OpenAPI")?;

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
                    .whatever_context("Failed to load serialized OpenAPI JSON")?;

                ensure_whatever!(
                    loaded_yaml.len() == 1,
                    "Loaded YAML data should contain exactly one document"
                );

                let mut openapi_yaml_string = String::new();
                let mut yaml_emitter = YamlEmitter::new(&mut openapi_yaml_string);
                yaml_emitter.multiline_strings(true);
                yaml_emitter.dump(&loaded_yaml[0]).unwrap();
                normalized_string(&openapi_yaml_string)
            }
            ExportFormat::Json => openapi_json_string,
        };

        write!(outstream, "{content}").whatever_context("Failed to write OpenAPI specification")
    }
}

#[derive(Default, ValueEnum, Debug, Clone)]
#[clap(rename_all = "kebab_case")]
enum ExportFormat {
    #[default]
    /// YAML output format
    Yaml,

    /// JSON output format
    Json,
}

fn normalized_string(s: &str) -> String {
    s.lines()
        .chain(std::iter::once("\n"))
        .map(str::trim_end)
        .join("\n")
}
