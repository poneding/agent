use serde::{Deserialize, Serialize};

use crate::ContextError;

#[derive(Copy, Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PromptLayerKind {
    System,
    Product,
    Project,
    Skill,
    Tool,
    Memory,
    Runtime,
    User,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct PromptLayer {
    pub id: String,
    pub kind: PromptLayerKind,
    pub priority: i32,
    pub content: String,
    pub token_estimate: u32,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct PromptMessage {
    pub role: String,
    pub content: String,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct PromptBundle {
    pub layers: Vec<PromptLayer>,
    pub messages: Vec<PromptMessage>,
    pub estimated_tokens: u32,
}

pub fn assemble_prompt_bundle(layers: Vec<PromptLayer>) -> Result<PromptBundle, ContextError> {
    for layer in &layers {
        validate_layer(layer)?;
    }

    let mut indexed_layers: Vec<_> = layers.into_iter().enumerate().collect();
    indexed_layers.sort_by(|(left_index, left), (right_index, right)| {
        right
            .priority
            .cmp(&left.priority)
            .then_with(|| left_index.cmp(right_index))
    });

    let layers: Vec<_> = indexed_layers.into_iter().map(|(_, layer)| layer).collect();
    let messages = layers
        .iter()
        .map(|layer| PromptMessage {
            role: role_for_kind(layer.kind).to_string(),
            content: layer.content.clone(),
        })
        .collect();
    let estimated_tokens = layers.iter().map(|layer| layer.token_estimate).sum();

    Ok(PromptBundle {
        layers,
        messages,
        estimated_tokens,
    })
}

fn validate_layer(layer: &PromptLayer) -> Result<(), ContextError> {
    if layer.id.trim().is_empty() {
        return Err(ContextError::InvalidLayer(
            "layer id cannot be empty".to_string(),
        ));
    }
    if layer.content.trim().is_empty() {
        return Err(ContextError::InvalidLayer(
            "layer content cannot be empty".to_string(),
        ));
    }
    Ok(())
}

fn role_for_kind(kind: PromptLayerKind) -> &'static str {
    match kind {
        PromptLayerKind::User => "user",
        PromptLayerKind::System
        | PromptLayerKind::Product
        | PromptLayerKind::Project
        | PromptLayerKind::Skill
        | PromptLayerKind::Tool
        | PromptLayerKind::Memory
        | PromptLayerKind::Runtime => "system",
    }
}
