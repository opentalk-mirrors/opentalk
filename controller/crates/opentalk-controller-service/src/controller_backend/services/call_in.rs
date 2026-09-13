// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use std::str::FromStr;

use opentalk_controller_utils::{CaptureApiError, FeatureRequiredError};
use opentalk_inventory::{
    Room, User,
    utils::{self, CallInUnavailable},
};
use opentalk_roomserver_types::client_parameters::{ClientKind, ClientParameters, Role};
use opentalk_types_api_internal::call_in::PostCallInStartRoomServerRequestBody;
use opentalk_types_api_v1::{
    error::ApiError,
    rooms::{RoomResource, by_room_id::RoomserverStartResponseBody},
    services::call_in::PostCallInStartRequestBody,
};
use opentalk_types_common::{
    rooms::GuestAccess,
    roomserver::{DEVICE_SECRET_MIN_LENGTH, DeviceSecret},
    tariffs::TariffResource,
};
use rand::RngExt;
use url::Url;

use crate::{ControllerBackend, ToUserProfile};

impl ControllerBackend {
    async fn check_call_in_request(
        &self,
        request: PostCallInStartRequestBody,
    ) -> Result<(Room, User), CaptureApiError> {
        let mut inventory = self.inventory_provider.get_inventory().await?;

        let (sip_config, room, creator) = inventory
            .get_room_sip_config_with_room_and_creator(request.id)
            .await?
            .ok_or_else(invalid_credentials_error)?;

        let tariff = self.get_tariff_for_user(room.created_by).await?;
        Self::ensure_call_in_permission(room.e2e_encryption, room.guest_access, &tariff)?;

        if sip_config.password != request.pin {
            return Err(invalid_credentials_error().into());
        }

        Ok((room, creator))
    }

    /// Maps a [`CallInUnavailable`] reason to the default `403 Forbidden` API
    /// error.
    ///
    /// The resulting error codes and messages are part of the public API
    /// contract of the endpoints performing call-in checks.
    pub(crate) fn call_in_unavailable_error(reason: CallInUnavailable) -> ApiError {
        match reason {
            CallInUnavailable::E2eEnabled => ApiError::forbidden()
                .with_code("service_unavailable")
                .with_message("call-in is not available for encrypted rooms"),
            CallInUnavailable::GuestAccessDisabled => ApiError::forbidden()
                .with_code("service_unavailable")
                .with_message("call-in is not available for rooms without guest access"),
            CallInUnavailable::FeatureDisabled(feature) => FeatureRequiredError { feature }.into(),
        }
    }

    /// Returns an error if call-in is not available for the given room
    /// configuration and tariff, mapping every failure to a `403 Forbidden`.
    ///
    /// Callers that need a different HTTP status for a specific reason should
    /// use [`check_call_in`](utils::check_call_in) and map the
    /// [`CallInUnavailable`] reason themselves.
    pub fn ensure_call_in_permission(
        e2e_encryption: bool,
        guest_access: GuestAccess,
        tariff: &TariffResource,
    ) -> Result<(), ApiError> {
        utils::check_call_in(e2e_encryption, guest_access, tariff)
            .map_err(Self::call_in_unavailable_error)
    }

    pub(crate) async fn start_call_in_roomserver_impl(
        &self,
        request: PostCallInStartRoomServerRequestBody,
        host: Url,
    ) -> Result<RoomserverStartResponseBody, CaptureApiError> {
        let settings = self.settings_provider.get();

        let (room, creator) = self
            .check_call_in_request(PostCallInStartRequestBody {
                id: request.id.clone(),
                pin: request.pin.clone(),
            })
            .await?;

        let room_resource = RoomResource {
            id: room.id,
            created_by: creator.to_public_user_profile(&settings),
            created_at: room.created_at,
            password: room.password,
            guest_access: room.guest_access,
            waiting_room: room.waiting_room,
            alias: room.alias,
        };

        let device_secret = rand::rng()
            .sample_iter(rand::distr::Alphanumeric)
            .take(DEVICE_SECRET_MIN_LENGTH)
            .map(|c| c as char)
            .collect::<String>();
        let device_secret = DeviceSecret::from_str(&device_secret)
            .expect("String is at least DEVICE_SECRET_MIN_LENGTH long");

        let client_parameters = ClientParameters {
            device_secret,
            kind: ClientKind::CallIn,
            role: Role::User,
        };

        let mut inventory = self.inventory_provider.get_inventory().await?;
        let access = self
            .roomserver
            .request_access(
                inventory.as_mut(),
                settings,
                self.module_features.clone(),
                room_resource,
                client_parameters,
                host,
            )
            .await?;

        Ok(RoomserverStartResponseBody {
            token: access.token,
            roomserver_address: access.public_url.to_string(),
        })
    }
}

