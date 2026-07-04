use std::collections::{BTreeMap, HashMap, HashSet};

use agent_memory::{
    EmbeddingInput, EmbeddingProvider, EmbeddingRequest, EmbeddingVector, MemorySensitivity,
};
use serde::{Deserialize, Serialize};

use crate::{
    ContextCandidate, ContextCandidateOmissionReason, ContextError, ContextRetrievalDiagnostic,
    ContextRetrievalDiagnosticSeverity, ContextRetrievalQuery, ContextSelectionReason,
    ProjectContextSummary, ProjectContextVisibility, ProjectFileKind, ProjectFileSummary,
    PromptLayer, PromptLayerKind, PromptLayerPolicy, PromptLayerSensitivity, PromptLayerSource,
    PromptLayerVisibility,
    retrieval::{lexical_tokens, score_file},
};

const QUERY_EMBEDDING_ID: &str = "query";
const MAX_SEMANTIC_DOCUMENT_CHARS: usize = 1_200;
const MAX_PROMPT_TASK_CHARS: usize = 160;
const SEMANTIC_SCORE_SCALE: f64 = 1_000.0;

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ContextSemanticRetrievalConfig {
    pub mode: ContextSemanticRetrievalMode,
    pub lexical_weight: f64,
    pub semantic_weight: f64,
    pub minimum_score: f64,
    pub fallback_policy: ContextSemanticFallbackPolicy,
}

impl ContextSemanticRetrievalConfig {
    pub fn new(
        mode: ContextSemanticRetrievalMode,
        lexical_weight: f64,
        semantic_weight: f64,
        minimum_score: f64,
        fallback_policy: ContextSemanticFallbackPolicy,
    ) -> Result<Self, ContextError> {
        validate_mode(&mode)?;
        validate_weight("lexical_weight", lexical_weight)?;
        validate_weight("semantic_weight", semantic_weight)?;
        if lexical_weight == 0.0 && semantic_weight == 0.0 {
            return Err(invalid_retrieval(
                "semantic retrieval weights cannot both be zero",
            ));
        }
        if !minimum_score.is_finite() || minimum_score < 0.0 {
            return Err(invalid_retrieval(
                "minimum_score must be finite and non-negative",
            ));
        }
        if matches!(mode, ContextSemanticRetrievalMode::Disabled)
            && fallback_policy == ContextSemanticFallbackPolicy::FailClosed
        {
            return Err(invalid_retrieval(
                "disabled semantic retrieval cannot fail closed",
            ));
        }

        Ok(Self {
            mode,
            lexical_weight,
            semantic_weight,
            minimum_score,
            fallback_policy,
        })
    }

    #[must_use]
    pub fn disabled() -> Self {
        Self {
            mode: ContextSemanticRetrievalMode::Disabled,
            lexical_weight: 1.0,
            semantic_weight: 0.0,
            minimum_score: 0.0,
            fallback_policy: ContextSemanticFallbackPolicy::LexicalOnly,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ContextSemanticRetrievalMode {
    Disabled,
    Provider {
        provider_id: String,
        model_id: String,
        dimensions: usize,
    },
}

#[derive(Copy, Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ContextSemanticFallbackPolicy {
    LexicalOnly,
    FailClosed,
}

impl ContextSemanticFallbackPolicy {
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::LexicalOnly => "lexical_only",
            Self::FailClosed => "fail_closed",
        }
    }
}

#[derive(Copy, Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ContextSemanticRetrievalStatus {
    Disabled,
    Completed,
    FallbackLexical,
}

impl ContextSemanticRetrievalStatus {
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Disabled => "disabled",
            Self::Completed => "completed",
            Self::FallbackLexical => "fallback_lexical",
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct ContextSemanticRetrievalSummary {
    pub status: ContextSemanticRetrievalStatus,
    pub provider_id: Option<String>,
    pub model_id: Option<String>,
    pub dimensions: Option<usize>,
    pub document_count: u32,
    pub embedded_document_count: u32,
}

impl ContextSemanticRetrievalSummary {
    fn disabled(document_count: u32) -> Self {
        Self {
            status: ContextSemanticRetrievalStatus::Disabled,
            provider_id: None,
            model_id: None,
            dimensions: None,
            document_count,
            embedded_document_count: 0,
        }
    }

