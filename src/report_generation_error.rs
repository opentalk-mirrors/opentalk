// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use ecow::EcoVec;
use snafu::Snafu;
use typst::diag::SourceDiagnostic;

/// An error that can happen during report generation
#[derive(Debug, Snafu)]
#[snafu(visibility(pub(crate)))]
pub enum ReportGenerationError {
    /// Compilation failed
    #[snafu(display("Compilation failed"))]
    Compilation {
        /// The warnings that were emitted during compilation
        warnings: EcoVec<SourceDiagnostic>,
    },

    /// Error creating the dump directory
    #[snafu(display("Error creating the dump directory"))]
    DumpDirectoryCreation {
        /// The source of the error
        source: std::io::Error,
    },

    /// Error exporting dump file
    #[snafu(display("Error exporting dump file"))]
    DumpFileExport {
        /// The source of the error
        source: std::io::Error,
    },
}
