// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use std::str::FromStr;

use crate::{sql_enum, utils::ExampleData};
sql_enum!(
    feature_gated:

    #[derive(PartialEq, Eq, PartialOrd, Ord, Hash)]
    #[cfg_attr(
        feature = "serde",
        derive(serde::Serialize, serde::Deserialize),
        serde(rename_all = "snake_case")
    )]
    #[cfg_attr(
        feature = "utoipa",
        derive(utoipa::ToSchema),
        schema(example = json!(Theme::example_data()))
    )]
    Theme,
    "theme",
    ThemeType,
    {
        Light = b"light",
        Dark = b"dark",
        System = b"system",
    }
);

impl ExampleData for Theme {
    fn example_data() -> Self {
        Self::System
    }
}

impl FromStr for Theme {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "light" => Ok(Self::Light),
            "dark" => Ok(Self::Dark),
            "system" => Ok(Self::System),
            _ => Err(format!("unknown theme {s:?}")),
        }
    }
}

impl std::fmt::Display for Theme {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(match self {
            Theme::Light => "light",
            Theme::Dark => "dark",
            Theme::System => "system",
        })
    }
}

#[cfg(test)]
mod tests {
    use pretty_assertions::assert_eq;

    use super::Theme;

    #[test]
    fn parse_valid() {
        assert_eq!("light".parse::<Theme>().unwrap(), Theme::Light);
        assert_eq!("LIGHT".parse::<Theme>().unwrap(), Theme::Light);
        assert_eq!("dark".parse::<Theme>().unwrap(), Theme::Dark);
        assert_eq!("system".parse::<Theme>().unwrap(), Theme::System);
    }

    #[test]
    fn parse_invalid() {
        assert!("unknown".parse::<Theme>().is_err());
    }

    #[test]
    fn display() {
        assert_eq!(Theme::Light.to_string(), "light");
        assert_eq!(Theme::Dark.to_string(), "dark");
        assert_eq!(Theme::System.to_string(), "system");
    }
}
