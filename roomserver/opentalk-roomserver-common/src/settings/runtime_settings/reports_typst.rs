// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use std::path::{Path, PathBuf};

use crate::settings::settings_file;

/// typst-specific report generation configuration.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ReportsTypst {
    /// The location where typst looks for packages.
    pub packages_path: PathBuf,
}

impl Default for ReportsTypst {
    fn default() -> Self {
        Self {
            packages_path: default_packages_path(),
        }
    }
}

impl From<settings_file::reports_typst::ReportsTypst> for ReportsTypst {
    fn from(
        settings_file::reports_typst::ReportsTypst { packages_path }: settings_file::reports_typst::ReportsTypst,
    ) -> Self {
        Self {
            packages_path: packages_path.unwrap_or_else(default_packages_path),
        }
    }
}

/// The default location where typst looks for packages.
fn default_packages_path() -> PathBuf {
    Path::new("/usr/share/typst/packages").to_path_buf()
}

pub fn reports_typst_packages_test_path() -> PathBuf {
    const TYPST_PACKAGE_CACHE_PATH_ENV_VARIABLE: &str = "TYPST_PACKAGE_CACHE_PATH";
    if let Some(env_variable) = std::env::var_os(TYPST_PACKAGE_CACHE_PATH_ENV_VARIABLE) {
        Path::new(&env_variable).to_path_buf()
    } else {
        dirs::cache_dir()
            .map(|d| d.join("typst/packages"))
            .unwrap_or_else(default_packages_path)
    }
}