    fn provider(
        status: ContextSemanticRetrievalStatus,
        provider_id: String,
        model_id: String,
        dimensions: usize,
        document_count: u32,
        embedded_document_count: u32,
    ) -> Self {
        Self {
            status,
            provider_id: Some(provider_id),
            model_id: Some(model_id),
            dimensions: Some(dimensions),
            document_count,
            embedded_document_count,
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct ProjectSemanticDocument {
    pub path: String,
    pub text_hash: String,
    pub token_estimate: u32,
    pub visibility: ProjectContextVisibility,
    pub embeddable: bool,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ContextHybridCandidate {
    pub candidate: ContextCandidate,
    pub lexical_score: f64,
    pub semantic_score: Option<f64>,
    pub combined_score: f64,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ContextHybridCandidateOmission {
    pub candidate: ContextHybridCandidate,
    pub reason: ContextCandidateOmissionReason,
}

impl ContextHybridCandidateOmission {
    pub const fn new(
        candidate: ContextHybridCandidate,
        reason: ContextCandidateOmissionReason,
    ) -> Self {
        Self { candidate, reason }
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ContextHybridRetrievalReport {
    pub query: ContextRetrievalQuery,
    pub semantic: ContextSemanticRetrievalSummary,
    pub selected: Vec<ContextHybridCandidate>,
    pub omitted: Vec<ContextHybridCandidateOmission>,
    pub diagnostics: Vec<ContextRetrievalDiagnostic>,
    pub used_tokens: u32,
}

impl ContextHybridRetrievalReport {
    pub fn new(
        query: ContextRetrievalQuery,
        semantic: ContextSemanticRetrievalSummary,
        selected: Vec<ContextHybridCandidate>,
        omitted: Vec<ContextHybridCandidateOmission>,
        mut diagnostics: Vec<ContextRetrievalDiagnostic>,
        used_tokens: u32,
    ) -> Self {
        diagnostics.sort_by(|left, right| {
            left.code
                .cmp(&right.code)
                .then_with(|| left.path.cmp(&right.path))
                .then_with(|| left.message.cmp(&right.message))
        });
        Self {
            query,
            semantic,
            selected,
            omitted,
            diagnostics,
            used_tokens,
        }
    }

    pub fn to_prompt_layer(
        &self,
        id: impl Into<String>,
        priority: i32,
    ) -> Result<PromptLayer, ContextError> {
        let content = self.prompt_content();
        let token_estimate = estimate_tokens(&content);
        PromptLayer::new(
            id,
            PromptLayerKind::Project,
            priority,
            content.clone(),
            token_estimate,
            PromptLayerSource::ProjectFile {
                path: "project-context-hybrid-retrieval".to_string(),
                content_hash: fnv1a64(&content),
            },
            PromptLayerVisibility::SummaryOnly,
            PromptLayerSensitivity::Workspace,
            PromptLayerPolicy::Optional,
            1,
        )
    }

    fn prompt_content(&self) -> String {
        let mut lines = vec![format!(
            "Hybrid project context for task: {}",
            bounded_prompt_text(&self.query.task)
        )];
        lines.push(format!(
            "Semantic status: semantic={} provider={} model={} embedded={}/{}",
            self.semantic.status.as_str(),
            self.semantic.provider_id.as_deref().unwrap_or("none"),
            self.semantic.model_id.as_deref().unwrap_or("none"),
            self.semantic.embedded_document_count,
            self.semantic.document_count
        ));
        lines.push(format!(
            "Selected {} files using {}/{} tokens.",
            self.selected.len(),
            self.used_tokens,
            self.query.budget_tokens
        ));
        if !self.selected.is_empty() {
            lines.push("Selected files:".to_string());
            for candidate in &self.selected {
                let reasons = candidate
                    .candidate
                    .reasons
                    .iter()
                    .map(|reason| reason.as_str())
                    .collect::<Vec<_>>()
                    .join(",");
                lines.push(format!(
                    "- {} [{}; {}; lexical={:.3}; semantic={}; combined={:.3}; tokens={}; reasons={}]",
                    candidate.candidate.path,
                    candidate.candidate.kind.as_str(),
                    candidate.candidate.visibility.as_str(),
                    candidate.lexical_score,
                    candidate
                        .semantic_score
                        .map(|score| format!("{score:.3}"))
                        .unwrap_or_else(|| "none".to_string()),
                    candidate.combined_score,
                    candidate.candidate.token_estimate,
                    reasons
                ));
            }
        }
        if !self.omitted.is_empty() {
            lines.push("Omitted candidates:".to_string());
            for omission in self.omitted.iter().take(40) {
                lines.push(format!(
                    "- {}: {}",
                    omission.candidate.candidate.path,
                    omission.reason.as_str()
                ));
            }
        }
        lines.join("\n")
    }
}

#[derive(Clone, Debug)]
struct ProjectSemanticDocumentInput {
    document: ProjectSemanticDocument,
    text: String,
}

#[derive(Clone, Debug)]
struct CandidateScores {
    file: ProjectFileSummary,
    lexical_score: f64,
    semantic_score: Option<f64>,
    combined_score: f64,
    reasons: Vec<ContextSelectionReason>,
}

type SemanticScoreResult = Result<
    (
        Option<ContextSemanticRetrievalSummary>,
        HashMap<String, f64>,
        Vec<ContextRetrievalDiagnostic>,
    ),
    ContextError,
>;

pub fn build_project_semantic_documents(
    summary: &ProjectContextSummary,
) -> Result<Vec<ProjectSemanticDocument>, ContextError> {
    semantic_document_inputs(summary).map(|inputs| {
        inputs
            .into_iter()
            .map(|input| input.document)
            .collect::<Vec<_>>()
    })
}

pub async fn retrieve_hybrid_context(
    summary: &ProjectContextSummary,
    query: ContextRetrievalQuery,
    config: ContextSemanticRetrievalConfig,
    provider: Option<&dyn EmbeddingProvider>,
) -> Result<ContextHybridRetrievalReport, ContextError> {
    let documents = semantic_document_inputs(summary)?;
    let document_count = documents.len().try_into().unwrap_or(u32::MAX);
    let mut diagnostics = pin_diagnostics(summary, &query)?;
    let (semantic_summary, semantic_scores, semantic_diagnostics) =
        semantic_scores(&query, &config, provider, &documents).await?;
    diagnostics.extend(semantic_diagnostics);
    let semantic_summary = match semantic_summary {
        Some(summary) => summary,
        None => ContextSemanticRetrievalSummary::disabled(document_count),
    };

    let mut candidates = hybrid_candidates(summary, &query, &config, &semantic_scores)?;
    candidates.sort_by(compare_hybrid_candidates);

    let mut selected = Vec::new();
    let mut omitted = Vec::new();
    let mut used_tokens = 0_u32;
    for candidate in candidates {
        if selected.len() >= query.max_candidates as usize {
            omitted.push(ContextHybridCandidateOmission::new(
                candidate,
                ContextCandidateOmissionReason::MaxCandidates,
            ));
            continue;
        }
        if used_tokens.saturating_add(candidate.candidate.token_estimate) > query.budget_tokens {
            omitted.push(ContextHybridCandidateOmission::new(
                candidate,
                ContextCandidateOmissionReason::BudgetExceeded,
            ));
            continue;
        }
        used_tokens = used_tokens.saturating_add(candidate.candidate.token_estimate);
        selected.push(candidate);
    }

    if selected.is_empty() {
        diagnostics.push(diagnostic(
            ContextRetrievalDiagnosticSeverity::Warning,
            "context_retrieval.no_candidates",
            "no relevant files were selected",
            None,
        ));
    }

    Ok(ContextHybridRetrievalReport::new(
        query,
        semantic_summary,
        selected,
        omitted,
        diagnostics,
        used_tokens,
    ))
}

async fn semantic_scores(
    query: &ContextRetrievalQuery,
    config: &ContextSemanticRetrievalConfig,
    provider: Option<&dyn EmbeddingProvider>,
    documents: &[ProjectSemanticDocumentInput],
) -> SemanticScoreResult {
    let ContextSemanticRetrievalMode::Provider {
        provider_id,
        model_id,
        dimensions,
    } = &config.mode
    else {
        return Ok((None, HashMap::new(), Vec::new()));
    };
    let document_count = documents.len().try_into().unwrap_or(u32::MAX);
    let embedded_documents = documents
        .iter()
        .filter(|input| input.document.embeddable)
        .collect::<Vec<_>>();
    let embedded_document_count = embedded_documents.len().try_into().unwrap_or(u32::MAX);

    let Some(provider) = provider else {
        return semantic_failure(
            config,
            provider_id,
            model_id,
            *dimensions,
            document_count,
            "semantic embedding provider is unavailable",
        );
    };

    let mut inputs = vec![
        EmbeddingInput::new(
            QUERY_EMBEDDING_ID,
            bounded_prompt_text(&query.task),
            MemorySensitivity::Internal,
        )
        .map_err(memory_error)?,
    ];
    for document in &embedded_documents {
        inputs.push(
            EmbeddingInput::new(
                semantic_record_id(&document.document.path),
                document.text.clone(),
                MemorySensitivity::Internal,
            )
            .map_err(memory_error)?,
        );
    }

    let request = EmbeddingRequest::new(provider_id, model_id, inputs, Some(*dimensions))
        .map_err(memory_error)?;
    let batch = match provider.embed(request).await {
        Ok(batch) => batch,
        Err(error) => {
            return semantic_failure(
                config,
                provider_id,
                model_id,
                *dimensions,
                document_count,
                &error.to_string(),
            );
        }
    };
    let vectors = batch
        .vectors
        .into_iter()
        .map(|vector| (vector.record_id.clone(), vector))
        .collect::<BTreeMap<_, _>>();
    let Some(query_vector) = vectors.get(QUERY_EMBEDDING_ID) else {
        return semantic_failure(
            config,
            provider_id,
            model_id,
            *dimensions,
            document_count,
            "semantic embedding response did not include a query vector",
        );
    };

    let mut scores = HashMap::new();
    for document in embedded_documents {
        let record_id = semantic_record_id(&document.document.path);
        let Some(document_vector) = vectors.get(&record_id) else {
            continue;
        };
        let score = cosine_similarity(query_vector, document_vector)?;
        if score > 0.0 {
            scores.insert(document.document.path.clone(), score);
        }
    }

    Ok((
        Some(ContextSemanticRetrievalSummary::provider(
            ContextSemanticRetrievalStatus::Completed,
            provider_id.clone(),
            model_id.clone(),
            *dimensions,
            document_count,
            embedded_document_count,
        )),
        scores,
        Vec::new(),
    ))
}

fn semantic_failure(
    config: &ContextSemanticRetrievalConfig,
    provider_id: &str,
    model_id: &str,
    dimensions: usize,
    document_count: u32,
    message: &str,
) -> SemanticScoreResult {
    if config.fallback_policy == ContextSemanticFallbackPolicy::FailClosed {
        return Err(invalid_retrieval(format!(
            "semantic retrieval failed: {}",
            redact_secret_like_text(message)
        )));
    }
    Ok((
        Some(ContextSemanticRetrievalSummary::provider(
            ContextSemanticRetrievalStatus::FallbackLexical,
            provider_id.to_string(),
            model_id.to_string(),
            dimensions,
            document_count,
            0,
        )),
        HashMap::new(),
        vec![diagnostic(
            ContextRetrievalDiagnosticSeverity::Warning,
            "context_retrieval.semantic.fallback",
            "semantic retrieval failed; using lexical retrieval",
            None,
        )],
    ))
}

fn hybrid_candidates(
    summary: &ProjectContextSummary,
    query: &ContextRetrievalQuery,
    config: &ContextSemanticRetrievalConfig,
    semantic_scores: &HashMap<String, f64>,
) -> Result<Vec<ContextHybridCandidate>, ContextError> {
    let task_tokens = lexical_tokens(&query.task);
    let pinned_paths = query
        .pinned_paths
        .iter()
        .map(String::as_str)
        .collect::<HashSet<_>>();
    let mut scored = Vec::new();
    let mut max_lexical_score = 0_u32;
    for file in &summary.files {
        let (lexical_raw_score, mut reasons) = score_file(file, &task_tokens, &pinned_paths);
        if semantic_scores
            .get(&file.path)
            .is_some_and(|score| *score > 0.0)
        {
            reasons.push(ContextSelectionReason::SemanticMatch);
        }
        if lexical_raw_score > max_lexical_score {
            max_lexical_score = lexical_raw_score;
        }
        if lexical_raw_score == 0 && !reasons.contains(&ContextSelectionReason::SemanticMatch) {
            continue;
        }
        scored.push((file, lexical_raw_score, reasons));
    }

    let max_lexical_score = f64::from(max_lexical_score.max(1));
    let mut candidates = Vec::new();
    for (file, lexical_raw_score, reasons) in scored {
        let lexical_score = f64::from(lexical_raw_score) / max_lexical_score;
        let semantic_score = semantic_scores.get(&file.path).copied();
        let combined_score = (lexical_score * config.lexical_weight)
            + (semantic_score.unwrap_or(0.0) * config.semantic_weight);
        if !combined_score.is_finite() {
            return Err(invalid_retrieval(
                "hybrid retrieval produced a non-finite score",
            ));
        }
        if combined_score < config.minimum_score {
            continue;
        }
        candidates.push(CandidateScores {
            file: file.clone(),
            lexical_score,
            semantic_score,
            combined_score,
            reasons,
        });
    }

    candidates
        .into_iter()
        .map(hybrid_candidate_from_scores)
        .collect()
}

fn hybrid_candidate_from_scores(
    scores: CandidateScores,
) -> Result<ContextHybridCandidate, ContextError> {
    let diagnostic_score = (scores.combined_score * SEMANTIC_SCORE_SCALE)
        .round()
        .clamp(1.0, f64::from(u32::MAX)) as u32;
    let candidate = ContextCandidate::new(
        scores.file.path,
        scores.file.kind,
        scores.file.language,
        scores.file.size_bytes,
        scores.file.line_count,
        scores.file.token_estimate,
        scores.file.visibility,
        diagnostic_score,
        scores.reasons,
    )?;
    Ok(ContextHybridCandidate {
        candidate,
        lexical_score: round_score(scores.lexical_score),
        semantic_score: scores.semantic_score.map(round_score),
        combined_score: round_score(scores.combined_score),
    })
}

fn compare_hybrid_candidates(
    left: &ContextHybridCandidate,
    right: &ContextHybridCandidate,
) -> std::cmp::Ordering {
    right
        .combined_score
        .total_cmp(&left.combined_score)
        .then_with(|| {
            right
                .semantic_score
                .unwrap_or(0.0)
                .total_cmp(&left.semantic_score.unwrap_or(0.0))
        })
        .then_with(|| right.lexical_score.total_cmp(&left.lexical_score))
        .then_with(|| left.candidate.path.cmp(&right.candidate.path))
}

fn semantic_document_inputs(
    summary: &ProjectContextSummary,
) -> Result<Vec<ProjectSemanticDocumentInput>, ContextError> {
    summary
        .files
        .iter()
        .map(|file| {
            let embeddable = is_embeddable(file);
            let text = bounded_semantic_text(&semantic_document_text(file));
            let document = ProjectSemanticDocument {
                path: file.path.clone(),
                text_hash: fnv1a64(&text),
                token_estimate: estimate_tokens(&text),
                visibility: file.visibility,
                embeddable,
            };
            Ok(ProjectSemanticDocumentInput { document, text })
        })
        .collect()
}

fn semantic_document_text(file: &ProjectFileSummary) -> String {
    let language = file.language.as_deref().unwrap_or("unknown");
    let file_name = file.path.rsplit('/').next().unwrap_or(file.path.as_str());
    let directory = file
        .path
        .rsplit_once('/')
        .map(|(directory, _)| directory)
        .unwrap_or(".");
    format!(
        "path: {}\nfile: {}\ndirectory: {}\nkind: {}\nlanguage: {}\nvisibility: {}\nsize_bytes: {}\nline_count: {}\ntoken_estimate: {}",
        file.path,
        file_name,
        directory,
        file.kind.as_str(),
        language,
        file.visibility.as_str(),
        file.size_bytes,
        file.line_count
            .map_or_else(|| "unknown".to_string(), |count| count.to_string()),
        file.token_estimate
    )
}

fn is_embeddable(file: &ProjectFileSummary) -> bool {
    file.visibility != ProjectContextVisibility::MetadataOnly
        && !matches!(
            file.kind,
            ProjectFileKind::SecretLike | ProjectFileKind::Binary | ProjectFileKind::Generated
        )
}

fn pin_diagnostics(
    summary: &ProjectContextSummary,
    query: &ContextRetrievalQuery,
) -> Result<Vec<ContextRetrievalDiagnostic>, ContextError> {
    let known_paths = summary
        .files
        .iter()
        .map(|file| file.path.as_str())
        .collect::<HashSet<_>>();
    let mut diagnostics = Vec::new();
    for pinned_path in &query.pinned_paths {
        if !known_paths.contains(pinned_path.as_str()) {
            diagnostics.push(diagnostic(
                ContextRetrievalDiagnosticSeverity::Warning,
                "context_retrieval.pin.unknown",
                "pinned path was not found in project summary",
                Some(pinned_path.clone()),
            ));
        }
    }
    Ok(diagnostics)
}

fn semantic_record_id(path: &str) -> String {
    format!("project:{path}")
}

fn validate_mode(mode: &ContextSemanticRetrievalMode) -> Result<(), ContextError> {
    match mode {
        ContextSemanticRetrievalMode::Disabled => Ok(()),
        ContextSemanticRetrievalMode::Provider {
            provider_id,
            model_id,
            dimensions,
        } => {
            validate_non_empty("semantic provider_id", provider_id)?;
            validate_non_empty("semantic model_id", model_id)?;
            if *dimensions == 0 {
                return Err(invalid_retrieval(
                    "semantic dimensions must be greater than zero",
                ));
            }
            Ok(())
        }
    }
}

fn validate_weight(label: &str, value: f64) -> Result<(), ContextError> {
    if !value.is_finite() || value < 0.0 {
        return Err(invalid_retrieval(format!(
            "{label} must be finite and non-negative"
        )));
    }
    Ok(())
}

fn cosine_similarity(
    query_vector: &EmbeddingVector,
    document_vector: &EmbeddingVector,
) -> Result<f64, ContextError> {
    if query_vector.values.len() != document_vector.values.len() {
        return Err(invalid_retrieval(format!(
            "semantic vector dimension mismatch: expected {}, got {}",
            query_vector.values.len(),
            document_vector.values.len()
        )));
    }
    let mut dot = 0.0_f32;
    let mut query_norm = 0.0_f32;
    let mut document_norm = 0.0_f32;
    for (query, document) in query_vector.values.iter().zip(&document_vector.values) {
        dot += query * document;
        query_norm += query * query;
        document_norm += document * document;
    }
    if query_norm == 0.0 || document_norm == 0.0 {
        return Ok(0.0);
    }
    let score = dot / (query_norm.sqrt() * document_norm.sqrt());
    if !score.is_finite() {
        return Err(invalid_retrieval("semantic score must be finite"));
    }
    Ok(f64::from(score.clamp(-1.0, 1.0).max(0.0)))
}

fn diagnostic(
    severity: ContextRetrievalDiagnosticSeverity,
    code: &str,
    message: &str,
    path: Option<String>,
) -> ContextRetrievalDiagnostic {
    ContextRetrievalDiagnostic::new(severity, code, message, path)
        .expect("semantic retrieval emits valid diagnostics")
}

fn memory_error(error: agent_memory::MemoryError) -> ContextError {
    invalid_retrieval(error.to_string())
}

fn validate_non_empty(label: &str, value: &str) -> Result<(), ContextError> {
    if value.trim().is_empty() {
        return Err(invalid_retrieval(format!("{label} cannot be empty")));
    }
    Ok(())
}

fn invalid_retrieval(message: impl Into<String>) -> ContextError {
    ContextError::InvalidRetrieval(redact_secret_like_text(&message.into()))
}

fn round_score(score: f64) -> f64 {
    (score * 1_000.0).round() / 1_000.0
}

fn estimate_tokens(content: &str) -> u32 {
    let chars = content.chars().count();
    chars.div_ceil(4).max(1).try_into().unwrap_or(u32::MAX)
}

fn fnv1a64(value: &str) -> String {
    let mut hash = 0xcbf29ce484222325u64;
    for byte in value.as_bytes() {
        hash ^= u64::from(*byte);
        hash = hash.wrapping_mul(0x100000001b3);
    }
    format!("fnv1a64:{hash:016x}")
}

fn bounded_semantic_text(value: &str) -> String {
    let redacted = redact_secret_like_text(value);
    if redacted.chars().count() > MAX_SEMANTIC_DOCUMENT_CHARS {
        redacted.chars().take(MAX_SEMANTIC_DOCUMENT_CHARS).collect()
    } else {
        redacted
    }
}

fn bounded_prompt_text(value: &str) -> String {
    let redacted = redact_secret_like_text(value);
    if redacted.chars().count() > MAX_PROMPT_TASK_CHARS {
        redacted.chars().take(MAX_PROMPT_TASK_CHARS).collect()
    } else {
        redacted
    }
}

fn redact_secret_like_text(value: &str) -> String {
    value
        .split_whitespace()
        .map(|part| {
            let lower = part.to_ascii_lowercase();
            if part.starts_with("sk-")
                || lower.contains("api_key")
                || lower.contains("apikey")
                || lower.contains("secret")
                || lower.contains("token")
                || lower.contains("password")
            {
                "[redacted]"
            } else {
                part
            }
        })
        .collect::<Vec<_>>()
        .join(" ")
}
