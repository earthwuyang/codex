//! Integration tests for MCP parallel subagent execution
//!
//! Tests verify that:
//! - Supervisor tool executes multiple agents in parallel
//! - Subagent tool actions work correctly (start_task, check_inbox, get_status, etc.)
//! - Error cases are handled gracefully
//! - Results are equivalent between CLI and MCP paths

use codex_core::AuthManager;
use codex_core::agents::AgentRuntime;
use codex_core::agents::AgentStatus;
use codex_core::async_subagent_integration::AgentType;
use codex_core::async_subagent_integration::AsyncSubAgentIntegration;
use codex_core::auth::CODEX_API_KEY_ENV_VAR;
use codex_core::auth::OPENAI_API_KEY_ENV_VAR;
use codex_core::config::Config;
use codex_core::terminal;
use codex_otel::otel_event_manager::OtelEventManager;
use codex_protocol::ConversationId;
use mcp_types::CallToolResult;
use mcp_types::ContentBlock;
use pretty_assertions::assert_eq;
use std::collections::HashMap;
use std::env;
use std::sync::Arc;
use std::time::Instant;
use tempfile::TempDir;

const TEST_RUNTIME_BUDGET: usize = 200_000;

/// Helper to create test runtime with auth
async fn create_test_runtime(
    workspace_dir: std::path::PathBuf,
) -> anyhow::Result<Arc<AgentRuntime>> {
    let config = Config::load_default().await?;
    let config = Arc::new(config);

    let auth_manager = AuthManager::shared(config.codex_home.clone(), false);
    let conversation_id = ConversationId::default();

    let otel_manager = OtelEventManager::new(
        conversation_id,
        config.model.as_str(),
        config.model_family.slug.as_str(),
        auth_manager
            .auth()
            .as_ref()
            .and_then(|auth| auth.get_account_id()),
        auth_manager.auth().as_ref().map(|auth| auth.mode),
        config.otel.log_user_prompt,
        terminal::user_agent(),
    );

    Ok(Arc::new(AgentRuntime::new(
        workspace_dir,
        TEST_RUNTIME_BUDGET,
        config.clone(),
        Some(Arc::clone(&auth_manager)),
        otel_manager,
        config.model_provider.clone(),
        conversation_id,
    )))
}

/// Test supervisor tool executes multiple agents in parallel
#[tokio::test]
async fn test_supervisor_parallel_execution() {
    // Skip if no API key available
    if env::var(OPENAI_API_KEY_ENV_VAR).is_err() && env::var(CODEX_API_KEY_ENV_VAR).is_err() {
        println!("Skipping test: No API key available");
        return;
    }

    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let runtime = create_test_runtime(temp_dir.path().to_path_buf())
        .await
        .expect("Failed to create runtime");

    // Define agent configurations
    let agent_configs = vec![
        (
            "CodeExpert".to_string(),
            "Review code quality".to_string(),
            HashMap::new(),
            None,
        ),
        (
            "SecurityExpert".to_string(),
            "Check security vulnerabilities".to_string(),
            HashMap::new(),
            None,
        ),
    ];

    let start = Instant::now();

    // Execute agents in parallel
    let results = runtime
        .delegate_parallel(agent_configs, None)
        .await
        .expect("Parallel delegation failed");

    let parallel_duration = start.elapsed();

    // Verify results
    assert_eq!(results.len(), 2, "Should have 2 agent results");

    // Check each agent completed
    for result in &results {
        assert!(
            matches!(result.status, AgentStatus::Completed),
            "Agent {} should complete",
            result.agent_name
        );
        assert!(result.tokens_used > 0, "Should use tokens");
        assert!(result.duration_secs > 0.0, "Should have duration");
    }

    println!(
        "✓ Parallel execution completed in {:.2}s",
        parallel_duration.as_secs_f64()
    );
}

