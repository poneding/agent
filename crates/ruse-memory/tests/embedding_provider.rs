use ruse_memory::{
    DeterministicEmbeddingProvider, EmbeddingBatch, EmbeddingInput, EmbeddingMetadata,
    EmbeddingProvider, EmbeddingRequest, EmbeddingVector, MemoryError, MemorySensitivity,
};

fn input(record_id: &str, text: &str) -> EmbeddingInput {
    EmbeddingInput::new(record_id, text, MemorySensitivity::Internal).expect("valid input")
}

#[tokio::test]
async fn deterministic_embedding_provider_preserves_order_and_metadata() {
    let provider =
        DeterministicEmbeddingProvider::new("fixture", "fixture-embedding", 4).expect("provider");
    let request = EmbeddingRequest::new(
        "fixture",
        "fixture-embedding",
        vec![
            input("memory-b", "second fact"),
            input("memory-a", "first fact"),
        ],
        Some(4),
    )
    .expect("request");

    let first = provider.embed(request.clone()).await.expect("embed first");
    let second = provider.embed(request).await.expect("embed second");

    assert_eq!(first, second);
    assert_eq!(first.vectors[0].record_id, "memory-b");
    assert_eq!(first.vectors[1].record_id, "memory-a");
    assert_eq!(first.vectors[0].values.len(), 4);
    assert_eq!(first.vectors[0].metadata.provider_id, "fixture");
    assert_eq!(first.vectors[0].metadata.model_id, "fixture-embedding");
    assert_eq!(first.vectors[0].metadata.dimensions, 4);
    assert_eq!(
        first.vectors[0].metadata.sensitivity,
        MemorySensitivity::Internal
    );
    assert_ne!(
        first.vectors[0].metadata.content_hash,
        first.vectors[1].metadata.content_hash
    );
}

#[test]
fn embedding_request_rejects_invalid_inputs_before_provider_execution() {
    let duplicate = EmbeddingRequest::new(
        "fixture",
        "fixture-embedding",
        vec![input("memory", "first"), input("memory", "second")],
        Some(4),
    )
    .expect_err("duplicate ids rejected");
    let empty_model =
        EmbeddingRequest::new("fixture", " ", vec![input("memory", "first")], Some(4))
            .expect_err("empty model rejected");
    let empty_batch = EmbeddingRequest::new("fixture", "fixture-embedding", Vec::new(), Some(4))
        .expect_err("empty batch rejected");
    let zero_dimensions = EmbeddingRequest::new(
        "fixture",
        "fixture-embedding",
        vec![input("memory", "first")],
        Some(0),
    )
    .expect_err("zero dimensions rejected");

    assert!(
        matches!(duplicate, MemoryError::InvalidEmbedding(message) if message.contains("duplicate"))
    );
    assert!(
        matches!(empty_model, MemoryError::InvalidEmbedding(message) if message.contains("model"))
    );
    assert!(
        matches!(empty_batch, MemoryError::InvalidEmbedding(message) if message.contains("input"))
    );
    assert!(
        matches!(zero_dimensions, MemoryError::InvalidEmbedding(message) if message.contains("dimension"))
    );
}

#[test]
fn embedding_vector_rejects_dimension_mismatch_and_non_finite_values() {
    let metadata = EmbeddingMetadata::new(
        "fixture",
        "fixture-embedding",
        2,
        "hash",
        MemorySensitivity::Public,
    )
    .expect("metadata");
    let mismatch = EmbeddingVector::new("memory", vec![0.1], metadata.clone())
        .expect_err("dimension mismatch rejected");
    let non_finite =
        EmbeddingVector::new("memory", vec![0.1, f32::NAN], metadata).expect_err("nan rejected");

    assert!(
        matches!(mismatch, MemoryError::InvalidEmbedding(message) if message.contains("dimension"))
    );
    assert!(
        matches!(non_finite, MemoryError::InvalidEmbedding(message) if message.contains("finite"))
    );
}

#[test]
fn embedding_batch_rejects_duplicate_vectors() {
    let metadata = EmbeddingMetadata::new(
        "fixture",
        "fixture-embedding",
        2,
        "hash",
        MemorySensitivity::Public,
    )
    .expect("metadata");
    let left = EmbeddingVector::new("memory", vec![0.1, 0.2], metadata.clone()).expect("left");
    let right = EmbeddingVector::new("memory", vec![0.3, 0.4], metadata).expect("right");

    let error = EmbeddingBatch::new(vec![left, right]).expect_err("duplicate vectors rejected");

    assert!(
        matches!(error, MemoryError::InvalidEmbedding(message) if message.contains("duplicate"))
    );
}

#[test]
fn embedding_validation_errors_redact_secret_like_text() {
    let error = EmbeddingInput::new(
        "secret-memory",
        "token sk-secret password api_key",
        MemorySensitivity::Sensitive,
    )
    .and_then(|input| EmbeddingRequest::new("fixture", "fixture-embedding", vec![input], Some(0)))
    .expect_err("invalid dimensions");
    let message = error.to_string();

    assert!(!message.contains("sk-secret"));
    assert!(!message.contains("token"));
    assert!(!message.contains("password"));
    assert!(!message.contains("api_key"));
}
