// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use serde::{Deserialize, Serialize};

use super::Result;
use crate::{Error, KeycloakAdminClient};

const MAX_NUM_KEYCLOAK_SEARCH_RESULTS: i32 = 100;
const MAX_USER_SEARCH_RESULTS: usize = 50;

#[derive(Debug, Deserialize, PartialEq, Eq)]
#[serde(untagged)]
enum ParseResult {
    User(User),
    Failure {},
}

impl From<ParseResult> for Option<User> {
    fn from(result: ParseResult) -> Self {
        match result {
            ParseResult::User(user) => Some(user),
            _ => None,
        }
    }
}

#[derive(Clone, Debug, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct User {
    pub id: String,
    pub username: String,
    pub enabled: bool,
    pub email_verified: bool,
    pub first_name: String,
    pub last_name: String,
    pub email: String,
    #[serde(default)]
    pub attributes: serde_json::Map<String, serde_json::Value>,
}

impl User {
    pub fn get_keycloak_user_id(&self, user_attribute_name: Option<&str>) -> Option<&str> {
        if let Some(user_attribute_name) = user_attribute_name {
            if let Some(serde_json::Value::Array(user_id_vector)) =
                self.attributes.get(user_attribute_name)
            {
                return user_id_vector.iter().find_map(|v| v.as_str());
            }
            return None;
        }

        Some(self.id.as_str())
    }

    fn is_in_tenant(&self, tenant_id_user_attribute_name: &str, tenant_id: &str) -> bool {
        if let Some(serde_json::Value::Array(tenant_id_vector)) =
            &self.attributes.get(tenant_id_user_attribute_name)
        {
            return tenant_id_vector.iter().any(|t| t == tenant_id);
        }

        false
    }
}

#[derive(Serialize)]
struct SearchQuery<'s> {
    search: &'s str,
    max: i32,
}

#[derive(Serialize)]
struct VerifyEmailQuery<'s> {
    email: &'s str,
    exact: bool,
}

#[derive(Serialize)]
struct LimitQuery {
    first: i32,
    max: i32,
}

impl KeycloakAdminClient {
    /// Query keycloak for users using the given search string
    pub async fn search_user(&self, search_str: &str, max_users: usize) -> Result<Vec<User>> {
        let url = self.url(["admin", "realms", &self.realm, "users"])?;

        let query = SearchQuery {
            search: search_str,
            max: MAX_USER_SEARCH_RESULTS.min(max_users) as i32,
        };

        let response = self
            .send_authorized(move |c| c.get(url.clone()).query(&query))
            .await?;

        let status = response.status();
        if !status.is_success() {
            log::warn!(
                "Received unsuccessful status code {} from KeyCloak when attempting to search for a user",
                status.as_u16(),
            );
            log::warn!("{}", response.text().await?);
            return Err(Error::KeyCloak);
        }

        let found_results = response.json::<Vec<ParseResult>>().await?;
        let found_results_amount = found_results.len();
        let found_users: Vec<_> = found_results
            .into_iter()
            .filter_map(Option::<User>::from)
            .collect();

        if found_results_amount != found_users.len() {
            log::warn!("User JSON object from KeyCloak is missing at least one field.");
        }

        Ok(found_users)
    }

    /// Query keycloak for users using the given search string, filtered by the tenant_id
    pub async fn search_user_filtered(
        &self,
        tenant_id_user_attribute_name: &str,
        tenant_id: &str,
        search_str: &str,
        max_users: usize,
    ) -> Result<Vec<User>> {
        let url = self.url(["admin", "realms", &self.realm, "users"])?;

        // TODO: Fix this code once https://github.com/keycloak/keycloak/issues/16687 is resolved.
        // Currently we let keycloak give us MAX_NUM_KEYCLOAK_SEARCH_RESULTS users matching the search_str
        // and then filter by the tenant_id.
        // Ideally we want keycloak to filter these out for us. In a larger user base with more tenants this will
        // be insufficient to provide a good auto-completion, since more than MAX_NUM_KEYCLOAK_SEARCH_RESULTS
        // users outside of the searched tenant might match.
        let query = SearchQuery {
            search: search_str,
            max: MAX_NUM_KEYCLOAK_SEARCH_RESULTS,
        };

        let response = self
            .send_authorized(move |c| c.get(url.clone()).query(&query))
            .await?;

        let status = response.status();
        if !status.is_success() {
            log::warn!(
                "Received unsuccessful status code {} from KeyCloak when attempting to search for a user in a tenant",
                status.as_u16(),
            );
            log::warn!("{}", response.text().await?);
            return Err(Error::KeyCloak);
        }

        let found_results = response.json::<Vec<ParseResult>>().await?;
        let found_results_amount = found_results.len();
        let found_users: Vec<_> = found_results
            .into_iter()
            .filter_map(Option::<User>::from)
            .filter(|user| user.is_in_tenant(tenant_id_user_attribute_name, tenant_id))
            .take(MAX_USER_SEARCH_RESULTS.min(max_users))
            .collect::<Vec<User>>();

        if found_results_amount != found_users.len() {
            log::warn!("User JSON object from KeyCloak is missing at least one field.");
        }

        Ok(found_users)
    }