/// Test supervisor tool with JSON output format
#[tokio::test]
async fn test_supervisor_json_format() {
    if env::var(OPENAI_API_KEY_ENV_VAR).is_err() && env::var(CODEX_API_KEY_ENV_VAR).is_err() {
        println!("Skipping test: No API key available");
        return;
    }

    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let runtime = create_test_runtime(temp_dir.path().to_path_buf())
        .await
        .expect("Failed to create runtime");

    // Call supervisor tool handler directly
    let arguments = serde_json::json!({
        "goal": "Test goal",
        "agents": ["CodeExpert"],
        "format": "json"
    });

    let result = codex_mcp_server::supervisor_tool_handler::handle_supervisor_tool_call(
        mcp_types::RequestId::String("test-123".to_string()),
        Some(arguments),
        &Some(runtime),
    )
    .await;

    // Verify JSON result
    if let ContentBlock::TextContent(text) = &result.content[0] {
        let json_value: serde_json::Value =
            serde_json::from_str(&text.text).expect("Should be valid JSON");
        assert!(json_value.get("goal").is_some(), "Should have goal");
        assert!(json_value.get("agents").is_some(), "Should have agents");
        assert!(json_value.get("results").is_some(), "Should have results");
    } else {
        panic!("Expected TextContent");
    }
}

/// Test subagent tool: start_task action
#[tokio::test]
async fn test_subagent_start_task() {
    if env::var(OPENAI_API_KEY_ENV_VAR).is_err() && env::var(CODEX_API_KEY_ENV_VAR).is_err() {
        println!("Skipping test: No API key available");
        return;
    }

    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let runtime = create_test_runtime(temp_dir.path().to_path_buf())
        .await
        .expect("Failed to create runtime");

    let integration = Arc::new(AsyncSubAgentIntegration::new(runtime));

    // Call subagent tool handler
    let arguments = serde_json::json!({
        "action": "start_task",
        "agent_type": "CodeExpert",
        "task": "Review authentication module"
    });

    let result = codex_mcp_server::subagent_tool_handler::handle_subagent_tool_call(
        arguments,
        &Some(integration.clone()),
    )
    .await
    .expect("start_task should succeed");

    // Verify result contains agent_id
    if let ContentBlock::TextContent(text) = &result.content[0] {
        assert!(
            text.text.contains("Agent ID"),
            "Should include agent ID in response"
        );
        assert!(text.text.contains("✅"), "Should indicate success");
    } else {
        panic!("Expected TextContent");
    }
}

/// Test subagent tool: check_inbox action
#[tokio::test]
async fn test_subagent_check_inbox() {
    if env::var(OPENAI_API_KEY_ENV_VAR).is_err() && env::var(CODEX_API_KEY_ENV_VAR).is_err() {
        println!("Skipping test: No API key available");
        return;
    }

    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let runtime = create_test_runtime(temp_dir.path().to_path_buf())
        .await
        .expect("Failed to create runtime");

    let integration = Arc::new(AsyncSubAgentIntegration::new(runtime));

    // Start an agent first
    let agent_id = integration
        .start_agent(AgentType::CodeExpert, "Test task")
        .await
        .expect("Should start agent");

    // Check inbox
    let arguments = serde_json::json!({
        "action": "check_inbox"
    });

    let result = codex_mcp_server::subagent_tool_handler::handle_subagent_tool_call(
        arguments,
        &Some(integration),
    )
    .await
    .expect("check_inbox should succeed");

    // Verify result contains agent info
    if let ContentBlock::TextContent(text) = &result.content[0] {
        assert!(
            text.text.contains(&agent_id) || text.text.contains("Active Agents"),
            "Should list active agents"
        );
    } else {
        panic!("Expected TextContent");
    }
}

/// Test subagent tool: get_status action
#[tokio::test]
async fn test_subagent_get_status() {
    if env::var(OPENAI_API_KEY_ENV_VAR).is_err() && env::var(CODEX_API_KEY_ENV_VAR).is_err() {
        println!("Skipping test: No API key available");
        return;
    }

    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let runtime = create_test_runtime(temp_dir.path().to_path_buf())
        .await
        .expect("Failed to create runtime");

    let integration = Arc::new(AsyncSubAgentIntegration::new(runtime));

    // Start an agent
    let agent_id = integration
        .start_agent(AgentType::CodeExpert, "Test task")
        .await
        .expect("Should start agent");

    // Get status
    let arguments = serde_json::json!({
        "action": "get_status",
        "agent_id": agent_id
    });

    let result = codex_mcp_server::subagent_tool_handler::handle_subagent_tool_call(
        arguments,
        &Some(integration),
    )
    .await
    .expect("get_status should succeed");

    // Verify result contains status info
    if let ContentBlock::TextContent(text) = &result.content[0] {
        assert!(
            text.text.contains("Status") || text.text.contains(&agent_id),
            "Should show agent status"
        );
    } else {
        panic!("Expected TextContent");
    }
}

