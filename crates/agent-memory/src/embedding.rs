use std::collections::BTreeSet;

use async_trait::async_trait;
use serde::{Deserialize, Serialize};

use crate::{MemoryError, MemorySensitivity};

const FNV_OFFSET: u64 = 0xcbf29ce484222325;
const FNV_PRIME: u64 = 0x100000001b3;

#[async_trait]
pub trait EmbeddingProvider: Send + Sync {
    async fn embed(&self, request: EmbeddingRequest) -> Result<EmbeddingBatch, MemoryError>;
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct EmbeddingRequest {
    pub provider_id: String,
    pub model_id: String,
    pub inputs: Vec<EmbeddingInput>,
    pub expected_dimensions: Option<usize>,
}

impl EmbeddingRequest {
    pub fn new(
        provider_id: impl Into<String>,
        model_id: impl Into<String>,
        inputs: Vec<EmbeddingInput>,
        expected_dimensions: Option<usize>,
    ) -> Result<Self, MemoryError> {
        let provider_id = non_empty(provider_id, "embedding provider id")?;
        let model_id = non_empty(model_id, "embedding model id")?;
        if inputs.is_empty() {
            return Err(invalid_embedding("embedding input batch cannot be empty"));
        }
        if expected_dimensions.is_some_and(|dimensions| dimensions == 0) {
            return Err(invalid_embedding(
                "embedding expected dimensions must be greater than zero",
            ));
        }
        ensure_unique_input_ids(&inputs)?;

        Ok(Self {
            provider_id,
            model_id,
            inputs,
            expected_dimensions,
        })
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct EmbeddingInput {
    pub record_id: String,
    pub text: String,
    pub sensitivity: MemorySensitivity,
}

impl EmbeddingInput {
    pub fn new(
        record_id: impl Into<String>,
        text: impl Into<String>,
        sensitivity: MemorySensitivity,
    ) -> Result<Self, MemoryError> {
        Ok(Self {
            record_id: non_empty(record_id, "embedding record id")?,
            text: non_empty(text, "embedding input text")?,
            sensitivity,
        })
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct EmbeddingBatch {
    pub vectors: Vec<EmbeddingVector>,
}

impl EmbeddingBatch {
    pub fn new(vectors: Vec<EmbeddingVector>) -> Result<Self, MemoryError> {
        if vectors.is_empty() {
            return Err(invalid_embedding("embedding vector batch cannot be empty"));
        }
        let mut seen = BTreeSet::new();
        for vector in &vectors {
            if !seen.insert(vector.record_id.clone()) {
                return Err(invalid_embedding(format!(
                    "duplicate embedding vector record id: {}",
                    redact_secret_like_text(&vector.record_id)
                )));
            }
        }
        Ok(Self { vectors })
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct EmbeddingVector {
    pub record_id: String,
    pub values: Vec<f32>,
    pub metadata: EmbeddingMetadata,
}

impl EmbeddingVector {
    pub fn new(
        record_id: impl Into<String>,
        values: Vec<f32>,
        metadata: EmbeddingMetadata,
    ) -> Result<Self, MemoryError> {
        let record_id = non_empty(record_id, "embedding vector record id")?;
        if values.len() != metadata.dimensions {
            return Err(invalid_embedding(format!(
                "embedding vector dimension mismatch: expected {}, got {}",
                metadata.dimensions,
                values.len()
            )));
        }
        if values.iter().any(|value| !value.is_finite()) {
            return Err(invalid_embedding(
                "embedding vector values must be finite numbers",
            ));
        }

        Ok(Self {
            record_id,
            values,
            metadata,
        })
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct EmbeddingMetadata {
    pub provider_id: String,
    pub model_id: String,
    pub dimensions: usize,
    pub content_hash: String,
    pub sensitivity: MemorySensitivity,
}

impl EmbeddingMetadata {
    pub fn new(
        provider_id: impl Into<String>,
        model_id: impl Into<String>,
        dimensions: usize,
        content_hash: impl Into<String>,
        sensitivity: MemorySensitivity,
    ) -> Result<Self, MemoryError> {
        if dimensions == 0 {
            return Err(invalid_embedding(
                "embedding dimensions must be greater than zero",
            ));
        }
        Ok(Self {
            provider_id: non_empty(provider_id, "embedding metadata provider id")?,
            model_id: non_empty(model_id, "embedding metadata model id")?,
            dimensions,
            content_hash: non_empty(content_hash, "embedding content hash")?,
            sensitivity,
        })
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DeterministicEmbeddingProvider {
    provider_id: String,
    model_id: String,
    dimensions: usize,
}

impl DeterministicEmbeddingProvider {
    pub fn new(
        provider_id: impl Into<String>,
        model_id: impl Into<String>,
        dimensions: usize,
    ) -> Result<Self, MemoryError> {
        if dimensions == 0 {
            return Err(invalid_embedding(
                "embedding dimensions must be greater than zero",
            ));
        }
        Ok(Self {
            provider_id: non_empty(provider_id, "embedding provider id")?,
            model_id: non_empty(model_id, "embedding model id")?,
            dimensions,
        })
    }
}

#[async_trait]
impl EmbeddingProvider for DeterministicEmbeddingProvider {
    async fn embed(&self, request: EmbeddingRequest) -> Result<EmbeddingBatch, MemoryError> {
        if request.provider_id != self.provider_id {
            return Err(invalid_embedding("embedding provider id does not match"));
        }
        if request.model_id != self.model_id {
            return Err(invalid_embedding("embedding model id does not match"));
        }
        if request
            .expected_dimensions
            .is_some_and(|dimensions| dimensions != self.dimensions)
        {
            return Err(invalid_embedding(format!(
                "embedding expected dimension mismatch: expected {}, provider has {}",
                request.expected_dimensions.unwrap_or_default(),
                self.dimensions
            )));
        }

        let vectors = request
            .inputs
            .into_iter()
            .map(|input| self.embed_input(input))
            .collect::<Result<Vec<_>, _>>()?;
        EmbeddingBatch::new(vectors)
    }
}

impl DeterministicEmbeddingProvider {
    fn embed_input(&self, input: EmbeddingInput) -> Result<EmbeddingVector, MemoryError> {
        let content_hash = stable_hash_hex(&input.text);
        let metadata = EmbeddingMetadata::new(
            self.provider_id.clone(),
            self.model_id.clone(),
            self.dimensions,
            content_hash,
            input.sensitivity,
        )?;
        let values = (0..self.dimensions)
            .map(|index| {
                hash_to_unit(stable_hash_parts(&[
                    &self.provider_id,
                    &self.model_id,
                    &input.record_id,
                    &input.text,
                    &index.to_string(),
                ]))
            })
            .collect::<Vec<_>>();
        EmbeddingVector::new(input.record_id, values, metadata)
    }
}

fn ensure_unique_input_ids(inputs: &[EmbeddingInput]) -> Result<(), MemoryError> {
    let mut seen = BTreeSet::new();
    for input in inputs {
        if !seen.insert(input.record_id.clone()) {
            return Err(invalid_embedding(format!(
                "duplicate embedding input record id: {}",
                redact_secret_like_text(&input.record_id)
            )));
        }
    }
    Ok(())
}

fn non_empty(value: impl Into<String>, label: &str) -> Result<String, MemoryError> {
    let value = value.into();
    let value = value.trim();
    if value.is_empty() {
        return Err(invalid_embedding(format!("{label} cannot be empty")));
    }
    Ok(value.to_string())
}

fn invalid_embedding(message: impl Into<String>) -> MemoryError {
    MemoryError::InvalidEmbedding(redact_secret_like_text(&message.into()))
}

fn stable_hash_hex(value: &str) -> String {
    format!("{:016x}", stable_hash(value.as_bytes()))
}

fn stable_hash_parts(parts: &[&str]) -> u64 {
    let mut hash = FNV_OFFSET;
    for part in parts {
        hash = update_hash(hash, part.as_bytes());
        hash = update_hash(hash, &[0xff]);
    }
    hash
}

fn stable_hash(bytes: &[u8]) -> u64 {
    update_hash(FNV_OFFSET, bytes)
}

fn update_hash(mut hash: u64, bytes: &[u8]) -> u64 {
    for byte in bytes {
        hash ^= u64::from(*byte);
        hash = hash.wrapping_mul(FNV_PRIME);
    }
    hash
}

fn hash_to_unit(hash: u64) -> f32 {
    let normalized = (hash as f64 / u64::MAX as f64) as f32;
    (normalized * 2.0) - 1.0
}

fn redact_secret_like_text(value: &str) -> String {
    value
        .split_whitespace()
        .map(|token| {
            if is_secret_like(token) {
                "[redacted]"
            } else {
                token
            }
        })
        .collect::<Vec<_>>()
        .join(" ")
}

fn is_secret_like(value: &str) -> bool {
    let value = value.to_ascii_lowercase();
    value.starts_with("sk-")
        || value.contains("secret")
        || value.contains("token")
        || value.contains("password")
        || value.contains("api_key")
        || value.contains("apikey")
}
