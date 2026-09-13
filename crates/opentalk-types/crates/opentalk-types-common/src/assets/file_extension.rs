// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use std::str::FromStr;

use snafu::Snafu;

use crate::utils::ExampleData;

/// The maximum allowed length for valid file extensions
pub const FILE_EXTENSION_MAX_LENGTH: usize = 10;

/// The extension of a filename.
///
/// Can be parsed using [`std::str::FromStr`].
/// May contain alphanumeric ascii characters only, and is restricted to a
/// length of [`FILE_EXTENSION_MAX_LENGTH`] characters. This is an arbitrary
/// decision based on regular usage of filename extensions as seen commonly
/// used. This serves as sanitization measure for user input.
#[derive(Debug, Clone, Default, PartialEq, Eq, derive_more::Display)]
#[cfg_attr(
    feature = "serde",
    derive(serde::Serialize, serde_with::DeserializeFromStr)
)]
pub struct FileExtension(String);

impl FileExtension {
    /// Extracts a string slice containing the file extension.
    pub fn as_str(&self) -> &str {
        self.0.as_str()
    }

    /// Return the file extension including a leading dot.
    ///
    /// This is useful for appending it to a file stem without having
    /// to care about placing the dot depending on whether the extension
    /// is empty.
    ///
    /// ```
    /// # use opentalk_types_common::assets::FileExtension;
    /// let file_stem = "hello-world";
    /// let file_extension: FileExtension = "abc".parse().unwrap();
    /// let file_name = format!("{}{}", file_stem, file_extension.to_string_with_leading_dot());
    /// assert_eq!(file_name.as_str(), "hello-world.abc");
    ///
    /// let file_stem = "hello-world";
    /// let file_extension: FileExtension = "".parse().unwrap();
    /// let file_name = format!("{}{}", file_stem, file_extension.to_string_with_leading_dot());
    /// assert_eq!(file_name.as_str(), "hello-world");
    /// ```
    pub fn to_string_with_leading_dot(&self) -> String {
        if self.0.is_empty() {
            String::new()
        } else {
            format!(".{}", self.0)
        }
    }

    /// Predefined extension for PDF files
    pub fn pdf() -> Self {
        Self("pdf".to_string())
    }
}

#[derive(Debug, Snafu)]
pub enum ParseFileExtensionError {
    #[snafu(display("FileExtension must not be longer than {max_length} characters"))]
    TooLong { max_length: usize },

    #[snafu(display("FileExtension only allows alphanumeric characters"))]
    InvalidCharacters,
}

impl FromStr for FileExtension {
    type Err = ParseFileExtensionError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if s.len() > FILE_EXTENSION_MAX_LENGTH {
            return Err(ParseFileExtensionError::TooLong {
                max_length: FILE_EXTENSION_MAX_LENGTH,
            });
        }
        if s.chars().any(|c| !c.is_ascii_alphanumeric()) {
            return Err(ParseFileExtensionError::InvalidCharacters);
        }
        Ok(FileExtension(s.into()))
    }
}

#[cfg(feature = "utoipa")]
mod impl_utoipa {
    use serde_json::json;
    use utoipa::{
        PartialSchema, ToSchema,
        openapi::{ObjectBuilder, RefOr, Schema, Type},
    };

    use super::{FILE_EXTENSION_MAX_LENGTH, FileExtension};
    use crate::utils::ExampleData;

    impl PartialSchema for FileExtension {
        fn schema() -> RefOr<Schema> {
            ObjectBuilder::new()
                .schema_type(Type::String)
                .max_length(Some(FILE_EXTENSION_MAX_LENGTH))
                .pattern(Some("^[0-9a-zA-Z]*$".to_string()))
                .description(Some("An extension for a file path"))
                .examples([json!(FileExtension::example_data())])
                .into()
        }
    }

    impl ToSchema for FileExtension {
        fn schemas(schemas: &mut Vec<(String, RefOr<Schema>)>) {
            schemas.push((Self::name().into(), Self::schema()));
        }
    }
}

impl ExampleData for FileExtension {
    fn example_data() -> Self {
        Self::pdf()
    }
}

#[cfg(test)]
mod tests {
    use pretty_assertions::assert_eq;

    use super::FileExtension;

    #[test]
    fn file_extension_to_string_with_leading_dot() {
        assert_eq!(
            "".to_string(),
            FileExtension("".to_string()).to_string_with_leading_dot()
        );

        assert_eq!(
            ".7z".to_string(),
            FileExtension("7z".to_string()).to_string_with_leading_dot()
        );
        assert_eq!(
            ".txt".to_string(),
            FileExtension("txt".to_string()).to_string_with_leading_dot()
        );
    }
}

#[cfg(all(test, feature = "serde"))]
mod serde_tests {
    use pretty_assertions::assert_eq;
    use serde_json::json;

    use super::FileExtension;

    #[test]
    fn file_extension_deserialization() {
        assert_eq!(
            FileExtension("".to_string()),
            serde_json::from_value::<FileExtension>(json!("")).unwrap()
        );
        assert_eq!(
            FileExtension("hello".to_string()),
            serde_json::from_value::<FileExtension>(json!("hello")).unwrap()
        );
        assert_eq!(
            FileExtension("HeLlOwOrLd".to_string()),
            serde_json::from_value::<FileExtension>(json!("HeLlOwOrLd")).unwrap()
        );
        assert_eq!(
            FileExtension("1337".to_string()),
            serde_json::from_value::<FileExtension>(json!("1337")).unwrap()
        );
        assert_eq!(
            FileExtension("7z".to_string()),
            serde_json::from_value::<FileExtension>(json!("7z")).unwrap()
        );
        assert!(serde_json::from_value::<FileExtension>(json!("HeLlOnIcEwOrLd")).is_err());
        assert!(serde_json::from_value::<FileExtension>(json!("hi world")).is_err());
        assert!(serde_json::from_value::<FileExtension>(json!("Hello!")).is_err());
        assert!(serde_json::from_value::<FileExtension>(json!("nice.try")).is_err());
        assert!(serde_json::from_value::<FileExtension>(json!("世界您好")).is_err());
    }
}
