// SubAgent Tool Handler - Real Implementation
use anyhow::Result;
use codex_core::async_subagent_integration::AgentType;
use codex_core::async_subagent_integration::AsyncSubAgentIntegration;
use mcp_types::CallToolResult;
use mcp_types::ContentBlock;
use mcp_types::TextContent;
use serde_json::Value;
use std::sync::Arc;
use tracing::{debug, error, info, warn};

use crate::subagent_tool::SubAgentToolParam;

pub async fn handle_subagent_tool_call(
    arguments: Value,
    async_integration: &Option<Arc<AsyncSubAgentIntegration>>,
) -> Result<CallToolResult> {
    let params: SubAgentToolParam = serde_json::from_value(arguments)?;

    debug!("SubAgent tool called with action: {}", params.action);

    // Check if async_integration is available
    let integration = async_integration.as_ref().ok_or_else(|| {
        error!(
            "AsyncSubAgentIntegration not initialized when trying action: {}",
            params.action
        );
        anyhow::anyhow!("AsyncSubAgentIntegration not initialized")
    })?;

    let response_text = match params.action.as_str() {
        "start_task" => {
            let agent_type_str = params
                .agent_type
                .ok_or_else(|| anyhow::anyhow!("agent_type required for start_task"))?;
            let task = params
                .task
                .ok_or_else(|| anyhow::anyhow!("task required for start_task"))?;

            // Parse agent type
            let agent_type = parse_agent_type(&agent_type_str)?;

            // Start the agent
            let agent_id = integration
                .start_agent(agent_type, &task)
                .await
                .map_err(|e| {
                    error!("Failed to start agent {}: {}", agent_type.as_str(), e);
                    e
                })?;

            info!(
                "Started agent: {} with id: {}",
                agent_type.as_str(),
                agent_id
            );

            format!(
                "✅ SubAgent Started\n\n\
                **Agent Type**: {}\n\
                **Agent ID**: {}\n\
                **Task**: {}\n\n\
                **Status**: Agent is now running in the background.\n\n\
                **Next Steps**:\n\
                - Use `get_status` with agent_id to check progress\n\
                - Use `get_thinking` with agent_id to see reasoning\n\
                - Use `get_token_report` to track token usage",
                agent_type_str, agent_id, task
            )
        }
        "check_inbox" => {
            debug!("Checking inbox for active agents");
            // Get all agent states as a proxy for "inbox"
            let states = integration.get_agent_states().await;

            if states.is_empty() {
                "📬 Inbox\n\nNo active agents or notifications.".to_string()
            } else {
                let mut output = String::from("📬 Active Agents\n\n");
                for state in states {
                    output.push_str(&format!(
                        "- **{}** ({}): {} - {:.1}% complete\n",
                        state.agent_id,
                        state.agent_type.as_str(),
                        state.status,
                        state.progress
                    ));
                }
                output
            }
        }
        "get_status" => {
            let agent_id = params
                .task_id
                .as_ref()
                .or(params.agent_id.as_ref())
                .ok_or_else(|| anyhow::anyhow!("agent_id or task_id required for get_status"))?;

            // Generate task summary
            let summary = integration.generate_task_summary(agent_id).await;

            format!("🤖 SubAgent Status\n\n{}", summary)
        }
        "auto_dispatch" => {
            let task = params
                .task
                .ok_or_else(|| anyhow::anyhow!("task required for auto_dispatch"))?;

            // Auto-dispatch task and start agent
            let agent_id = integration.auto_dispatch_task(&task).await?;

            format!(
                "🎯 Auto-Dispatch Complete\n\n\
                **Agent ID**: {}\n\
                **Task**: {}\n\n\
                **Status**: Agent has been automatically selected and started.\n\n\
                Use `get_status` with agent_id={} to check progress.",
                agent_id, task, agent_id
            )
        }
        "get_thinking" => {
            if let Some(task_id) = params.task_id.as_ref().or(params.agent_id.as_ref()) {
                debug!("Getting thinking process for: {}", task_id);
                // Get thinking summary for specific task
                let thinking = integration
                    .get_thinking_summary(task_id)
                    .await
                    .unwrap_or_else(|| format!("No thinking process found for {}", task_id));

                format!(
                    "💭 Thinking Process\n\n**Task ID**: {}\n\n{}",
                    task_id, thinking
                )
            } else {
                // Get all thinking summaries
                let all_thinking = integration.get_all_thinking_summaries().await;

                format!("💭 All Thinking Processes\n\n{}", all_thinking)
            }
        }
        "get_token_report" => {
            debug!("Generating token usage report");
            let report = integration.generate_token_report().await;

            format!("📊 Token Usage Report\n\n{}", report)
        }
        _ => {
            return Err(anyhow::anyhow!("Unknown action: {}", params.action));
        }
    };

    Ok(CallToolResult {
        content: vec![ContentBlock::TextContent(TextContent {
            r#type: "text".to_string(),
            text: response_text,
            annotations: None,
        })],
        is_error: None,
        structured_content: None,
    })
}

/// Parse agent type string into AgentType enum
fn parse_agent_type(type_str: &str) -> Result<AgentType> {
    debug!("Parsing agent type: {}", type_str);
    match type_str.to_lowercase().as_str() {
        "codeexpert" | "code-reviewer" | "code-expert" => Ok(AgentType::CodeExpert),
        "securityexpert" | "sec-audit" | "security-expert" | "security" => {
            Ok(AgentType::SecurityExpert)
        }
        "testingexpert" | "test-gen" | "testing-expert" | "tester" => Ok(AgentType::TestingExpert),
        "docsexpert" | "docs-expert" | "documentation" => Ok(AgentType::DocsExpert),
        "deepresearcher" | "researcher" | "research" => Ok(AgentType::DeepResearcher),
        "debugexpert" | "debug-expert" | "debugger" => Ok(AgentType::DebugExpert),
        "performanceexpert" | "perf-expert" | "performance" => Ok(AgentType::PerformanceExpert),
        "general" => Ok(AgentType::General),
        _ => Err(anyhow::anyhow!("Unknown agent type: {}", type_str)),
    }
}
