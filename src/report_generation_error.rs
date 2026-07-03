// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use std::path::PathBuf;

use ecow::EcoVec;
use snafu::Snafu;
use typst::{diag::SourceDiagnostic, syntax::PathError};

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

    /// Error reading the typst packages directory
    #[snafu(display("Error reading the typst packages directory {path:?}"))]
    ReadPackagesDir {
        /// The packages directory path that could not be read
        path: PathBuf,

        /// The source of the error
        source: std::io::Error,
    },

    /// Error reading the entries in the typst packages directory
    #[snafu(display("Error reading entry in the typst packages directory {packages_path:?}"))]
    ReadPackagesDirEntry {
        /// The packages directory path inside which an entry could not be read
        packages_path: PathBuf,

        /// The source of the error
        source: std::io::Error,
    },

    /// Error reading the typst packages namespace directory
    #[snafu(display("Error reading the typst packages namespace directory {path:?}"))]
    ReadPackagesNamespaceDir {
        /// The package namespace directory path that could not be read
        path: PathBuf,

        /// The source of the error
        source: std::io::Error,
    },

    /// Error reading the entries in a typst packages namespace directory
    #[snafu(display(
        "Error reading entry in a typst packages namespace directory {namespace_path:?}"
    ))]
    ReadPackagesNamespaceDirEntry {
        /// The package namespace directory path inside which an entry could not be read
        namespace_path: PathBuf,

        /// The source of the error
        source: std::io::Error,
    },

    /// Error reading a typst package from a directory
    #[snafu(display("Error reading a typst package from a directory {package_path:?}"))]
    ReadPackageDir {
        /// The package directory path that could not be read
        package_path: PathBuf,

        /// The source of the error
        source: std::io::Error,
    },

    /// Error reading a typst package from a directory
    #[snafu(display("Error reading an entry inside typst package directory {package_path:?}"))]
    ReadPackageDirEntry {
        /// The package directory path that could not be read
        package_path: PathBuf,

        /// The source of the error
        source: std::io::Error,
    },

    /// Error reading the file name of a typst package version directory
    #[snafu(display(
        "Error reading the file name of typst package version directory {package_version_path:?}"
    ))]
    ReadPackageVersionFileName {
        /// The package version directory path that could not be read
        package_version_path: PathBuf,
    },

    /// Error reading the file name of a typst package version directory
    #[snafu(display(
        "File name of typst package version directory {package_version_path:?} is not fully UTF-8"
    ))]
    ReadPackageVersionFileNameNotUtf8 {
        /// The package version directory path that could not be read
        package_version_path: PathBuf,
    },

    /// Error reading a typst package version directory
    #[snafu(display(
        "Error reading the file name of typst package version directory {package_version_path:?}"
    ))]
    ReadPackageVersionDirectory {
        /// The package version directory path that could not be read
        package_version_path: PathBuf,

        /// The source of the error
        source: std::io::Error,
    },

    /// Package version is not fully UTF-8, therefore cannot be used for building the glob pattern.
    #[snafu(display(
        "Path {package_version_path:?} is not valid UTF-8, so it can't be used for globbing the directory contents"
    ))]
    PackageVersionInvalidUtf8ForGlob {
        /// The package version directory path that could not be converted to UTF-8
        package_version_path: PathBuf,
    },

    /// Package version glob pattern is invalid.
    #[snafu(display("Package version path glob pattern {glob_pattern:?} invalid"))]
    PackageVersionInvalidGlobPattern {
        /// The invalid glob pattern
        glob_pattern: String,

        /// The source of the error
        source: glob::PatternError,
    },

    /// Could not read a file when globbing
    #[snafu(display("File {path:?} could not be read when globbing"))]
    PackageFileGlob {
        /// The path of the which could not be read
        path: PathBuf,

        /// The source of the error
        source: glob::GlobError,
    },

    /// Could not read file inside a package
    #[snafu(display("File {path:?} could not be read"))]
    PackageFileRead {
        /// The path of the file which could not be read
        path: PathBuf,

        /// The source of the error
        source: std::io::Error,
    },

    /// A file path of the package is not valid UTF-8
    #[snafu(display("Package path {path:?} isn't valid UTF-8"))]
    PackageFilePathNotUtf8 {
        /// The path that isn't valid UTF-8
        path: PathBuf,
    },

    /// A file path of the package contains characters not supported by the typst crate
    #[snafu(display("Package path {path:?} is not supported by typst"))]
    PackagePathNotSupported {
        /// The path that contains characters not supported by the typst crate
        path: PathBuf,

        /// The source of the error
        source: PathError,
    },
}
