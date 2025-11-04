// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

//! OpenTalk report generation functionality
//!
//! This crate provides an abstraction for generating PDF reports using
//! [`typst`](https://docs.rs/typst). ! The implementation here is strongly
//! opinionated with regard to the requirements in the OpenTalk ecosystem,
//! but you still may find pieces of it useful for general purpose scenarios.

#![deny(
    bad_style,
    missing_debug_implementations,
    missing_docs,
    overflowing_literals,
    patterns_in_fns_without_body,
    trivial_casts,
    trivial_numeric_casts,
    unsafe_code,
    unused,
    unused_extern_crates,
    unused_import_braces,
    unused_qualifications,
    unused_results
)]

mod report_date_time;
mod report_generation_error;
mod world;

use std::{borrow::Cow, collections::BTreeMap, path::Path};

pub use report_date_time::{ReportDateTime, ToReportDateTime};
pub use report_generation_error::ReportGenerationError;
use report_generation_error::{CompilationSnafu, DumpDirectoryCreationSnafu, DumpFileExportSnafu};
use snafu::ResultExt;
use typst::{World as _, diag::SourceResult, syntax::package::PackageSpec};
use typst_pdf::PdfOptions;
use world::World;

/// Options to be passed into the report generation.
#[derive(Default, Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub struct GenerateOptions<'a> {
    /// A path to which the intermediate artifacts will be dumped.
    ///
    /// Should be used mainly for debugging purposes.
    pub dump_to_path: Option<&'a Path>,

    /// A path where the typst packages are stored that can be loaded for the report generation.
    ///
    /// The tree structure should be identical to what typst
    /// stores itself when downloading packages from the
    /// [typst universe](https://typst.app/universe/).
    ///
    /// This directory should contain the `preview` subfolder.
    pub packages_path: Option<&'a Path>,
}

/// Generate a pdf file from a typst source string and data.
pub fn generate_pdf_report(
    source: String,
    data: BTreeMap<&Path, (Option<PackageSpec>, Cow<'static, [u8]>)>,
    generate_options: &GenerateOptions,
) -> Result<Vec<u8>, ReportGenerationError> {
    if let Some(dump_path) = generate_options.dump_to_path {
        log::info!("Dumping raw data and generated report file to {dump_path:?}");

        std::fs::create_dir_all(dump_path).context(DumpDirectoryCreationSnafu)?;
        std::fs::write(dump_path.join("report.typ"), &source).context(DumpFileExportSnafu)?;

        for (path, (_package, content)) in &data {
            std::fs::write(dump_path.join(path), content).context(DumpFileExportSnafu)?;
        }
    }

    let mut world = World::new(source, data);

    if let Some(packages_path) = generate_options.packages_path {
        world.load_packages(packages_path)?;
    }

    let report = match generate_pdf_report_inner(&world) {
        Ok(d) => Ok(d),
        Err(e) => {
            for diagnostic in &e {
                let range = world
                    .source(*world::MAIN_ID)
                    .expect("Source not found")
                    .range(diagnostic.span);

                log::warn!("{}: {:?}", diagnostic.message, range);
            }
            CompilationSnafu { warnings: e }.fail()
        }
    }?;

    if let Some(dump_path) = generate_options.dump_to_path {
        std::fs::create_dir_all(dump_path).context(DumpDirectoryCreationSnafu)?;
        std::fs::write(dump_path.join("report.pdf"), &report).context(DumpFileExportSnafu)?;
    }
    Ok(report)
}

fn generate_pdf_report_inner(world: &World) -> SourceResult<Vec<u8>> {
    let document = typst::compile(&world).output?;
    typst_pdf::pdf(&document, &PdfOptions::default())
}
