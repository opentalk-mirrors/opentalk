// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use opentalk_types_common::{events::EventId, utils::ExampleData};

use super::InstanceId;

/// Opaque id of an EventInstance or EventException resource. Should only be used to sort/index the related resource.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct EventAndInstanceId(pub EventId, pub InstanceId);

#[cfg(feature = "serde")]
mod serde_impls {
    use serde::{Deserialize, Deserializer, Serialize, Serializer, de::Error};

    use super::*;

    impl Serialize for EventAndInstanceId {
        fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
        where
            S: Serializer,
        {
            format!("{}_{}", self.0, self.1).serialize(serializer)
        }
    }

    impl<'de> Deserialize<'de> for EventAndInstanceId {
        fn deserialize<D>(deserializer: D) -> Result<EventAndInstanceId, D::Error>
        where
            D: Deserializer<'de>,
        {
            let s = String::deserialize(deserializer)?;
            let mut split = s.split('_');
            let event_id = split
                .next()
                .ok_or_else(|| D::Error::custom("missing event id"))?;
            let instance_id_str = split
                .next()
                .ok_or_else(|| D::Error::custom("missing instance id"))?;
            if split.next().is_some() {
                return Err(D::Error::custom("too many parts"));
            }

            let instance_id = instance_id_str.parse().map_err(D::Error::custom)?;

            let event_id = event_id.parse().map_err(D::Error::custom)?;

            Ok(EventAndInstanceId(event_id, instance_id))
        }
    }
}

impl ExampleData for EventAndInstanceId {
    fn example_data() -> Self {
        Self(EventId::example_data(), InstanceId::example_data())
    }
}

#[cfg(feature = "utoipa")]
mod impl_utoipa {
    use serde_json::json;
    use utoipa::{
        PartialSchema, ToSchema,
        openapi::{ObjectBuilder, RefOr, Schema, Type},
    };

    use super::{EventAndInstanceId, ExampleData as _};

    impl PartialSchema for EventAndInstanceId {
        fn schema() -> RefOr<Schema> {
            ObjectBuilder::new()
                .schema_type(Type::String)
                .description(Some("An event id and an instance id"))
                .examples([json!(EventAndInstanceId::example_data())])
                .into()
        }
    }

    impl ToSchema for EventAndInstanceId {
        fn schemas(schemas: &mut Vec<(String, RefOr<Schema>)>) {
            schemas.push((Self::name().into(), Self::schema()));
        }
    }
}

#[cfg(all(test, feature = "serde"))]
mod serde_tests {
    use opentalk_types_common::{events::EventId, utils::ExampleData as _};
    use pretty_assertions::assert_eq;
    use serde_json::json;

    use crate::events::{EventAndInstanceId, InstanceId};

    #[test]
    fn roundtrip() {
        let value = EventAndInstanceId(EventId::from_u128(0x1234), InstanceId::example_data());

        let serialized = serde_json::to_value(&value).unwrap();
        assert_eq!(
            serialized,
            json!("00000000-0000-0000-0000-000000001234_20240705T170242Z")
        );

        let deserialized: EventAndInstanceId = serde_json::from_value(serialized).unwrap();

        assert_eq!(value, deserialized);
    }
}