fn invalid_credentials_error() -> ApiError {
    ApiError::bad_request()
        .with_code("invalid_credentials")
        .with_message("given call-in id & pin combination is not valid")
}

#[cfg(test)]
mod tests {
    use std::collections::{BTreeMap, BTreeSet};

    use http::StatusCode;
    use opentalk_types_common::{
        features::{CALL_IN_MODULE_FEATURE_ID, GUESTS_ALLOWED_MODULE_FEATURE_ID},
        rooms::GuestAccess,
        tariffs::{TariffId, TariffModuleResource, TariffResource},
    };

    use super::ControllerBackend;

    fn allowed_tariff() -> TariffResource {
        // Both features live in the same module (`core`), so the entries must be
        // merged instead of collected directly into a map, where a shared key
        // would cause one feature to overwrite the other.
        let mut modules = BTreeMap::<_, TariffModuleResource>::new();
        let _ = modules
            .entry(CALL_IN_MODULE_FEATURE_ID.module)
            .or_default()
            .features
            .insert(CALL_IN_MODULE_FEATURE_ID.feature);
        let _ = modules
            .entry(GUESTS_ALLOWED_MODULE_FEATURE_ID.module)
            .or_default()
            .features
            .insert(GUESTS_ALLOWED_MODULE_FEATURE_ID.feature);

        TariffResource {
            id: TariffId::nil(),
            name: "Allowed".to_owned(),
            quotas: BTreeMap::new(),
            modules,
        }
    }

    #[test]
    fn ensure_call_in_permission_allowed() {
        let allowed_tariff = allowed_tariff();

        ControllerBackend::ensure_call_in_permission(
            false,
            GuestAccess::DirectAccess,
            &allowed_tariff,
        )
        .unwrap();
        ControllerBackend::ensure_call_in_permission(
            false,
            GuestAccess::WaitingRoom,
            &allowed_tariff,
        )
        .unwrap();
    }

    #[test]
    fn ensure_call_in_permission_feature_disabled() {
        let call_in_disabled_tariff = TariffResource {
            id: TariffId::nil(),
            name: "Guests disabled".to_owned(),
            quotas: BTreeMap::new(),
            modules: BTreeMap::from_iter([(
                GUESTS_ALLOWED_MODULE_FEATURE_ID.module,
                TariffModuleResource {
                    features: BTreeSet::from([GUESTS_ALLOWED_MODULE_FEATURE_ID.feature]),
                },
            )]),
        };

        let err = ControllerBackend::ensure_call_in_permission(
            false,
            GuestAccess::DirectAccess,
            &call_in_disabled_tariff,
        )
        .unwrap_err();
        assert_eq!(err.status, StatusCode::FORBIDDEN);
        assert_eq!(err.body.code, "feature_disabled");

        let call_in_feature_disabled_tariff = TariffResource {
            id: TariffId::nil(),
            name: "Call-in disabled".to_owned(),
            quotas: BTreeMap::new(),
            modules: BTreeMap::from_iter([(
                CALL_IN_MODULE_FEATURE_ID.module,
                TariffModuleResource::default(),
            )]),
        };
        let err = ControllerBackend::ensure_call_in_permission(
            false,
            GuestAccess::DirectAccess,
            &call_in_feature_disabled_tariff,
        )
        .unwrap_err();
        assert_eq!(err.status, StatusCode::FORBIDDEN);
        assert_eq!(err.body.code, "feature_disabled");
    }

    #[test]
    fn ensure_call_in_permission_e2ee() {
        let allowed_tariff = allowed_tariff();

        let err = ControllerBackend::ensure_call_in_permission(
            true,
            GuestAccess::DirectAccess,
            &allowed_tariff,
        )
        .unwrap_err();
        assert_eq!(err.status, StatusCode::FORBIDDEN);
        assert_eq!(err.body.code, "service_unavailable");
    }

    #[test]
    fn require_call_in_guest_access() {
        let allowed_tariff = allowed_tariff();

        let err = ControllerBackend::ensure_call_in_permission(
            false,
            GuestAccess::Disabled,
            &allowed_tariff,
        )
        .unwrap_err();
        assert_eq!(err.status, StatusCode::FORBIDDEN);
        assert_eq!(err.body.code, "service_unavailable");
    }
}
