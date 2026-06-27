use agent_context::{ContextError, PromptBudget, PromptLayerId, PromptProfile, PromptProfileId};
use serde_json::json;

#[test]
fn prompt_profile_accepts_valid_contract() {
    let profile = PromptProfile::new(
        PromptProfileId::new("default").expect("valid profile id"),
        1,
        "Default Agent",
        vec![PromptLayerId::new("system").expect("valid layer id")],
        PromptBudget::new(4096, 1024).expect("valid budget"),
    )
    .expect("valid profile");

    assert_eq!(profile.id.as_str(), "default");
    assert_eq!(profile.version, 1);
    assert_eq!(profile.display_name, "Default Agent");
    assert_eq!(profile.required_layers[0].as_str(), "system");
    assert_eq!(profile.default_budget.max_tokens, 4096);
    assert_eq!(profile.default_budget.reserved_response_tokens, 1024);
}

#[test]
fn prompt_profile_rejects_empty_id() {
    let error = PromptProfileId::new(" ").expect_err("empty profile id rejected");

    assert!(matches!(
        error,
        ContextError::InvalidLayer(message) if message.contains("profile id")
    ));
}

#[test]
fn prompt_profile_rejects_zero_version() {
    let error = PromptProfile::new(
        PromptProfileId::new("default").expect("valid profile id"),
        0,
        "Default Agent",
        vec![],
        PromptBudget::new(4096, 1024).expect("valid budget"),
    )
    .expect_err("zero version rejected");

    assert!(matches!(
        error,
        ContextError::InvalidLayer(message) if message.contains("version")
    ));
}

#[test]
fn prompt_profile_rejects_duplicate_required_layers() {
    let system = PromptLayerId::new("system").expect("valid layer id");
    let error = PromptProfile::new(
        PromptProfileId::new("default").expect("valid profile id"),
        1,
        "Default Agent",
        vec![system.clone(), system],
        PromptBudget::new(4096, 1024).expect("valid budget"),
    )
    .expect_err("duplicate required layers rejected");

    assert!(matches!(
        error,
        ContextError::InvalidLayer(message) if message.contains("duplicate required layer")
    ));
}

#[test]
fn prompt_profile_serializes_stable_shape() {
    let profile = PromptProfile::new(
        PromptProfileId::new("default").expect("valid profile id"),
        2,
        "Default Agent",
        vec![
            PromptLayerId::new("system").expect("valid layer id"),
            PromptLayerId::new("runtime").expect("valid layer id"),
        ],
        PromptBudget::new(8192, 2048).expect("valid budget"),
    )
    .expect("valid profile");

    let value = serde_json::to_value(profile).expect("serialize profile");

    assert_eq!(
        value,
        json!({
            "id": "default",
            "version": 2,
            "display_name": "Default Agent",
            "required_layers": ["system", "runtime"],
            "default_budget": {
                "max_tokens": 8192,
                "reserved_response_tokens": 2048
            }
        })
    );
}
