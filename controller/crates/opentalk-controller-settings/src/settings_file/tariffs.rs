// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use serde::Deserialize;

use super::TariffAssignment;

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub(crate) struct Tariffs {
    #[serde(default, flatten, skip_serializing_if = "Option::is_none")]
    pub assignment: Option<TariffAssignment>,
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeSet;

    use pretty_assertions::assert_eq;
    use serde::Deserialize;

    use super::Tariffs;
    use crate::settings_file::{TariffAssignment, TariffStatusMapping};

    #[derive(Debug, Clone, Default, PartialEq, Eq, Deserialize)]
    struct DummySettings {
        #[serde(default, skip_serializing_if = "Option::is_none")]
        tariffs: Option<Tariffs>,
    }

    #[test]
    fn empty() {
        let toml_settings: DummySettings = toml::from_str(
            r#"
        "#,
        )
        .unwrap();

        assert_eq!(toml_settings, DummySettings { tariffs: None });
    }

    #[test]
    fn default() {
        let toml_settings: DummySettings = toml::from_str(
            r#"
            [tariffs]
        "#,
        )
        .unwrap();

        assert_eq!(
            toml_settings,
            DummySettings {
                tariffs: Some(Tariffs { assignment: None })
            }
        );
    }

    #[test]
    fn static_without_tariff_name() {
        let toml_settings: DummySettings = toml::from_str(
            r#"
            [tariffs]
            assignment = "static"
        "#,
        )
        .unwrap();

        assert_eq!(
            toml_settings,
            DummySettings {
                tariffs: Some(Tariffs {
                    assignment: Some(TariffAssignment::Static {
                        static_tariff_name: None
                    })
                })
            }
        );
    }

    #[test]
    fn static_with_tariff_name() {
        let toml_settings: DummySettings = toml::from_str(
            r#"
            [tariffs]
            assignment = "static"
            static_tariff_name = "my_tariff"
        "#,
        )
        .unwrap();

        assert_eq!(
            toml_settings,
            DummySettings {
                tariffs: Some(Tariffs {
                    assignment: Some(TariffAssignment::Static {
                        static_tariff_name: Some("my_tariff".to_string())
                    })
                })
            }
        );
    }

    #[test]
    fn external_without_status_mapping() {
        let toml_settings: DummySettings = toml::from_str(
            r#"
            [tariffs]
            assignment = "by_external_tariff_id"
        "#,
        )
        .unwrap();

        assert_eq!(
            toml_settings,
            DummySettings {
                tariffs: Some(Tariffs {
                    assignment: Some(TariffAssignment::ByExternalTariffId {
                        status_mapping: None
                    })
                })
            }
        );
    }

    #[test]
    fn external_with_status_mapping() {
        let toml_settings: DummySettings = toml::from_str(
            r#"
            [tariffs]
            assignment = "by_external_tariff_id"

            [tariffs.status_mapping]
            downgraded_tariff_name = "basic"
            default = ["default", "standard"]
            paid = ["paid", "all_ok"]
            downgraded = ["unpaid"]
        "#,
        )
        .unwrap();

        assert_eq!(
            toml_settings,
            DummySettings {
                tariffs: Some(Tariffs {
                    assignment: Some(TariffAssignment::ByExternalTariffId {
                        status_mapping: Some(TariffStatusMapping {
                            downgraded_tariff_name: "basic".to_string(),
                            default: BTreeSet::from([
                                "default".to_string(),
                                "standard".to_string()
                            ]),
                            paid: BTreeSet::from(["paid".to_string(), "all_ok".to_string(),]),
                            downgraded: BTreeSet::from(["unpaid".to_string()])
                        })
                    })
                })
            }
        );
    }
}