    /// Query keycloak to get the first user that matches the given email
    pub async fn get_user_for_email<'a>(
        &self,
        tenant_filter: Option<TenantFilter<'a>>,
        email: &str,
    ) -> Result<Option<User>> {
        let url = self.url(["admin", "realms", &self.realm, "users"])?;

        let query = VerifyEmailQuery { email, exact: true };

        let response = self
            .send_authorized(move |c| c.get(url.clone()).query(&query))
            .await?;

        let status = response.status();
        if !status.is_success() {
            log::warn!(
                "Received unsuccessful status code {} from KeyCloak when attempting to get the user for an e-mail address",
                status.as_u16(),
            );
            log::warn!("{}", response.text().await?);
            return Err(Error::KeyCloak);
        }

        let found_users: Vec<User> = response.json().await?;
        let first_matching_user = found_users.iter().find(|user| {
            let tenant_matches = match &tenant_filter {
                Some(tenant_filter) => {
                    user.is_in_tenant(tenant_filter.field_name, tenant_filter.id)
                }
                None => true,
            };
            tenant_matches && user.email == email
        });

        if let Some(user) = first_matching_user {
            return Ok(Some((*user).clone()));
        }
        Ok(None)
    }

    /// Query keycloak users
    pub async fn get_users_chunk(&self, offset: i32, limit: i32) -> Result<Vec<User>> {
        let url = self.url(["admin", "realms", &self.realm, "users"])?;

        let query = LimitQuery {
            first: offset,
            max: limit,
        };

        let response = self
            .send_authorized(move |c| c.get(url.clone()).query(&query))
            .await?;

        let status = response.status();
        if !status.is_success() {
            log::warn!(
                "Received unsuccessful status code {} from KeyCloak when attempting to list users",
                status.as_u16(),
            );
            log::warn!("{}", response.text().await?);
            return Err(Error::KeyCloak);
        }

        let found_results = response.json::<Vec<ParseResult>>().await?;
        let found_results_amount = found_results.len();
        let found_users: Vec<_> = found_results
            .into_iter()
            .filter_map(Option::<User>::from)
            .collect();

        if found_results_amount != found_users.len() {
            log::warn!("User JSON object from KeyCloak is missing at least one field.");
        }

        Ok(found_users)
    }

    /// Query keycloak users count
    pub async fn get_users_count(&self) -> Result<i32> {
        let url = self.url(["admin", "realms", &self.realm, "users", "count"])?;

        let response = self.send_authorized(move |c| c.get(url.clone())).await?;

        let status = response.status();
        if !status.is_success() {
            log::warn!(
                "Received unsuccessful status code {} from KeyCloak when attempting to list users",
                status.as_u16(),
            );
            log::warn!("{}", response.text().await?);
            return Err(Error::KeyCloak);
        }

        let users_count = response.json::<i32>().await?;

        Ok(users_count)
    }
}

pub struct TenantFilter<'a> {
    pub field_name: &'a str,
    pub id: &'a str,
}

#[cfg(test)]
mod tests {
    #[test]
    fn deserialize_user() {
        use super::*;

        let json_with_email = serde_json::json!({
            "id": "someid",
            "username": "someuser",
            "enabled": true,
            "emailVerified": true,
            "firstName": "somefirstname",
            "lastName": "somelastname",
            "email": "someemail",
        });

        let json_without_email = serde_json::json!({
            "id": "someid",
            "username": "someuser",
            "enabled": true,
            "emailVerified": true,
            "firstName": "somefirstname",
            "lastName": "somelastname",
        });

        let parse_result_user: ParseResult = serde_json::from_value(json_with_email).unwrap();
        let parse_result_failure: ParseResult = serde_json::from_value(json_without_email).unwrap();

        assert!(matches!(parse_result_user, ParseResult::User(_)));
        assert!(matches!(parse_result_failure, ParseResult::Failure {}));
    }
}
