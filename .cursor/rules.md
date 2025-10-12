# 🎯 Codex Project Rules - OpenAI Best Practices + zapabob Extensions

**Version**: 0.47.0-alpha.1  
**Last Updated**: 2025-10-12  
**Based on**: OpenAI/codex official recommendations + zapabob enhancements

---

## 📋 Table of Contents

1. [Project Overview](#-project-overview)
2. [Core Principles](#-core-principles)
3. [Model Selection Strategy](#-model-selection-strategy)
4. [Security & Sandbox](#-security--sandbox)
5. [Sub-Agent System](#-sub-agent-system)
6. [Deep Research](#-deep-research)
7. [Coding Standards](#-coding-standards)
8. [Build & Development](#️-build--development)
9. [Testing Requirements](#-testing-requirements)
10. [Documentation](#-documentation)

---

## 🎯 Project Overview

**Codex Multi-Agent System** - AI-powered coding assistant with specialized sub-agents and deep research capabilities.

### Key Components

- **Codex Core (Rust)**: High-performance runtime with MCP integration
- **CLI Interface**: User-facing command-line tool
- **Sub-Agent Runtime**: Parallel execution engine for specialized agents
- **Deep Research Engine**: Multi-source research with citation management
- **MCP Servers**: Extensible tool integration via Model Context Protocol

### Repository Structure

```
codex-main/
├── codex-rs/          # Rust core implementation
│   ├── cli/           # Command-line interface
│   ├── core/          # Core runtime and agent execution
│   ├── protocol/      # MCP protocol implementation
│   └── tui/           # Terminal user interface
├── .codex/            # Agent definitions and configurations
│   └── agents/        # Sub-agent YAML definitions
├── _docs/             # Implementation logs (auto-generated)
├── examples/          # Usage examples and demos
└── scripts/           # Build and deployment scripts
```

---

## 🌟 Core Principles

### 1. OpenAI Official Best Practices

✅ **Flexibility**: Model selection via CLI flags  
✅ **Security**: Restrictive sandbox by default  
✅ **Explicitness**: Clear approval policies  
✅ **Traceability**: Comprehensive audit logging

### 2. zapabob Enhancements

✅ **Specialization**: Domain-specific sub-agents  
✅ **Parallelism**: Concurrent task execution  
✅ **Research**: Deep, cited, cross-validated reports  
✅ **Budget Control**: Token usage management

### 3. Development Philosophy

✅ **Rust-First**: Performance-critical code in Rust  
✅ **Type Safety**: Strict type checking, no `any`/`unsafe`  
✅ **Test Coverage**: 80%+ for critical paths  
✅ **Documentation**: Self-documenting code + auto-generated logs

---

## 🤖 Model Selection Strategy

### Recommended Models by Task Type

| Task Type | Model | Reasoning |
|-----------|-------|-----------|
| Quick fixes, formatting | `gpt-4o-mini` | Fast, cost-effective |
| Standard development | `gpt-4o` | Balanced performance |
| Complex refactoring | `gpt-4o` | Strong code understanding |
| Algorithm design | `o1-preview` | Superior reasoning |
| Documentation | `gpt-4o-mini` | Sufficient for text |
| Security auditing | `gpt-4o` | Detailed analysis |

### Usage Examples

```bash
# Explicit model selection (recommended)
codex --model gpt-4o-mini "Rename variable foo to bar"
codex --model gpt-4o "Implement JWT authentication"
codex --model o1-preview "Optimize sorting algorithm"

# Use default model from config
codex "Simple task with default model"
```

### Configuration

```toml
# ~/.codex/config.toml
model = "gpt-4o"  # Sensible default, override with --model flag

[model_providers.openai]
base_url = "https://api.openai.com/v1"
env_key = "OPENAI_API_KEY"
wire_api = "chat"
requires_openai_auth = true
```

---

## 🔒 Security & Sandbox

### Default Security Posture

**Principle**: Start restrictive, explicitly enable when needed.

```toml
# ~/.codex/config.toml
[sandbox]
default_mode = "read-only"  # Safe default

[sandbox_permissions]
workspace_write = true       # Allow within workspace
disk_full_read_access = false  # NO full disk access
network_access = false       # NO network by default

[approval]
policy = "on-request"        # Ask before executing
```

### Sandbox Modes

| Mode | Description | Use Case |
|------|-------------|----------|
| `read-only` | Read files only | Analysis, exploration |
| `workspace-write` | Write within workspace | Development, refactoring |
| `danger-full-access` | Full system access | ⚠️ Automated scripts (explicit only) |

### Usage

```bash
# Safe analysis (default)
codex "Analyze codebase structure"

# Allow file modifications
codex --sandbox=workspace-write "Refactor auth module"

# Dangerous operations (EXPLICIT)
codex --sandbox=danger-full-access --approval=never "Deploy to production"
```

### 🚫 Prohibited Actions

❌ **NEVER** hardcode API keys in config files  
❌ **NEVER** use `danger-full-access` as default  
❌ **NEVER** disable approval in untrusted environments  
❌ **NEVER** execute shell commands without sandboxing

---

## 🤖 Sub-Agent System

### Available Agents

| Agent | Purpose | Token Budget | Scope |
|-------|---------|--------------|-------|
| `code-reviewer` | Security, performance, best practices | 40,000 | TypeScript, Python, Rust, C# Unity |
| `test-gen` | Unit/Integration/E2E test generation | 30,000 | 80%+ coverage goal |
| `sec-audit` | CVE scanning, dependency audit | 50,000 | All dependencies |
| `researcher` | Deep research with citations | 60,000 | Multi-source validation |

### Usage

#### Single Agent

```bash
# Code review
codex delegate code-reviewer --scope ./src

# Test generation
codex delegate test-gen --scope ./tests --budget 30000

# Security audit
codex delegate sec-audit --scope ./package.json
```

#### Parallel Execution (3x faster!)

```bash
# Review + Test + Security in parallel
codex delegate-parallel code-reviewer,test-gen,sec-audit \
  --goals "Review code,Generate tests,Security audit" \
  --scopes ./src,./tests,./package.json \
  --budgets 40000,30000,20000
```

#### Custom Agent Creation

```bash
# Create agent from natural language
codex agent-create "Find all TODO comments and create a summary report" \
  --budget 50000
```

### Agent Definition (YAML)

```yaml
# .codex/agents/code-reviewer.yaml
name: code-reviewer
version: "1.0.0"
description: "Multi-language code reviewer with security focus"

capabilities:
  languages:
    - typescript
    - python
    - rust
    - csharp_unity

checks:
  - type_safety
  - security_vulnerabilities
  - performance_optimization
  - best_practices

token_budget: 40000
sandbox_mode: read-only
approval_policy: never  # Auto-approve for reviews
```

### Best Practices

✅ **Specialize**: Use dedicated agents for specific tasks  
✅ **Parallelize**: Run independent agents concurrently  
✅ **Budget**: Set appropriate token limits  
✅ **Isolate**: Each agent runs in separate process

---

## 🔍 Deep Research

### Research Strategies

| Strategy | Depth | Sources | Use Case |
|----------|-------|---------|----------|
| `focused` | 2 | 3-5 | Specific questions |
| `comprehensive` | 3-5 | 5-10 | Deep investigation |
| `exploratory` | 1-2 | 10+ | Broad survey |

### Usage

```bash
# Quick research
codex research "React Server Components best practices"

# Deep dive
codex research "Rust async error handling" --depth 5 --strategy comprehensive

# Broad survey
codex research "Modern web frameworks" --strategy exploratory
```

### Configuration

```toml
# ~/.codex/config.toml
[deep_research]
enabled = true
max_depth = 3
max_sources = 5
default_strategy = "focused"
require_citations = true
contradiction_detection = true
```

### Research Output

```markdown
# Research Report: Rust Async Error Handling

## Executive Summary
...

## Key Findings
1. Use `Result<T, E>` for recoverable errors [[Source 1]](#source-1)
2. Avoid `panic!()` in async contexts [[Source 2]](#source-2)

## Contradictions Detected
⚠️ Source 3 recommends `unwrap()` while Source 1/2 advise against it.
   Resolution: Use `unwrap()` only in tests or infallible cases.

## Citations
- [Source 1]: Rust Async Book (https://...)
- [Source 2]: Tokio Documentation (https://...)
```

---

## 💻 Coding Standards

### TypeScript/JavaScript

#### Rules

```typescript
// ✅ GOOD: Explicit types
function getUserById(id: number): Promise<User | null> {
  return database.findUser(id);
}

// ❌ BAD: any type
function getUserById(id: any): any {
  return database.findUser(id);
}

// ✅ GOOD: Optional chaining
const userName = user?.profile?.name ?? 'Anonymous';

// ❌ BAD: Nested conditionals
const userName = user && user.profile && user.profile.name 
  ? user.profile.name 
  : 'Anonymous';
```

#### Conventions

- ✅ Use `const` by default, `let` only when reassignment needed
- ✅ Prefer `async/await` over `.then()` chains
- ✅ Use optional chaining (`?.`) and nullish coalescing (`??`)
- ✅ Follow React Hooks rules strictly
- ❌ NEVER use `any` type
- ❌ NEVER use `var`

### Python

#### Rules

```python
# ✅ GOOD: Type hints
def calculate_total(items: list[Item]) -> Decimal:
    return sum(item.price for item in items)

# ❌ BAD: No type hints
def calculate_total(items):
    return sum(item.price for item in items)

# ✅ GOOD: pathlib
from pathlib import Path
config_path = Path.home() / ".config" / "app.toml"

# ❌ BAD: os.path
import os
config_path = os.path.join(os.path.expanduser("~"), ".config", "app.toml")
```

#### Conventions

- ✅ Follow PEP 8 style guide
- ✅ Use type hints (PEP 484)
- ✅ Use `pathlib` instead of `os.path`
- ✅ Use list comprehensions
- ✅ Format with Black
- ❌ NEVER use mutable default arguments

### Rust

#### Rules

```rust
// ✅ GOOD: Inline format arguments
println!("User {name} has {count} items");

// ❌ BAD: Non-inlined format arguments
println!("User {} has {} items", name, count);

// ✅ GOOD: Iterator chains
let total: i32 = numbers.iter().filter(|&&x| x > 0).sum();

// ❌ BAD: Explicit loops
let mut total = 0;
for &x in &numbers {
    if x > 0 {
        total += x;
    }
}

// ✅ GOOD: Method reference
items.iter().map(Item::price)

// ❌ BAD: Redundant closure
items.iter().map(|item| item.price())
```

#### Conventions

- ✅ Follow Clippy lints (all categories)
- ✅ Use inline format arguments (`println!("{name}")`)
- ✅ Prefer iterators over explicit loops
- ✅ Use method references over closures
- ✅ Collapse nested `if` statements
- ❌ NEVER use `unsafe` without justification and review
- ❌ NEVER use `clone()` unnecessarily

### C# Unity

#### Rules

```csharp
// ✅ GOOD: SerializeField with private
[SerializeField] private float speed = 5f;

// ❌ BAD: Public field
public float speed = 5f;

// ✅ GOOD: Object pooling
private Queue<GameObject> bulletPool = new();

void SpawnBullet() {
    var bullet = bulletPool.Count > 0 
        ? bulletPool.Dequeue() 
        : Instantiate(bulletPrefab);
}

// ❌ BAD: Instantiate in Update
void Update() {
    if (Input.GetKeyDown(KeyCode.Space)) {
        Instantiate(bulletPrefab);  // GC allocation!
    }
}
```

#### Conventions

- ✅ Use `[SerializeField]` for inspector-visible fields
- ✅ Implement object pooling for frequently created objects
- ✅ Use ScriptableObject for configuration
- ✅ Prefer async methods over Coroutines for I/O
- ❌ NEVER allocate in `Update()` or `FixedUpdate()`
- ❌ NEVER use `GetComponent()` in `Update()`
- ❌ NEVER use `Find()` or `GameObject.Find()` in loops

---

## 🛠️ Build & Development

### Rust Build Process

#### Standard Build

```powershell
# Navigate to Rust workspace
cd codex-rs

# Format code (automatic, no approval needed)
just fmt

# Fix linter issues (project-specific)
just fix -p codex-cli

# Build release
cargo build --release -p codex-cli

# Install globally
cargo install --path cli --force
```

#### Clean Build (After Major Changes)

```powershell
cd codex-rs

# Clean all artifacts
cargo clean

# Full rebuild
cargo build --release -p codex-cli

# Verify installation
codex --version
# Expected: codex-cli 0.47.0-alpha.1
```

#### Quick Scripts

```powershell
# Fast build and install
.\build-and-install.ps1

# Clean build and install
.\clean-build-install.ps1

# Emergency repair (if corrupted)
.\emergency-repair.ps1
```

### Testing

#### Project-Specific Tests

```bash
# Test specific crate
cargo test -p codex-tui

# Test with features
cargo test --all-features -p codex-core
```

#### Snapshot Tests

```bash
# Run tests (generates .snap.new files)
cargo test -p codex-tui

# Review pending snapshots
cargo insta pending-snapshots -p codex-tui

# Accept all snapshots (if intentional)
cargo insta accept -p codex-tui
```

### Linting

```bash
# Format check
cargo fmt --check

# Clippy (project-specific)
cargo clippy -p codex-cli -- -D warnings

# Full workspace Clippy (only if core/protocol changed)
cargo clippy --all-targets --all-features -- -D warnings
```

---

## 🧪 Testing Requirements

### Coverage Goals

| Test Type | Target | Priority |
|-----------|--------|----------|
| Unit Tests | 80%+ | High |
| Integration Tests | 100% main flows | High |
| E2E Tests | 100% critical paths | Critical |
| Snapshot Tests | UI components | Medium |

### Test Structure

```rust
use pretty_assertions::assert_eq;
use core_test_support::responses;

#[tokio::test]
async fn test_delegate_code_reviewer() {
    let mock = responses::mount_sse_once(&server, responses::sse(vec![
        responses::ev_response_created("resp-1"),
        responses::ev_function_call(call_id, "shell", &args_json),
        responses::ev_completed("resp-1"),
    ])).await;

    codex.submit(Op::UserTurn { ... }).await?;

    let request = mock.single_request();
    assert_eq!(request.function_call_output(call_id), expected);
}
```

### Best Practices

- ✅ Use `pretty_assertions::assert_eq` for better diffs
- ✅ Use `core_test_support::responses` for integration tests
- ✅ Compare entire objects, not individual fields
- ✅ Use snapshot tests for TUI rendering
- ❌ NEVER skip tests without `#[cfg(test)]` guard

---

## 📝 Documentation

### Auto-Generated Implementation Logs

After completing any feature, automatically generate a log:

```bash
# Get current time
codex mcp time get_current_time --timezone "Asia/Tokyo"

# Create log file
# Format: _docs/yyyy-mm-dd_feature-name.md
```

#### Log Template

```markdown
# Implementation Log: [Feature Name]

**Date**: 2025-10-12  
**Author**: AI Assistant  
**Status**: ✅ Completed

## Overview
Brief description of the feature.

## Implementation Details
- Key changes made
- Files modified
- New dependencies

## Testing
- Test cases added
- Coverage achieved

## Known Issues
- Any limitations or TODOs

## References
- Related PRs
- Documentation links
```

### Code Documentation

```rust
/// Executes a sub-agent with the given configuration.
///
/// # Arguments
/// * `agent_name` - The name of the agent to execute
/// * `scope` - The file/directory scope for the agent
/// * `budget` - Token budget limit
///
/// # Returns
/// * `Result<AgentOutput, AgentError>` - The agent's output or error
///
/// # Example
/// ```
/// let output = execute_agent("code-reviewer", "./src", 40000).await?;
/// ```
pub async fn execute_agent(
    agent_name: &str,
    scope: &Path,
    budget: usize,
) -> Result<AgentOutput, AgentError> {
    // Implementation
}
```

---

## 🚀 Quick Reference

### Common Commands

```bash
# Code review
codex delegate code-reviewer --scope ./src

# Parallel execution
codex delegate-parallel code-reviewer,test-gen --scopes ./src,./tests

# Deep research
codex research "Topic" --depth 3

# Custom agent
codex agent-create "Task description"

# Resume session
codex resume

# Check status
codex login status
```

### File Paths

```
Config:     ~/.codex/config.toml
Agents:     .codex/agents/*.yaml
Logs:       _docs/yyyy-mm-dd_feature.md
Scripts:    codex-rs/scripts/
Tests:      codex-rs/*/tests/
```

### Environment Variables

```bash
export OPENAI_API_KEY="sk-..."
export RUST_LOG="info"
export CODEX_CONFIG_PATH="~/.codex/config.toml"
```

---

## ⚠️ Common Pitfalls

### ❌ Don't Do This

```toml
# BAD: API key in config
api_key = "sk-..."  # NEVER!

# BAD: Overly permissive
default_mode = "danger-full-access"

# BAD: Unlimited budget
token_budget = 999999999
```

### ✅ Do This Instead

```toml
# GOOD: Environment variable
env_key = "OPENAI_API_KEY"

# GOOD: Restrictive default
default_mode = "read-only"

# GOOD: Reasonable limit
token_budget = 40000
```

---

## 📊 Performance Optimization

### Sub-Agent Configuration by Project Size

| Project Size | max_parallel | token_budget | Strategy |
|--------------|--------------|--------------|----------|
| Small (<1K LOC) | 2 | 5,000 | Sequential |
| Medium (1K-10K) | 4 | 10,000 | Hybrid |
| Large (10K-100K) | 8 | 20,000 | Parallel |
| Monorepo (100K+) | 16 | 40,000 | Parallel |

### Model Selection by Task Complexity

- **Simple** (formatting, renaming): `gpt-4o-mini`
- **Standard** (features, refactoring): `gpt-4o`
- **Complex** (algorithms, architecture): `o1-preview`

---

## 🎯 Summary

### OpenAI Official Compliance

✅ Flexible model selection  
✅ Secure sandbox by default  
✅ Explicit approval policies  
✅ Proper provider configuration  
✅ Session management  
✅ Comprehensive logging

### zapabob Extensions

✅ Specialized sub-agents  
✅ Parallel execution  
✅ Deep research with citations  
✅ Token budget management  
✅ Audit logging  
✅ Model inheritance

### Result

**Production-ready Codex with OpenAI best practices + powerful zapabob enhancements** 🚀

---

**Version**: 0.47.0-alpha.1  
**Maintained by**: zapabob  
**Based on**: OpenAI/codex official recommendations  
**Status**: ✅ Production Ready

