// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use std::path::{Path, PathBuf};

use crate::settings_file;

/// typst-specific report generation configuration.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ReportsTypst {
    /// The location where typst looks for packages.
    pub packages_path: PathBuf,
}

impl Default for ReportsTypst {
    fn default() -> Self {
        Self {
            packages_path: reports_typst_default_packages_path().to_path_buf(),
        }
    }
}

impl From<settings_file::ReportsTypst> for ReportsTypst {
    fn from(settings_file::ReportsTypst { packages_path }: settings_file::ReportsTypst) -> Self {
        Self {
            packages_path: packages_path
                .unwrap_or_else(|| reports_typst_default_packages_path().to_path_buf()),
        }
    }
}

/// The default location where typst looks for packages.
pub fn reports_typst_default_packages_path() -> &'static Path {
    Path::new("/usr/share/typst/packages")
}
