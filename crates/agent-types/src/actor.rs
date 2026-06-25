use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ActorKind {
    User,
    Agent,
    System,
    Tool,
    Scheduler,
    Plugin,
    McpServer,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct ActorRef {
    pub kind: ActorKind,
    pub id: String,
    pub display_name: Option<String>,
}

impl ActorRef {
    #[must_use]
    pub fn system() -> Self {
        Self {
            kind: ActorKind::System,
            id: "system".to_string(),
            display_name: Some("system".to_string()),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn actor_kind_uses_snake_case() {
        let json = serde_json::to_string(&ActorKind::McpServer).expect("serialize actor kind");
        assert_eq!(json, "\"mcp_server\"");
    }
}
