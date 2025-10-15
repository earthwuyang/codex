//! Handler for supervisor tool calls via MCP.

use crate::supervisor_tool::SupervisorToolParam;
use codex_core::agents::AgentRuntime;
use mcp_types::CallToolResult;
use mcp_types::ContentBlock;
use mcp_types::RequestId;
use mcp_types::TextContent;
use serde_json::json;
use std::collections::HashMap;
use std::sync::Arc;
use tracing::{debug, error, info, warn};

/// Handle a supervisor tool call.
pub async fn handle_supervisor_tool_call(
    id: RequestId,
    arguments: Option<serde_json::Value>,
    agent_runtime: &Option<Arc<AgentRuntime>>,
) -> CallToolResult {
    debug!("Supervisor tool called with request_id: {:?}", id);

    let params = match arguments {
        Some(json_val) => match serde_json::from_value::<SupervisorToolParam>(json_val) {
            Ok(p) => p,
            Err(e) => {
                error!("Invalid supervisor parameters: {}", e);
                return CallToolResult {
                    content: vec![ContentBlock::TextContent(TextContent {
                        r#type: "text".to_string(),
                        text: format!("Invalid supervisor parameters: {}", e),
                        annotations: None,
                    })],
                    is_error: Some(true),
                    structured_content: None,
                };
            }
        },
        None => {
            warn!("Supervisor called without parameters");
            return CallToolResult {
                content: vec![ContentBlock::TextContent(TextContent {
                    r#type: "text".to_string(),
                    text: "Missing supervisor parameters".to_string(),
                    annotations: None,
                })],
                is_error: Some(true),
                structured_content: None,
            };
        }
    };

    // Execute supervisor coordination
    info!(
        "Executing supervisor with goal: '{}', agents: {:?}",
        params.goal,
        params.agents.as_ref().unwrap_or(&vec![])
    );

    let result_text = match execute_supervisor(&params, agent_runtime).await {
        Ok(output) => {
            info!("Supervisor coordination completed successfully");
            if params.format == "json" {
                output
            } else {
                format!(
                    "# Supervisor Coordination Result\n\n\
                     **Goal**: {}\n\n\
                     **Agents**: {:?}\n\n\
                     **Strategy**: {}\n\n\
                     ## Results\n\n\
                     {}",
                    params.goal,
                    params.agents.as_ref().unwrap_or(&vec![]),
                    params.strategy.as_ref().unwrap_or(&"default".to_string()),
                    output
                )
            }
        }
        Err(e) => {
            error!("Supervisor execution failed: {}", e);
            return CallToolResult {
                content: vec![ContentBlock::TextContent(TextContent {
                    r#type: "text".to_string(),
                    text: format!("Supervisor execution failed: {}", e),
                    annotations: None,
                })],
                is_error: Some(true),
                structured_content: None,
            };
        }
    };

    CallToolResult {
        content: vec![ContentBlock::TextContent(TextContent {
            r#type: "text".to_string(),
            text: result_text,
            annotations: None,
        })],
        is_error: None,
        structured_content: None,
    }
}

/// Execute the supervisor coordination.
async fn execute_supervisor(
    params: &SupervisorToolParam,
    agent_runtime: &Option<Arc<AgentRuntime>>,
) -> anyhow::Result<String> {
    // Check if runtime is available
    let runtime = agent_runtime
        .as_ref()
        .ok_or_else(|| anyhow::anyhow!("Agent runtime not initialized"))?;

    // Parse agents list (default to CodeExpert if not specified)
    let agent_names = params
        .agents
        .as_ref()
        .map(|v| v.clone())
        .unwrap_or_else(|| vec!["CodeExpert".to_string()]);

    // Build agent_configs for delegate_parallel
    // Format: Vec<(agent_name, goal, inputs, budget)>
    let agent_configs: Vec<(String, String, HashMap<String, String>, Option<usize>)> = agent_names
        .iter()
        .map(|agent_name| {
            (
                agent_name.clone(),
                params.goal.clone(),
                HashMap::new(), // Empty inputs
                None,           // No per-agent budget limit
            )
        })
        .collect();

    // Call delegate_parallel
    debug!(
        "Starting parallel execution of {} agents",
        agent_names.len()
    );
    let results = runtime
        .delegate_parallel(agent_configs, params.deadline)
        .await
        .map_err(|e| {
            error!("Parallel delegation failed: {}", e);
            e
        })?;

    // Log results summary
    let success_count = results
        .iter()
        .filter(|r| matches!(r.status, codex_core::agents::AgentStatus::Completed))
        .count();
    info!(
        "Parallel execution complete: {}/{} agents succeeded",
        success_count,
        results.len()
    );

    // Log individual agent results
    for result in &results {
        if let Some(ref error) = result.error {
            warn!("Agent '{}' failed: {}", result.agent_name, error);
        } else {
            debug!(
                "Agent '{}' completed successfully ({}tokens, {:.2}s)",
                result.agent_name, result.tokens_used, result.duration_secs
            );
        }
    }

    // Format results based on params.format
    if params.format == "json" {
        // Return JSON format
        Ok(serde_json::to_string_pretty(&json!({
            "goal": params.goal,
            "agents": agent_names,
            "strategy": params.strategy.as_ref().unwrap_or(&"parallel".to_string()),
            "merge_strategy": params.merge_strategy.as_ref().unwrap_or(&"concatenate".to_string()),
            "results": results,
        }))?)
    } else {
        // Return markdown format
        let mut output = format!(
            "# Supervisor Coordination Complete\n\n\
             **Goal**: {}\n\
             **Agents**: {}\n\
             **Strategy**: {}\n\n\
             ## Execution Results\n\n",
            params.goal,
            agent_names.join(", "),
            params.strategy.as_ref().unwrap_or(&"parallel".to_string())
        );

        for (i, result) in results.iter().enumerate() {
            output.push_str(&format!(
                "### Agent {}: {}\n\
                 - **Status**: {:?}\n\
                 - **Tokens Used**: {}\n\
                 - **Duration**: {:.2}s\n",
                i + 1,
                result.agent_name,
                result.status,
                result.tokens_used,
                result.duration_secs
            ));

            if !result.artifacts.is_empty() {
                output.push_str("- **Artifacts**:\n");
                for artifact in &result.artifacts {
                    output.push_str(&format!("  - {}\n", artifact));
                }
            }

            if let Some(ref error) = result.error {
                output.push_str(&format!("- **Error**: {}\n", error));
            }

            output.push('\n');
        }

        let success_count = results
            .iter()
            .filter(|r| matches!(r.status, codex_core::agents::AgentStatus::Completed))
            .count();

        output.push_str(&format!(
            "## Summary\n\
             - **Total Agents**: {}\n\
             - **Successful**: {}\n\
             - **Failed**: {}\n",
            results.len(),
            success_count,
            results.len() - success_count
        ));

        Ok(output)
    }
}
