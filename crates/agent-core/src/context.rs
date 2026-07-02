use agent_context::{
    PromptLayer, PromptLayerKind, PromptLayerPolicy, PromptLayerSensitivity, PromptLayerSource,
    PromptLayerVisibility, assemble_prompt_bundle,
};

#[derive(Clone, Debug, Default)]
pub struct ContextAssembler {
    layers: Vec<PromptLayer>,
}

impl ContextAssembler {
    #[must_use]
    pub fn assemble(&self, user_input: &str) -> Vec<String> {
        if self.layers.is_empty() {
            return vec![user_input.to_string()];
        }

        let Ok(user_layer) = PromptLayer::new(
            "user",
            PromptLayerKind::User,
            0,
            user_input,
            estimate_tokens(user_input),
            PromptLayerSource::BuiltIn,
            PromptLayerVisibility::Full,
            PromptLayerSensitivity::Public,
            PromptLayerPolicy::Required,
            1,
        ) else {
            return vec![user_input.to_string()];
        };

        let mut layers = self.layers.clone();
        layers.push(user_layer);

        assemble_prompt_bundle(layers)
            .map(|bundle| {
                bundle
                    .messages
                    .into_iter()
                    .map(|message| message.content)
                    .collect()
            })
            .unwrap_or_else(|_| vec![user_input.to_string()])
    }

    #[must_use]
    pub fn with_layers(layers: Vec<PromptLayer>) -> Self {
        Self { layers }
    }
}

fn estimate_tokens(value: &str) -> u32 {
    let count = value
        .split_whitespace()
        .filter(|word| !word.is_empty())
        .count();
    u32::try_from(count.max(1)).unwrap_or(u32::MAX)
}
