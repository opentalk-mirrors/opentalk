// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

mod download_proxy_stream;

use std::{collections::BTreeMap, time::Duration};

use actix_http::error::PayloadError;
use aws_sdk_s3::{
    Client,
    config::{
        Builder, Credentials as AwsCred, Region,
        endpoint::{Endpoint, EndpointFuture, Params, ResolveEndpoint},
    },
    presigning::PresigningConfig,
    primitives::ByteStream,
    types::{CompletedMultipartUpload, CompletedPart},
};
use bytes::Bytes;
pub use download_proxy_stream::DownloadProxyStream;
use futures::{Stream, StreamExt};
use http::StatusCode;
use opentalk_controller_settings::MinIO;
use opentalk_types_api_common::{
    error::{ApiError, ErrorBody},
    pagination::Cursor,
};
use snafu::{OptionExt, ResultExt, Snafu, ensure};
use url::Url;

const CHUNK_SIZE: usize = 5_242_880; // 5 MebiByte (minimum for aws s3)

#[derive(Debug, Snafu)]
pub enum ObjectStorageError {
    #[snafu(display("Object size exceeded configured maximum ({max_size} bytes)"))]
    SizeLimitExceeded {
        max_size: usize,
    },
    #[snafu(display("{message}: {source}"))]
    InvalidSettings {
        message: String,
        source: Box<dyn std::error::Error + Send + Sync>,
    },

    InvalidResponse {
        message: String,
    },

    Upload {
        message: String,
        source: Box<dyn std::error::Error + Send + Sync>,
    },

    Put {
        message: String,
        source: Box<dyn std::error::Error + Send + Sync>,
    },

    Get {
        source: Box<dyn std::error::Error + Send + Sync>,
    },

    Delete {
        source: Box<dyn std::error::Error + Send + Sync>,
    },

    #[snafu(display("the following bucket is missing: {name}"))]
    MissingBucket {
        name: String,
    },

    #[snafu(whatever)]
    Other {
        message: String,
        #[snafu(source(from(Box<dyn std::error::Error + Send + Sync>, Some)))]
        source: Option<Box<dyn std::error::Error + Send + Sync>>,
    },
}

impl From<reqwest::Error> for ObjectStorageError {
    fn from(value: reqwest::Error) -> Self {
        Self::Other {
            message: "Reqwest error".into(),
            source: Some(value.into()),
        }
    }
}

impl From<PayloadError> for ObjectStorageError {
    fn from(value: PayloadError) -> Self {
        Self::Other {
            message: "Actix error".into(),
            source: Some(value.into()),
        }
    }
}

type Result<T, E = ObjectStorageError> = std::result::Result<T, E>;

#[derive(Debug, Snafu)]
pub enum ProxyRequestError {
    #[snafu(display("Proxy request to object storage failed"))]
    ServiceUnavailable,

    #[snafu(display("Download link has expired or is invalid"))]
    Forbidden,

    #[snafu(display("Not found"))]
    NotFound,

    #[snafu(display("Internal server error"))]
    Internal,
}

impl ProxyRequestError {
    fn status_code(&self) -> http::StatusCode {
        match self {
            ProxyRequestError::ServiceUnavailable => StatusCode::SERVICE_UNAVAILABLE,
            ProxyRequestError::Forbidden => StatusCode::FORBIDDEN,
            ProxyRequestError::NotFound => StatusCode::NOT_FOUND,
            ProxyRequestError::Internal => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }

    fn error_code(&self) -> &'static str {
        match self {
            ProxyRequestError::ServiceUnavailable => "service_unavailable",
            ProxyRequestError::Forbidden => "forbidden",
            ProxyRequestError::NotFound => "not_found",
            ProxyRequestError::Internal => "internal_server_error",
        }
    }
}

impl From<ProxyRequestError> for ApiError {
    fn from(error: ProxyRequestError) -> Self {
        Self {
            status: error.status_code(),
            www_authenticate: None,
            body: ErrorBody::new(error.error_code(), error.to_string()),
        }
    }
}

#[derive(Debug, Clone)]
pub struct ObjectStorage {
    /// The s3 client
    client: Client,

    /// The configured bucket
    bucket: String,

