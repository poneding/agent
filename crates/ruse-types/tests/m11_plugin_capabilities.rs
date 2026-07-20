use ruse_types::{PluginCapabilityValidated, RuntimeEventPayload};
use serde_json::json;

#[test]
fn m11_plugin_capabilities_event_serializes_stable_shape() {
    let payload = RuntimeEventPayload::PluginCapabilityValidated(PluginCapabilityValidated {
        plugin_name: "example-plugin".to_string(),
        capability_count: 2,
        tool_count: 1,
        hook_count: 1,
    });

    let value = serde_json::to_value(payload).expect("serialize plugin capability payload");

    assert_eq!(
        value,
        json!({
            "type": "plugin_capability_validated",
            "data": {
                "plugin_name": "example-plugin",
                "capability_count": 2,
                "tool_count": 1,
                "hook_count": 1
            }
        })
    );
}
