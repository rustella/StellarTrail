//! Object storage abstraction for private feedback images. Production uses S3-compatible MinIO; tests use in-memory storage.

use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
};

use async_trait::async_trait;
use aws_config::BehaviorVersion;
use aws_credential_types::Credentials;
use aws_sdk_s3::{Client, config::Region, primitives::ByteStream};

use crate::config::ObjectStorageConfig;

/// Safe metadata attached to object writes. Sensitive values and raw request data are intentionally excluded.
#[derive(Clone, Debug)]
pub struct ObjectMetadata {
    pub original_filename: String,
    pub sha256: String,
    pub image_type: String,
}

/// Result of writing an object.
#[derive(Clone, Debug)]
pub struct PutObjectResult {
    pub etag: Option<String>,
}

/// Object bytes returned after a private owner check.
#[derive(Clone, Debug)]
pub struct ObjectBytes {
    pub bytes: Vec<u8>,
    pub content_type: String,
}

/// Object storage trait used by routes and services.
#[async_trait]
pub trait ObjectStore: Send + Sync {
    async fn put_image(
        &self,
        object_key: &str,
        content_type: &str,
        bytes: Vec<u8>,
        metadata: ObjectMetadata,
    ) -> anyhow::Result<PutObjectResult>;

    async fn get_image(&self, object_key: &str) -> anyhow::Result<Option<ObjectBytes>>;

    async fn delete_image(&self, object_key: &str) -> anyhow::Result<()>;
}

/// Test-only in-memory object store.
#[derive(Clone, Default)]
pub struct InMemoryObjectStore {
    inner: Arc<Mutex<HashMap<String, StoredObject>>>,
}

/// Stored object snapshot exposed to tests for behavioral assertions.
#[derive(Clone, Debug)]
pub struct StoredObject {
    pub object_key: String,
    pub content_type: String,
    pub bytes: Vec<u8>,
    pub metadata: ObjectMetadata,
}

impl InMemoryObjectStore {
    /// Returns the number of stored objects.
    pub fn object_count(&self) -> usize {
        self.inner.lock().unwrap().len()
    }

    /// Returns the only stored object when exactly one exists.
    pub fn only_object(&self) -> Option<StoredObject> {
        let inner = self.inner.lock().unwrap();
        if inner.len() == 1 {
            inner.values().next().cloned()
        } else {
            None
        }
    }
}

#[async_trait]
impl ObjectStore for InMemoryObjectStore {
    async fn put_image(
        &self,
        object_key: &str,
        content_type: &str,
        bytes: Vec<u8>,
        metadata: ObjectMetadata,
    ) -> anyhow::Result<PutObjectResult> {
        self.inner.lock().unwrap().insert(
            object_key.to_owned(),
            StoredObject {
                object_key: object_key.to_owned(),
                content_type: content_type.to_owned(),
                bytes,
                metadata,
            },
        );
        Ok(PutObjectResult { etag: None })
    }

    async fn get_image(&self, object_key: &str) -> anyhow::Result<Option<ObjectBytes>> {
        Ok(self
            .inner
            .lock()
            .unwrap()
            .get(object_key)
            .map(|object| ObjectBytes {
                bytes: object.bytes.clone(),
                content_type: object.content_type.clone(),
            }))
    }

    async fn delete_image(&self, object_key: &str) -> anyhow::Result<()> {
        self.inner.lock().unwrap().remove(object_key);
        Ok(())
    }
}

/// S3-compatible MinIO object store.
#[derive(Clone)]
pub struct MinioObjectStore {
    client: Client,
    bucket: String,
}

impl MinioObjectStore {
    /// Builds a MinIO/S3 client from API configuration.
    pub async fn from_config(config: &ObjectStorageConfig) -> anyhow::Result<Self> {
        let credentials = Credentials::new(
            config.access_key_id.clone(),
            config.secret_access_key.clone(),
            None,
            None,
            "stellartrail-object-storage",
        );
        let shared_config = aws_config::defaults(BehaviorVersion::latest())
            .region(Region::new(config.region.clone()))
            .credentials_provider(credentials)
            .endpoint_url(config.endpoint.clone())
            .load()
            .await;
        let s3_config = aws_sdk_s3::config::Builder::from(&shared_config)
            .force_path_style(config.force_path_style)
            .build();
        Ok(Self {
            client: Client::from_conf(s3_config),
            bucket: config.bucket.clone(),
        })
    }
}

#[async_trait]
impl ObjectStore for MinioObjectStore {
    async fn put_image(
        &self,
        object_key: &str,
        content_type: &str,
        bytes: Vec<u8>,
        metadata: ObjectMetadata,
    ) -> anyhow::Result<PutObjectResult> {
        let response = self
            .client
            .put_object()
            .bucket(&self.bucket)
            .key(object_key)
            .content_type(content_type)
            .metadata("sha256", metadata.sha256)
            .metadata("image_type", metadata.image_type)
            .metadata("original_filename", metadata.original_filename)
            .body(ByteStream::from(bytes))
            .send()
            .await?;
        Ok(PutObjectResult {
            etag: response.e_tag().map(ToOwned::to_owned),
        })
    }

    async fn get_image(&self, object_key: &str) -> anyhow::Result<Option<ObjectBytes>> {
        let response = match self
            .client
            .get_object()
            .bucket(&self.bucket)
            .key(object_key)
            .send()
            .await
        {
            Ok(response) => response,
            Err(error) => {
                let message = error.to_string();
                if message.contains("NoSuchKey") || message.contains("NotFound") {
                    return Ok(None);
                }
                return Err(error.into());
            }
        };
        let content_type = response
            .content_type()
            .unwrap_or("application/octet-stream")
            .to_owned();
        let bytes = response.body.collect().await?.into_bytes().to_vec();
        Ok(Some(ObjectBytes {
            bytes,
            content_type,
        }))
    }

    async fn delete_image(&self, object_key: &str) -> anyhow::Result<()> {
        self.client
            .delete_object()
            .bucket(&self.bucket)
            .key(object_key)
            .send()
            .await?;
        Ok(())
    }
}
