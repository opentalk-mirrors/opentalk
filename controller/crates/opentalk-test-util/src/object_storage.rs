// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use opentalk_controller_settings::MinIO;
use testcontainers_modules::testcontainers::{
    ContainerAsync, GenericImage, ImageExt as _,
    core::{IntoContainerPort as _, WaitFor},
    runners::AsyncRunner as _,
};

/// Image name for the Garage container.
const GARAGE_IMAGE: &str = "dxflrs/garage";
/// Image tag for the Garage container.
const GARAGE_TAG: &str = "v2.3.0";
/// Port the S3 API is exposed on inside the container.
const GARAGE_S3_PORT: u16 = 3900;
/// S3 region configured for the Garage container. Must match `s3_region` in [`GARAGE_CONFIG`].
const GARAGE_REGION: &str = "garage";
/// Bucket created automatically on startup from `GARAGE_DEFAULT_BUCKET`.
const GARAGE_BUCKET: &str = "controller";
/// Access key created automatically on startup from `GARAGE_DEFAULT_ACCESS_KEY`.
const GARAGE_ACCESS_KEY: &str = "GK0123456789abcdef0123456789abcdef";
/// Secret key created automatically on startup from `GARAGE_DEFAULT_SECRET_KEY`.
const GARAGE_SECRET_KEY: &str = "0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef";

/// Builds the minimal single-node Garage configuration copied to `/etc/garage.toml` inside the
/// container.
///
/// Metadata and data are kept in `/tmp` because the container is thrown away after the test.
fn garage_config() -> String {
    format!(
        r#"metadata_dir = "/tmp/meta"
data_dir = "/tmp/data"
db_engine = "sqlite"

replication_factor = 1

rpc_bind_addr = "[::]:3901"
rpc_public_addr = "127.0.0.1:3901"
rpc_secret = "0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef"

[s3_api]
s3_region = "{GARAGE_REGION}"
api_bind_addr = "[::]:{GARAGE_S3_PORT}"

[admin]
api_bind_addr = "[::]:3903"
admin_token = "test-admin-token"
"#
    )
}

/// Provides an S3-compatible object storage backed by a [Garage] [testcontainer].
///
/// The container is owned by the returned [`ObjectStorageContext`] and is stopped and removed when it is dropped, so
/// callers must keep the context alive for as long as they use its storage.
///
/// [Garage]: https://garagehq.deuxfleurs.fr/
/// [testcontainer]: https://testcontainers.com/
pub struct ObjectStorageContext {
    /// Settings pointing at the Garage container.
    pub minio: MinIO,
    /// The Garage testcontainer backing this context.
    ///
    /// Dropping the [`ObjectStorageContext`] stops and removes the container.
    _container: ContainerAsync<GenericImage>,
}

impl ObjectStorageContext {
    /// Create a new [`ObjectStorageContext`].
    ///
    /// Starts a dedicated Garage [testcontainer] for this context. Because every context gets its own container, tests
    /// are isolated from each other and can run in parallel. Running the tests requires a working docker (or
    /// compatible) environment.
    ///
    /// [testcontainer]: https://testcontainers.com/
    pub async fn new() -> Self {
        let container = GenericImage::new(GARAGE_IMAGE, GARAGE_TAG)
            .with_exposed_port(GARAGE_S3_PORT.tcp())
            .with_wait_for(WaitFor::message_on_stderr("S3 API server listening on"))
            .with_entrypoint("/garage")
            .with_copy_to("/etc/garage.toml", garage_config().as_bytes().to_vec())
            .with_env_var("GARAGE_DEFAULT_ACCESS_KEY", GARAGE_ACCESS_KEY)
            .with_env_var("GARAGE_DEFAULT_SECRET_KEY", GARAGE_SECRET_KEY)
            .with_env_var("GARAGE_DEFAULT_BUCKET", GARAGE_BUCKET)
            .with_cmd(["server", "--single-node", "--default-bucket"])
            .start()
            .await
            .expect("Failed to start garage testcontainer");

        let host = container
            .get_host()
            .await
            .expect("Failed to get testcontainer host");
        let port = container
            .get_host_port_ipv4(GARAGE_S3_PORT)
            .await
            .expect("Failed to get testcontainer port");

        Self {
            minio: MinIO {
                uri: format!("http://{host}:{port}"),
                bucket: GARAGE_BUCKET.to_owned(),
                region: Some(GARAGE_REGION.to_owned()),
                force_path_style: None,
                access_key: GARAGE_ACCESS_KEY.to_owned(),
                secret_key: GARAGE_SECRET_KEY.to_owned(),
            },
            _container: container,
        }
    }
}