    /// The base url for accessing objects
    base_url: Url,

    /// A reqwest client for sending direct requests
    reqwest_client: reqwest::Client,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum ChunkFormat {
    Data,
    SequenceNumberAndData,
}

impl ChunkFormat {
    pub fn contains_sequence_number(&self) -> bool {
        matches!(self, ChunkFormat::SequenceNumberAndData)
    }
}

impl ObjectStorage {
    pub async fn new(minio: &MinIO) -> Result<Self> {
        let base_url: Url =
            minio
                .uri
                .parse()
                .map_err(Into::into)
                .context(InvalidSettingsSnafu {
                    message: "Invalid minio URI",
                })?;

        let credentials = AwsCred::new(
            minio.access_key.clone(),
            minio.secret_key.clone(),
            None,
            None,
            "opentalk",
        );

        #[derive(Debug)]
        struct Resolver {
            minio_url: Url,
        }

        impl ResolveEndpoint for Resolver {
            fn resolve_endpoint(&self, params: &Params) -> EndpointFuture<'_> {
                let url = params.bucket().map(|bucket| self.minio_url.join(bucket));
                let url = match url {
                    Some(Ok(url)) => url.to_string(),
                    Some(Err(e)) => return EndpointFuture::ready(Err(e.into())),
                    None => self.minio_url.to_string(),
                };
                let endpoint = Endpoint::builder().url(url).build();

                EndpointFuture::ready(Ok(endpoint))
            }
        }

        let region = Region::new(minio.region.clone().unwrap_or("unknown".to_string()));

        let conf = Builder::new()
            .endpoint_resolver(Resolver {
                minio_url: base_url.clone(),
            })
            .force_path_style(minio.force_path_style.unwrap_or(false))
            .credentials_provider(credentials)
            .region(region)
            .build();

        let client = Client::from_conf(conf);

        // check if the bucket exists
        ensure!(
            client
                .list_buckets()
                .send()
                .await
                .map_err(Into::into)
                .context(InvalidSettingsSnafu {
                    message: "Cannot list buckets for configured MinIO storage",
                })?
                .buckets()
                .iter()
                .any(|b| b.name() == Some(minio.bucket.as_str())),
            MissingBucketSnafu {
                name: minio.bucket.as_str(),
            },
        );

        tracing::info!("Using MinIO S3 bucket: {} ", minio.bucket,);

        Ok(Self {
            client,
            bucket: minio.bucket.clone(),
            base_url,
            reqwest_client: reqwest::Client::new(),
        })
    }

    /// Create a broken placeholder S3 client for tests
    ///
    /// The resulting [`ObjectStorage`] will error on first access. This is a placeholder until we can mock the client
    /// or have a minio test deployment.
    ///
    // TODO: create mock client or minio test deployment
    pub fn broken() -> Self {
        let credentials = AwsCred::new("broken", "broken", None, None, "broken");

        let conf = Builder::new()
            .endpoint_url("localhost")
            .credentials_provider(credentials)
            .region(Region::new(""))
            .build();

        let client = Client::from_conf(conf);

        Self {
            client,
            bucket: "broken".into(),
            base_url: "http://localhost".parse().unwrap(),
            reqwest_client: reqwest::Client::new(),
        }
    }

    /// Put an object into S3 storage
    ///
    /// Depending on the data size, this function will either use the `put_object` or `multipart_upload` S3 API call.
    ///
    /// Returns the file size of the uploaded object
    pub async fn put<E>(
        &self,
        key: &str,
        data: impl Stream<Item = Result<Bytes, E>> + Unpin,
        chunk_format: ChunkFormat,
        max_size: Option<usize>,
    ) -> Result<usize>
    where
        ObjectStorageError: From<E>,
    {
        let mut multipart_context = None;

        let res = self
            .put_inner(key, data, &mut multipart_context, chunk_format, max_size)
            .await;

        // complete or abort the multipart upload if the context exists
        if let Some(ctx) = multipart_context {
            match &res {
                Ok(_) => {
                    self.complete_multipart_upload(key, ctx).await?;
                }
                Err(_) => {
                    self.abort_multipart_upload(key, ctx).await?;
                }
            }
        }

        if let Err(err) = res.as_ref() {
            tracing::debug!("Failed to upload object to s3: {err}");
        } else {
            tracing::debug!("Uploaded object to s3");
        }

        res
    }

