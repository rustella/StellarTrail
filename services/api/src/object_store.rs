//! Object storage abstraction for private feedback images and public skill media. Production uses S3-compatible MinIO; tests use in-memory storage.

use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
};

use async_trait::async_trait;
use aws_config::BehaviorVersion;
use aws_credential_types::Credentials;
use aws_sdk_s3::{Client, config::Region, primitives::ByteStream};

use crate::config::MinioConfig;

/// Safe metadata attached to object writes. Sensitive values and raw request data are intentionally excluded.
#[derive(Clone, Debug)]
pub struct ObjectMetadata {
    pub original_filename: String,
    pub sha256: String,
    pub image_type: String,
}

impl ObjectMetadata {
    fn into_map(self) -> HashMap<String, String> {
        HashMap::from([
            ("original_filename".to_owned(), self.original_filename),
            ("sha256".to_owned(), self.sha256),
            ("image_type".to_owned(), self.image_type),
        ])
    }
}

/// Generic object write request for feedback images and Knots3D media.
#[derive(Clone, Debug)]
pub struct PutObjectRequest {
    pub bucket: Option<String>,
    pub object_key: String,
    pub content_type: String,
    pub bytes: Vec<u8>,
    pub metadata: HashMap<String, String>,
    pub cache_control: Option<String>,
}

/// Result of writing an object.
#[derive(Clone, Debug)]
pub struct PutObjectResult {
    pub etag: Option<String>,
    pub size_bytes: u64,
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
    async fn put_object(&self, request: PutObjectRequest) -> anyhow::Result<PutObjectResult>;

    async fn get_object(&self, object_key: &str) -> anyhow::Result<Option<ObjectBytes>>;

    async fn delete_object(&self, object_key: &str) -> anyhow::Result<()>;

    async fn put_image(
        &self,
        object_key: &str,
        content_type: &str,
        bytes: Vec<u8>,
        metadata: ObjectMetadata,
    ) -> anyhow::Result<PutObjectResult> {
        self.put_object(PutObjectRequest {
            bucket: None,
            object_key: object_key.to_owned(),
            content_type: content_type.to_owned(),
            bytes,
            metadata: metadata.into_map(),
            cache_control: None,
        })
        .await
    }

    async fn get_image(&self, object_key: &str) -> anyhow::Result<Option<ObjectBytes>> {
        self.get_object(object_key).await
    }

    async fn delete_image(&self, object_key: &str) -> anyhow::Result<()> {
        self.delete_object(object_key).await
    }
}

/// Test-only in-memory object store.
#[derive(Clone, Default)]
pub struct InMemoryObjectStore {
    inner: Arc<Mutex<HashMap<String, StoredObject>>>,
}

/// Stored object snapshot exposed to tests for behavioral assertions.
#[derive(Clone, Debug)]
pub struct StoredObject {
    pub bucket: Option<String>,
    pub object_key: String,
    pub content_type: String,
    pub bytes: Vec<u8>,
    pub metadata: HashMap<String, String>,
    pub cache_control: Option<String>,
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
    async fn put_object(&self, request: PutObjectRequest) -> anyhow::Result<PutObjectResult> {
        let size_bytes = request.bytes.len() as u64;
        let storage_key = match &request.bucket {
            Some(bucket) => format!("{bucket}/{}", request.object_key),
            None => request.object_key.clone(),
        };
        self.inner.lock().unwrap().insert(
            storage_key,
            StoredObject {
                bucket: request.bucket,
                object_key: request.object_key,
                content_type: request.content_type,
                bytes: request.bytes,
                metadata: request.metadata,
                cache_control: request.cache_control,
            },
        );
        Ok(PutObjectResult {
            etag: None,
            size_bytes,
        })
    }

    async fn get_object(&self, object_key: &str) -> anyhow::Result<Option<ObjectBytes>> {
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

    async fn delete_object(&self, object_key: &str) -> anyhow::Result<()> {
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
    /// Builds a MinIO/S3 client from shared connection configuration and a default business bucket.
    pub async fn from_config(config: &MinioConfig, default_bucket: &str) -> anyhow::Result<Self> {
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
            bucket: default_bucket.to_owned(),
        })
    }
}

#[async_trait]
impl ObjectStore for MinioObjectStore {
    async fn put_object(&self, request: PutObjectRequest) -> anyhow::Result<PutObjectResult> {
        let size_bytes = request.bytes.len() as u64;
        let bucket = request.bucket.unwrap_or_else(|| self.bucket.clone());
        let mut put = self
            .client
            .put_object()
            .bucket(bucket)
            .key(request.object_key)
            .content_type(request.content_type)
            .body(ByteStream::from(request.bytes));
        if let Some(cache_control) = request.cache_control {
            put = put.cache_control(cache_control);
        }
        for (key, value) in request.metadata {
            put = put.metadata(key, value);
        }
        let response = put.send().await?;
        Ok(PutObjectResult {
            etag: response.e_tag().map(ToOwned::to_owned),
            size_bytes,
        })
    }

    async fn get_object(&self, object_key: &str) -> anyhow::Result<Option<ObjectBytes>> {
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

    async fn delete_object(&self, object_key: &str) -> anyhow::Result<()> {
        self.client
            .delete_object()
            .bucket(&self.bucket)
            .key(object_key)
            .send()
            .await?;
        Ok(())
    }
}
