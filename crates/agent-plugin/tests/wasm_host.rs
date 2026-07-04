use agent_plugin::{
    PluginIsolationMode, PluginManifest, PluginWasmConfig, PluginWasmError, PluginWasmHost,
    PluginWasmModule,
};
use serde_json::{Value, json};

const TEMPLATE: &str = include_str!("../../../docs/templates/plugin-manifest.template.json");
const MINIMAL_WASM: &[u8] = b"\0asm\x01\0\0\0";

#[test]
fn wasm_host_accepts_minimal_module_header() {
    let host = PluginWasmHost;
    let config = PluginWasmConfig {
        entrypoint: "plugin_main".to_string(),
        max_module_bytes: 64,
    };
    let module = PluginWasmModule {
        bytes: MINIMAL_WASM.to_vec(),
    };

    let description = host
        .describe_module(&config, &module)
        .expect("describe wasm module");

    assert_eq!(description.entrypoint, "plugin_main");
    assert_eq!(description.module_bytes, MINIMAL_WASM.len());
    assert_eq!(description.version, 1);
}

#[test]
fn wasm_host_rejects_invalid_magic() {
    let err = PluginWasmHost
        .describe_module(
            &config(),
            &PluginWasmModule {
                bytes: b"\0bad\x01\0\0\0".to_vec(),
            },
        )
        .unwrap_err();

    assert!(matches!(err, PluginWasmError::InvalidModule(_)));
}

#[test]
fn wasm_host_rejects_oversized_module() {
    let err = PluginWasmHost
        .describe_module(
            &PluginWasmConfig {
                entrypoint: "plugin_main".to_string(),
                max_module_bytes: 4,
            },
            &PluginWasmModule {
                bytes: MINIMAL_WASM.to_vec(),
            },
        )
        .unwrap_err();

    assert!(matches!(err, PluginWasmError::ModuleTooLarge { .. }));
}

#[test]
fn wasm_host_rejects_manifest_without_entrypoint() {
    let mut value = template_value();
    value["isolation"] = json!({
        "mode": "wasm",
        "entrypoint": null
    });
    let manifest: PluginManifest =
        serde_json::from_value(value).expect("deserialize manifest without validation");

    let err = PluginWasmConfig::from_manifest(&manifest, 64).unwrap_err();

    assert_eq!(manifest.isolation.mode, PluginIsolationMode::Wasm);
    assert!(matches!(err, PluginWasmError::InvalidConfig(_)));
}

#[test]
fn wasm_host_rejects_unsupported_host_call() {
    let err = PluginWasmHost.call_host_function("fs.read").unwrap_err();

    assert_eq!(
        err,
        PluginWasmError::UnsupportedHostCall {
            name: "fs.read".to_string()
        }
    );
}

fn config() -> PluginWasmConfig {
    PluginWasmConfig {
        entrypoint: "plugin_main".to_string(),
        max_module_bytes: 64,
    }
}

fn template_value() -> Value {
    serde_json::from_str(TEMPLATE).expect("parse template json")
}