    async fn complete_multipart_upload(
        &self,
        key: &str,
        ctx: MultipartUploadContext,
    ) -> Result<()> {
        let parts = Vec::from_iter(ctx.parts.into_values());

        self.client
            .complete_multipart_upload()
            .bucket(&self.bucket)
            .key(key)
            .upload_id(ctx.upload_id)
            .multipart_upload(
                CompletedMultipartUpload::builder()
                    .set_parts(Some(parts))
                    .build(),
            )
            .send()
            .await
            .map_err(Into::into)
            .context(UploadSnafu {
                message: "failed to complete multipart upload",
            })?;
        Ok(())
    }

    async fn abort_multipart_upload(&self, key: &str, ctx: MultipartUploadContext) -> Result<()> {
        self.client
            .abort_multipart_upload()
            .bucket(&self.bucket)
            .key(key)
            .upload_id(ctx.upload_id)
            .send()
            .await
            .map_err(Into::into)
            .context(UploadSnafu {
                message: "failed to abort multipart upload",
            })?;
        Ok(())
    }

    async fn put_inner<E>(
        &self,
        key: &str,
        mut data: impl Stream<Item = Result<Bytes, E>> + Unpin,
        multipart_context: &mut Option<MultipartUploadContext>,
        chunk_format: ChunkFormat,
        max_size: Option<usize>,
    ) -> Result<usize>
    where
        ObjectStorageError: From<E>,
    {
        let mut file_size = 0;

        let mut part = 0;

        loop {
            let (buf, last_part) = Self::read_bytes_into_buffer(&mut data, chunk_format).await?;
            let (buf, part) = if chunk_format.contains_sequence_number() {
                let Some(Ok(part)) = buf.get(..4).map(TryInto::try_into) else {
                    break;
                };
                let part = i32::from_be_bytes(part) + 1;
                let Some(buf) = buf.get(4..) else {
                    break;
                };

                (buf.to_vec(), part)
            } else {
                part += 1;

                (buf, part)
            };

            file_size += buf.len();

            if let Some(max_size) = max_size
                && file_size > max_size
            {
                return Err(ObjectStorageError::SizeLimitExceeded { max_size });
            }

            // Check if there is only one chunk to send
            // Skip multipart API and put object directly
            let put_object = last_part && part == 1;

            if put_object {
                self.put_object(key, buf).await?;
            } else {
                self.put_part(key, multipart_context, part, buf).await?;
            }

            if last_part {
                break;
            }
        }

        Ok(file_size)
    }

    async fn read_bytes_into_buffer<E>(
        data: &mut (impl Stream<Item = Result<Bytes, E>> + Unpin + Sized),
        chunk_format: ChunkFormat,
    ) -> Result<(Vec<u8>, bool), ObjectStorageError>
    where
        ObjectStorageError: From<E>,
    {
        let mut buf = Self::initialize_buffer();
        let mut last_part = false;

        // Read chunk to upload
        loop {
            match data.next().await {
                Some(bytes) => {
                    buf.extend_from_slice(&bytes?);

                    if chunk_format.contains_sequence_number() || buf.len() >= CHUNK_SIZE {
                        break;
                    }
                }
                None => {
                    // EOS
                    last_part = true;
                    break;
                }
            }
        }
        Ok((buf, last_part))
    }

    fn initialize_buffer() -> Vec<u8> {
        // This buffer must be reallocated on each call since the aws
        // crate takes ownership and drops the buffer internally.
        Vec::with_capacity(CHUNK_SIZE * 2)
    }

    async fn put_object(&self, key: &str, buf: Vec<u8>) -> Result<()> {
        self.client
            .put_object()
            .bucket(&self.bucket)
            .key(key)
            .content_length(buf.len() as i64)
            .body(buf.into())
            .send()
            .await
            .map_err(Into::into)
            .context(PutSnafu {
                message: "failed to put object",
            })?;
        Ok(())
    }

