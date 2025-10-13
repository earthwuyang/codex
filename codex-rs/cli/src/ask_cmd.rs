use anyhow::Context;
use anyhow::Result;
use codex_common::CliConfigOverrides;
use codex_core::agents::AgentAliases;
use std::path::PathBuf;

pub async fn run_ask_command(
    config_overrides: CliConfigOverrides,
    prompt: String,
    scope: Option<PathBuf>,
    budget: Option<usize>,
    out: Option<PathBuf>,
) -> Result<()> {
    // Load aliases
    let aliases = AgentAliases::load().unwrap_or_default();

    // Check if prompt starts with @mention
    let (agent_name, task) = if AgentAliases::has_mention(&prompt) {
        let (agent, rest) =
            AgentAliases::extract_mention(&prompt).context("Failed to parse @mention")?;
        let resolved = aliases.resolve(agent);
        (resolved, rest.to_string())
    } else {
        // Default to researcher if no @mention
        ("researcher".to_string(), prompt.clone())
    };

    println!("🤖 Using agent: {agent_name}");
    println!("📝 Task: {task}\n");

    // Use the existing delegate logic
    crate::delegate_cmd::run_delegate_command(
        config_overrides,
        agent_name,
        Some(task),
        scope,
        budget,
        None, // deadline
        out,
    )
    .await
}

/// Shortcut command that automatically selects the appropriate agent
pub async fn run_shortcut_command(
    config_overrides: CliConfigOverrides,
    shortcut: &str,
    prompt: String,
    scope: Option<PathBuf>,
    budget: Option<usize>,
    out: Option<PathBuf>,
) -> Result<()> {
    let aliases = AgentAliases::load().unwrap_or_default();
    let agent_name = aliases.resolve(shortcut);

    println!("🚀 Shortcut: {shortcut} → {agent_name}");
    println!("📝 Task: {prompt}\n");

    crate::delegate_cmd::run_delegate_command(
        config_overrides,
        agent_name,
        Some(prompt),
        scope,
        budget,
        None,
        out,
    )
    .await
}
