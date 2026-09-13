// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use serde::Deserialize;

#[derive(Clone, Debug, Deserialize, PartialEq, Eq)]
#[serde(tag = "provider", rename_all = "snake_case")]
pub(crate) enum SharedFolder {
    Nextcloud {
        url: url::Url,
        username: String,
        password: String,
        #[serde(default)]
        directory: String,
        #[serde(default)]
        expiry: Option<u64>,
    },
    Opencloud {
        url: url::Url,
        username: String,
        password: String,
        #[serde(default)]
        directory: String,
        #[serde(default)]
        expiry: Option<u64>,
    },
}

#[cfg(test)]
mod tests {
    use pretty_assertions::assert_eq;
    use serde_json::json;

    use super::*;

    #[test]
    fn shared_folder_provider_nextcloud() {
        let shared_folder = SharedFolder::Nextcloud {
            url: "https://nextcloud.example.org/".parse().unwrap(),
            username: "exampleuser".to_string(),
            password: "v3rys3cr3t".to_string(),
            directory: "meetings/opentalk".to_string(),
            expiry: Some(34),
        };
        let json = json!({
            "provider": "nextcloud",
            "url": "https://nextcloud.example.org/",
            "username": "exampleuser",
            "password": "v3rys3cr3t",
            "directory": "meetings/opentalk",
            "expiry": 34,
        });

        assert_eq!(
            serde_json::from_value::<SharedFolder>(json).unwrap(),
            shared_folder
        );
    }

    #[test]
    fn shared_folder_provider_opencloud() {
        let shared_folder = SharedFolder::Opencloud {
            url: "https://opencloud.example.org/".parse().unwrap(),
            username: "exampleuser".to_string(),
            password: "v3rys3cr3t".to_string(),
            directory: "meetings/opentalk".to_string(),
            expiry: Some(34),
        };
        let json = json!({
            "provider": "opencloud",
            "url": "https://opencloud.example.org/",
            "username": "exampleuser",
            "password": "v3rys3cr3t",
            "directory": "meetings/opentalk",
            "expiry": 34,
        });

        assert_eq!(
            serde_json::from_value::<SharedFolder>(json).unwrap(),
            shared_folder
        );
    }
}