    async fn put_part(
        &self,
        key: &str,
        multipart_context: &mut Option<MultipartUploadContext>,
        count: i32,
        buf: Vec<u8>,
    ) -> Result<()> {
        let ctx = if let Some(ctx) = multipart_context {
            ctx
        } else {
            let output = self
                .client
                .create_multipart_upload()
                .bucket(&self.bucket)
                .key(key)
                .send()
                .await
                .map_err(Into::into)
                .context(PutSnafu {
                    message: "failed to create multipart upload",
                })?;

            // initialize multipart upload lazily once there is data to upload
            multipart_context.insert(MultipartUploadContext {
                upload_id: output.upload_id.context(InvalidResponseSnafu {
                    message: "no upload_id in create_multipart_upload response",
                })?,
                parts: BTreeMap::new(),
            })
        };

        // upload a part of the multipart
        let part = self
            .client
            .upload_part()
            .bucket(&self.bucket)
            .key(key)
            .upload_id(&ctx.upload_id)
            .part_number(count)
            .content_length(buf.len() as i64)
            .body(buf.into())
            .send()
            .await
            .map_err(Into::into)
            .context(PutSnafu {
                message: "failed to upload part",
            })?;

        ctx.parts.insert(
            count,
            CompletedPart::builder()
                .e_tag(part.e_tag().context(InvalidResponseSnafu {
                    message: "missing etag in upload_part response",
                })?)
                .part_number(count)
                .build(),
        );
        Ok(())
    }

    pub async fn get(&self, key: String) -> Result<ByteStream> {
        let data = self
            .client
            .get_object()
            .bucket(&self.bucket)
            .key(key)
            .send()
            .await
            .map_err(Into::into)
            .context(GetSnafu)?;

        Ok(data.body)
    }

    pub async fn get_object_size_if_exists(&self, key: String) -> Result<Option<i64>> {
        let response = self
            .client
            .head_object()
            .bucket(&self.bucket)
            .key(key)
            .send()
            .await
            .map_err(|e| e.into_service_error());

        match response {
            Ok(head_object) => Ok(Some(head_object.content_length.unwrap_or(0))),
            Err(e) => {
                if e.is_not_found() {
                    return Ok(None);
                }

                Err(ObjectStorageError::Get { source: e.into() })
            }
        }
    }

    pub async fn delete(&self, key: String) -> Result<()> {
        self.client
            .delete_object()
            .bucket(&self.bucket)
            .key(key)
            .send()
            .await
            .map_err(Into::into)
            .context(DeleteSnafu)?;

        Ok(())
    }

    pub fn get_base_url(&self, key: &str) -> Url {
        let mut url = self.base_url.clone();

        let path = format!("{}/{}", self.bucket, key);
        url.set_path(&path);

        url
    }

    pub async fn get_proxy_download_token(
        &self,
        key: &str,
        expires_in: Duration,
        content_disposition: String,
    ) -> Result<String> {
        let get = self
            .client
            .get_object()
            .bucket(&self.bucket)
            .key(key)
            .response_content_disposition(content_disposition);

        let presigned = get
            .presigned(PresigningConfig::expires_in(expires_in).map_err(|e| {
                ObjectStorageError::Other {
                    message: "Failed to build presigning config".into(),
                    source: Some(Box::new(e)),
                }
            })?)
            .await
            .map_err(|e| ObjectStorageError::Other {
                message: "Failed to presign get_object".into(),
                source: Some(Box::new(e)),
            })?;

        let url: Url = presigned
            .uri()
            .parse()
            .whatever_context::<&str, ObjectStorageError>(
                "Received an invalid URL from the pre-signing request",
            )?;
        // Preserve the exact signed query string
        let raw_query = url.query().unwrap_or_default().to_owned();

        Ok(Cursor(raw_query).to_base64())
    }

