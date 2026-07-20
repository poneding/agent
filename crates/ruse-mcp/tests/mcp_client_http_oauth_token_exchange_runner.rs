use ruse_mcp::{
    FixtureMcpClientHttpOAuthTokenExchangeTransport, McpClientHttpOAuthTokenExchangeDiagnosticCode,
    McpClientHttpOAuthTokenExchangeDiagnosticSeverity, McpClientHttpOAuthTokenExchangeHttpResponse,
    McpClientHttpOAuthTokenExchangeRequest, McpError, exchange_mcp_client_http_oauth_token,
};
use serde_json::json;

fn exchange_request() -> McpClientHttpOAuthTokenExchangeRequest {
    McpClientHttpOAuthTokenExchangeRequest {
        token_endpoint: "https://auth.example.com/token".to_string(),
        form: vec![
            ("grant_type".to_string(), "authorization_code".to_string()),
            ("code".to_string(), "callback-code-secret-value".to_string()),
            (
                "code_verifier".to_string(),
                "verifier-secret-token".to_string(),
            ),
        ],
    }
}

fn exchange_response(status: u16, body: &str) -> McpClientHttpOAuthTokenExchangeHttpResponse {
    McpClientHttpOAuthTokenExchangeHttpResponse::new(status, body).expect("response")
}

#[tokio::test]
async fn mcp_client_http_oauth_token_exchange_runner_reports_success_without_leaking() {
    let mut transport =
        FixtureMcpClientHttpOAuthTokenExchangeTransport::from_responses([Ok(exchange_response(
            200,
            r#"{"access_token":"access-secret-token","token_type":"Bearer","expires_in":3600}"#,
        ))]);

    let outcome = exchange_mcp_client_http_oauth_token(&mut transport, &exchange_request())
        .await
        .expect("outcome");

    assert!(outcome.report.exchanged);
    assert_eq!(
        outcome.report.diagnostics[0].severity,
        McpClientHttpOAuthTokenExchangeDiagnosticSeverity::Info
    );
    assert_eq!(
        outcome.report.diagnostics[0].code,
        McpClientHttpOAuthTokenExchangeDiagnosticCode::TokenExchangeSucceeded
    );
    assert_eq!(
        outcome.response.as_ref().expect("response").access_token,
        "access-secret-token"
    );
    assert_eq!(
        serde_json::to_value(&outcome.report).expect("json"),
        json!({
            "exchanged": true,
            "diagnostics": [
                {
                    "severity": "info",
                    "code": "token_exchange_succeeded",
                    "message": "MCP HTTP OAuth token exchange succeeded"
                }
            ]
        })
    );
    let rendered = serde_json::to_string(&outcome.report).expect("json");
    assert!(!rendered.contains("access-secret-token"));
    assert!(!rendered.contains("callback-code-secret-value"));
    assert!(!rendered.contains("secret-token"));
}

#[tokio::test]
async fn mcp_client_http_oauth_token_exchange_runner_reports_provider_error_without_leaking() {
    let mut transport =
        FixtureMcpClientHttpOAuthTokenExchangeTransport::from_responses([Ok(exchange_response(
            400,
            r#"{"error":"invalid_grant","error_description":"provider secret-token description"}"#,
        ))]);

    let outcome = exchange_mcp_client_http_oauth_token(&mut transport, &exchange_request())
        .await
        .expect("outcome");

    assert!(!outcome.report.exchanged);
    assert!(outcome.response.is_none());
    assert_eq!(
        outcome.report.diagnostics[0].severity,
        McpClientHttpOAuthTokenExchangeDiagnosticSeverity::Error
    );
    assert_eq!(
        outcome.report.diagnostics[0].code,
        McpClientHttpOAuthTokenExchangeDiagnosticCode::ProviderError
    );
    let rendered = serde_json::to_string(&outcome.report).expect("json");
    assert!(!rendered.contains("invalid_grant"));
    assert!(!rendered.contains("provider secret-token description"));
    assert!(!rendered.contains("secret-token"));
}

#[tokio::test]
async fn mcp_client_http_oauth_token_exchange_runner_reports_invalid_response_without_leaking() {
    let mut transport =
        FixtureMcpClientHttpOAuthTokenExchangeTransport::from_responses([Ok(exchange_response(
            200,
            r#"{"access_token":"access-secret-token","expires_in":"soon"}"#,
        ))]);

    let outcome = exchange_mcp_client_http_oauth_token(&mut transport, &exchange_request())
        .await
        .expect("outcome");

    assert!(!outcome.report.exchanged);
    assert!(outcome.response.is_none());
    assert_eq!(
        outcome.report.diagnostics[0].code,
        McpClientHttpOAuthTokenExchangeDiagnosticCode::InvalidResponse
    );
    let rendered = serde_json::to_string(&outcome.report).expect("json");
    assert!(!rendered.contains("access-secret-token"));
    assert!(!rendered.contains("soon"));
    assert!(!rendered.contains("secret-token"));
}

#[tokio::test]
async fn mcp_client_http_oauth_token_exchange_runner_reports_transport_failure_without_leaking() {
    let mut transport = FixtureMcpClientHttpOAuthTokenExchangeTransport::from_responses([Err(
        McpError::Client("network secret-token failure".to_string()),
    )]);

    let outcome = exchange_mcp_client_http_oauth_token(&mut transport, &exchange_request())
        .await
        .expect("outcome");

    assert!(!outcome.report.exchanged);
    assert!(outcome.response.is_none());
    assert_eq!(
        outcome.report.diagnostics[0].severity,
        McpClientHttpOAuthTokenExchangeDiagnosticSeverity::Warning
    );
    assert_eq!(
        outcome.report.diagnostics[0].code,
        McpClientHttpOAuthTokenExchangeDiagnosticCode::TransportFailed
    );
    let rendered = serde_json::to_string(&outcome.report).expect("json");
    assert!(!rendered.contains("network secret-token failure"));
    assert!(!rendered.contains("secret-token"));
}

#[tokio::test]
async fn mcp_client_http_oauth_token_exchange_runner_reports_retryable_status_without_leaking() {
    let mut transport =
        FixtureMcpClientHttpOAuthTokenExchangeTransport::from_responses([Ok(exchange_response(
            503,
            r#"{"error":"server_error","error_description":"secret-token"}"#,
        ))]);

    let outcome = exchange_mcp_client_http_oauth_token(&mut transport, &exchange_request())
        .await
        .expect("outcome");

    assert!(!outcome.report.exchanged);
    assert!(outcome.response.is_none());
    assert_eq!(
        outcome.report.diagnostics[0].severity,
        McpClientHttpOAuthTokenExchangeDiagnosticSeverity::Warning
    );
    assert_eq!(
        outcome.report.diagnostics[0].code,
        McpClientHttpOAuthTokenExchangeDiagnosticCode::RetryableStatus
    );
    let rendered = serde_json::to_string(&outcome.report).expect("json");
    assert!(!rendered.contains("server_error"));
    assert!(!rendered.contains("secret-token"));
}
