use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ModelStreamEvent {
    Started {
        request_id: String,
    },
    Delta {
        text: String,
    },
    Completed {
        finish_reason: String,
        output_token_count: u32,
    },
}

pub type ModelStreamResult = Vec<ModelStreamEvent>;