    pub async fn get_proxied(
        &self,
        key: &str,
        token: String,
        range_header: Option<String>,
    ) -> Result<DownloadProxyStream, ProxyRequestError> {
        let Cursor(raw_query): Cursor<String> =
            Cursor::from_base64(&token).map_err(|_| ForbiddenSnafu.build())?;

        let mut url = self.get_base_url(key);
        url.set_query(Some(&raw_query));

        let mut request = self.reqwest_client.get(url);

        if let Some(range) = range_header
            && let Ok(v) = reqwest::header::HeaderValue::from_bytes(range.as_bytes())
        {
            request = request.header(reqwest::header::RANGE, v);
        }
        let response = request.send().await.map_err(|e| {
            tracing::error!("Proxy request to object storage failed: {e}");
            ServiceUnavailableSnafu {}.build()
        })?;

        let status = response.status();

        if !status.is_success() {
            return Err(match status {
                reqwest::StatusCode::FORBIDDEN | reqwest::StatusCode::UNAUTHORIZED => {
                    ForbiddenSnafu.build()
                }
                reqwest::StatusCode::NOT_FOUND => NotFoundSnafu.build(),
                _ => InternalSnafu.build(),
            });
        }

        const HEADERS_TO_FORWARD: [&str; 5] = [
            "content-type",
            "content-disposition",
            "content-length",
            "accept-ranges",
            "content-range",
        ];
        let headers = HEADERS_TO_FORWARD
            .iter()
            .filter_map(|&k| {
                response
                    .headers()
                    .get(k)
                    .map(|v| (k.to_string(), Bytes::copy_from_slice(v.as_bytes())))
            })
            .collect();

        let stream = response.bytes_stream().map(|item| {
            item.map_err(|e| {
                let b: Box<dyn std::error::Error + Send + Sync> = Box::new(e);
                b
            })
        });
        Ok(DownloadProxyStream {
            status: status.as_u16(),
            headers,
            stream: Box::new(stream),
        })
    }
}

struct MultipartUploadContext {
    upload_id: String,
    parts: BTreeMap<i32, CompletedPart>,
}

#[cfg(test)]
mod tests {
    use std::{
        sync::{Arc, Mutex},
        time::Duration,
    };

    use axum::{Router, extract::RawQuery, routing::get};
    use tokio::net::TcpListener;

    use super::{AwsCred, Builder, Client, ObjectStorage, Region};

    /// Build an [`ObjectStorage`] whose S3 endpoint and object base URL both point
    /// at the given mock server. Presigning is performed offline, so real
    /// credentials are not required.
    fn storage_pointing_at(mock_base_url: &str) -> ObjectStorage {
        let credentials = AwsCred::new("test", "test", None, None, "test");
        let conf = Builder::new()
            .endpoint_url(mock_base_url)
            .force_path_style(true)
            .credentials_provider(credentials)
            .region(Region::new("unknown"))
            .build();

        ObjectStorage {
            client: Client::from_conf(conf),
            bucket: "test-bucket".into(),
            base_url: mock_base_url.parse().expect("valid mock url"),
            reqwest_client: reqwest::Client::new(),
        }
    }

    #[tokio::test]
    async fn proxied_request_forwards_signed_query_raw() {
        // Shared state to capture the raw query string received by the mock server
        let captured = Arc::new(Mutex::new(None));

        let app = Router::new().route(
            "/{*path}",
            get({
                let captured = captured.clone();
                move |RawQuery(query): RawQuery| async move {
                    *captured.lock().expect("lock not poisoned") = query;
                    "ok"
                }
            }),
        );

        let listener = TcpListener::bind("127.0.0.1:0")
            .await
            .expect("bind mock server");
        let addr = listener.local_addr().expect("local addr");
        tokio::spawn(async move { axum::serve(listener, app).await.expect("serve mock") });

        let storage = storage_pointing_at(&format!("http://{addr}"));

        // Generate the token (using a filename with spaces to trigger %20 encoding)
        let token = storage
            .get_proxy_download_token(
                "assets/test",
                Duration::from_secs(30),
                "attachment; filename=\"my recording.mkv\"".to_owned(),
            )
            .await
            .expect("generate download token");

        // Execute the proxy request against our mock server
        let _stream = storage
            .get_proxied("assets/test", token, None)
            .await
            .expect("proxied request succeeds");

        // Verify exactly what was transmitted
        let received = captured
            .lock()
            .expect("lock not poisoned")
            .take()
            .expect("mock captured a query");

        assert!(
            received.contains("%20"),
            "query must contain a percent-encoded space: {received}"
        );
        assert!(
            !received.contains('+'),
            "space must not be re-encoded as `+`: {received}"
        );
    }
}
