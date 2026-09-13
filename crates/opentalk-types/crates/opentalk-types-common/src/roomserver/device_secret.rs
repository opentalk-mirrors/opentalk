// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use std::str::FromStr;

use snafu::{Snafu, ensure};

use crate::utils::ExampleData;

/// The minimum allowed length for a valid device secret
pub const DEVICE_SECRET_MIN_LENGTH: usize = 16;

/// The maximum allowed length for a valid device secret
pub const DEVICE_SECRET_MAX_LENGTH: usize = 255;

/// A device secret.
///
/// Is generated and provided by the client to allow the roomserver to distinguish between devices. It is also used to
/// identify the client and associate old state when reconnecting.
///
/// A guest may only use one device, any new device secret would be considered a new participant.
///
/// Can be parsed using [`std::str::FromStr`].
/// Must contain at least [`DEVICE_SECRET_MIN_LENGTH`] characters, at most
/// [`DEVICE_SECRET_MAX_LENGTH`] characters.
#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Hash, derive_more::Display)]
#[cfg_attr(
    feature = "serde",
    derive(serde::Serialize, serde_with::DeserializeFromStr)
)]
#[cfg_attr(
    feature = "typescript",
    derive(ts_rs::TS),
    ts(export, export_to = "roomserver/")
)]
pub struct DeviceSecret(String);

#[cfg(feature = "utoipa")]
mod impl_utoipa {
    //! The `#[derive(utoipa::ToSchema)] implementation does not yet properly support
    //! exposing schema information of types wrapped by the NewType pattern, therefore
    //! a manual implementation is required for now.
    //! Issue: <https://github.com/juhaku/utoipa/issues/663>

    use serde_json::json;
    use utoipa::{
        PartialSchema, ToSchema,
        openapi::{ObjectBuilder, RefOr, Schema, Type},
    };

    use super::{DEVICE_SECRET_MAX_LENGTH, DEVICE_SECRET_MIN_LENGTH, DeviceSecret};
    use crate::utils::ExampleData;

    impl PartialSchema for DeviceSecret {
        fn schema() -> RefOr<Schema> {
            ObjectBuilder::new()
                .schema_type(Type::String)
                .description(Some("A device secret"))
                .min_length(Some(DEVICE_SECRET_MIN_LENGTH))
                .max_length(Some(DEVICE_SECRET_MAX_LENGTH))
                .examples([json!(DeviceSecret::example_data())])
                .into()
        }
    }

    impl ToSchema for DeviceSecret {
        fn schemas(schemas: &mut Vec<(String, RefOr<Schema>)>) {
            schemas.push((Self::name().into(), Self::schema()));
        }
    }
}

impl ExampleData for DeviceSecret {
    fn example_data() -> Self {
        Self("v3rys3cr3tD3v1ce5tr1ng".to_string())
    }
}

impl std::fmt::Debug for DeviceSecret {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_tuple("DeviceSecret")
            .field(&format_args!("********"))
            .finish()
    }
}

#[derive(Debug, Snafu)]
#[snafu(display(
    "device secret has an invalid length of {len}, it must be between {DEVICE_SECRET_MIN_LENGTH} and {DEVICE_SECRET_MAX_LENGTH}"
))]
pub struct InvalidDeviceSecretLength {
    len: usize,
}

impl FromStr for DeviceSecret {
    type Err = InvalidDeviceSecretLength;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let len = s.len();

        ensure!(
            len >= DEVICE_SECRET_MIN_LENGTH,
            InvalidDeviceSecretLengthSnafu { len }
        );
        ensure!(
            len <= DEVICE_SECRET_MAX_LENGTH,
            InvalidDeviceSecretLengthSnafu { len }
        );
        Ok(Self(s.to_string()))
    }
}

#[cfg(test)]
mod tests {
    use pretty_assertions::assert_eq;

    use super::{DeviceSecret, InvalidDeviceSecretLength};

    #[test]
    fn parse() {
        assert_eq!(
            "v3rys3cr3tD3v1ce".parse::<DeviceSecret>().unwrap(),
            DeviceSecret("v3rys3cr3tD3v1ce".to_string())
        );
        assert_eq!(
            "123456789-1234567890".parse::<DeviceSecret>().unwrap(),
            DeviceSecret("123456789-1234567890".to_string())
        );

        let longest: String = "x".repeat(255);
        assert_eq!(
            longest.parse::<DeviceSecret>().unwrap(),
            DeviceSecret(longest)
        );
    }

    #[test]
    fn parse_invalid() {
        assert!(matches!(
            "".parse::<DeviceSecret>(),
            Err(InvalidDeviceSecretLength { len: 0 })
        ));

        let too_long: String = "x".repeat(256);
        assert!(matches!(
            too_long.parse::<DeviceSecret>(),
            Err(InvalidDeviceSecretLength { len: 256 })
        ));
    }
}
