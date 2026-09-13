// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use serde::Deserialize;

use crate::settings::settings_file::reports_typst::ReportsTypst;

#[derive(Default, Clone, Debug, Deserialize, PartialEq, Eq)]
pub struct Reports {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub typst: Option<ReportsTypst>,
}

#[cfg(test)]
mod tests {
    use std::path::Path;

    use pretty_assertions::assert_eq;

    use crate::settings::settings_file::{reports::Reports, reports_typst::ReportsTypst};

    #[test]
    fn report_settings_full() {
        let toml_settings: Reports = toml::from_str(
            r#"
        [typst]
        packages_path = "/usr/local/my/typst/packages"
        "#,
        )
        .unwrap();
        assert_eq!(
            toml_settings,
            Reports {
                typst: Some(ReportsTypst {
                    packages_path: Some(Path::new("/usr/local/my/typst/packages").to_path_buf())
                })
            }
        );
    }

    #[test]
    fn report_settings_empty_typst_section() {
        let toml_settings: Reports = toml::from_str("[typst]").unwrap();
        assert_eq!(
            toml_settings,
            Reports {
                typst: Some(ReportsTypst {
                    packages_path: None
                })
            }
        );
    }

    #[test]
    fn report_settings_empty() {
        let toml_settings: Reports = toml::from_str("").unwrap();
        assert_eq!(toml_settings, Reports { typst: None });
    }
}