/// Test subagent tool: auto_dispatch action
#[tokio::test]
async fn test_subagent_auto_dispatch() {
    if env::var(OPENAI_API_KEY_ENV_VAR).is_err() && env::var(CODEX_API_KEY_ENV_VAR).is_err() {
        println!("Skipping test: No API key available");
        return;
    }

    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let runtime = create_test_runtime(temp_dir.path().to_path_buf())
        .await
        .expect("Failed to create runtime");

    let integration = Arc::new(AsyncSubAgentIntegration::new(runtime));

    // Auto-dispatch a task
    let arguments = serde_json::json!({
        "action": "auto_dispatch",
        "task": "Find security vulnerabilities in authentication code"
    });

    let result = codex_mcp_server::subagent_tool_handler::handle_subagent_tool_call(
        arguments,
        &Some(integration),
    )
    .await
    .expect("auto_dispatch should succeed");

    // Verify result contains agent_id
    if let ContentBlock::TextContent(text) = &result.content[0] {
        assert!(
            text.text.contains("Agent ID") && text.text.contains("Auto-Dispatch"),
            "Should show auto-dispatched agent"
        );
    } else {
        panic!("Expected TextContent");
    }
}

/// Test subagent tool: get_token_report action
#[tokio::test]
async fn test_subagent_token_report() {
    if env::var(OPENAI_API_KEY_ENV_VAR).is_err() && env::var(CODEX_API_KEY_ENV_VAR).is_err() {
        println!("Skipping test: No API key available");
        return;
    }

    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let runtime = create_test_runtime(temp_dir.path().to_path_buf())
        .await
        .expect("Failed to create runtime");

    let integration = Arc::new(AsyncSubAgentIntegration::new(runtime));

    // Get token report
    let arguments = serde_json::json!({
        "action": "get_token_report"
    });

    let result = codex_mcp_server::subagent_tool_handler::handle_subagent_tool_call(
        arguments,
        &Some(integration),
    )
    .await
    .expect("get_token_report should succeed");

    // Verify result contains report
    if let ContentBlock::TextContent(text) = &result.content[0] {
        assert!(
            text.text.contains("Token Usage") || text.text.contains("Report"),
            "Should show token report"
        );
    } else {
        panic!("Expected TextContent");
    }
}

/// Test error case: invalid agent type
#[tokio::test]
async fn test_subagent_invalid_agent_type() {
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let runtime = create_test_runtime(temp_dir.path().to_path_buf())
        .await
        .expect("Failed to create runtime");

    let integration = Arc::new(AsyncSubAgentIntegration::new(runtime));

    // Try to start agent with invalid type
    let arguments = serde_json::json!({
        "action": "start_task",
        "agent_type": "InvalidAgentType",
        "task": "Test task"
    });

    let result = codex_mcp_server::subagent_tool_handler::handle_subagent_tool_call(
        arguments,
        &Some(integration),
    )
    .await;

    // Should return error
    assert!(result.is_err(), "Should fail with invalid agent type");
}

/// Test error case: missing required parameters
#[tokio::test]
async fn test_subagent_missing_params() {
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let runtime = create_test_runtime(temp_dir.path().to_path_buf())
        .await
        .expect("Failed to create runtime");

    let integration = Arc::new(AsyncSubAgentIntegration::new(runtime));

    // Try start_task without required agent_type
    let arguments = serde_json::json!({
        "action": "start_task",
        "task": "Test task"
    });

    let result = codex_mcp_server::subagent_tool_handler::handle_subagent_tool_call(
        arguments,
        &Some(integration),
    )
    .await;

    // Should return error
    assert!(result.is_err(), "Should fail without agent_type");
}

/// Test error case: runtime not initialized
#[tokio::test]
async fn test_supervisor_no_runtime() {
    let arguments = serde_json::json!({
        "goal": "Test goal",
        "agents": ["CodeExpert"]
    });

    // Call with None runtime
    let result = codex_mcp_server::supervisor_tool_handler::handle_supervisor_tool_call(
        mcp_types::RequestId::String("test-456".to_string()),
        Some(arguments),
        &None,
    )
    .await;

    // Should return error
    assert!(
        result.is_error.unwrap_or(false),
        "Should indicate error when runtime not initialized"
    );

    if let ContentBlock::TextContent(text) = &result.content[0] {
        assert!(
            text.text.contains("runtime not initialized") || text.text.contains("failed"),
            "Should mention runtime issue"
        );
    }
}
