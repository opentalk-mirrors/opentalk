// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use std::path::PathBuf;

use serde::Deserialize;

#[derive(Clone, Debug, Deserialize, Default, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub struct ReportsTypst {
    /// The path where typst will look for packages, e.g. downloaded
    /// from the [typst universe](https://typst.app/universe/).
    ///
    /// This is the directory that can contain e.g. a `local` directory (for
    /// packages deployed locally) or a `preview` directory (for packages
    /// downloaded from the typst universe).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub packages_path: Option<PathBuf>,
}
