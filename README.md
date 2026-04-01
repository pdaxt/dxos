<p align="center">
  <h1 align="center">DXOS</h1>
  <p align="center"><strong>The open-source AI agent operating system.</strong></p>
  <p align="center">One Rust binary. Any model. From solo dev to agent fleet.</p>
</p>

<p align="center">
  <a href="#install">Install</a> &bull;
  <a href="#quick-start">Quick Start</a> &bull;
  <a href="#why-dxos">Why DXOS</a> &bull;
  <a href="#architecture">Architecture</a> &bull;
  <a href="#roadmap">Roadmap</a>
</p>

---

```
$ dxos run "find and fix the null pointer exception in src/api/handler.rs"

dxos v0.1.0 — model: claude-sonnet-4 — mode: workspace-write

I found the issue. In `handler.rs:47`, the `user` variable is unwrapped
without checking for None when the session token is expired...

[reads src/api/handler.rs]
[edits src/api/handler.rs — adds Option check]
[runs cargo test]

Fixed. The handler now returns 401 instead of panicking on expired sessions.
All 23 tests pass.

--- 6 tool calls | 3 iterations | 4,218 tokens ---
```

## Why DXOS

**Claude Code** costs $200/mo. You can't see what it does. You can't change how it works.

**Cursor** locks you into their editor. **Copilot** can't run commands. Every "open-source alternative" is a wrapper around a CLI tool, not an engine.

DXOS is the engine. Rebuilt from scratch in Rust. Open-source. Designed to run **any model** — Claude, GPT, Gemini, or local.

| | Claude Code | Cursor | DXOS |
|---|---|---|---|
| Open source | No | No | **Yes** |
| Provider lock-in | Anthropic only | OpenAI default | **Any model** |
| Multi-agent | No | No | **Yes (fleet mode)** |
| Persistent memory | Plugin | No | **Built-in** |
| Self-hostable | No | No | **Yes** |
| Tool overhead | 206 MCP tools in prompt (~10k tokens) | Unknown | **7 native tools (~350 tokens)** |
| Dependencies | Node.js + npm | Electron | **Zero. One binary.** |

### The token math

Every AI coding tool burns tokens describing its tools to the model. More tools = more cost per request.

- **Claude Code**: ~206 tools in system prompt = ~10,000 tokens overhead
- **DXOS**: 7 native Rust tools = ~350 tokens overhead

At $3/MTok input, that's **$0.03 vs $0.001 per session** just for tool definitions. Over thousands of sessions, DXOS saves real money.

## Install

```bash
cargo install dxos-cli
```

Or build from source:

```bash
git clone https://github.com/pdaxt/dxos
cd dxos
cargo install --path dxos-cli
```

Set your API key:

```bash
export ANTHROPIC_API_KEY=sk-ant-...
```

## Quick Start

```bash
# Run a single agent task
dxos run "add input validation to the signup endpoint"

# Initialize project-specific config
dxos init

# Use a different model
dxos run --model gpt-4o "explain how auth works in this codebase"

# Full access mode (allows shell commands without prompting)
dxos run --permission full-access "run the test suite and fix failures"
```

### Coming in v0.2

```bash
# Spawn 8 agents to close all P0 issues
dxos fleet "close all P0 issues" --agents 8

# Query persistent memory
dxos brain "how does the payment flow work?"

# Real-time monitoring dashboard
dxos dash

# Session audit trail
dxos log
```

## Architecture

DXOS is a Cargo workspace with composable crates. Use the full CLI or pick individual crates for your own tools.

```
dxos/
├── dxos-cli/          # The `dxos` binary
├── crates/
│   ├── core/          # Types, config, errors
│   ├── tools/         # 7 native Rust tools (read, write, edit, bash, glob, grep, git)
│   ├── harness/       # Conversation runtime, permissions, session compaction
│   ├── api/           # Multi-provider LLM client (Anthropic, OpenAI, local)
│   ├── fleet/         # Multi-agent orchestration [v0.2]
│   ├── brain/         # Persistent cross-session memory [v0.2]
│   └── dashboard/     # Real-time TUI monitoring [v0.2]
```

### Why 7 tools instead of 200?

Every tool you add to the system prompt costs tokens and adds decision complexity for the model. We found that **7 tools cover 95% of coding tasks**:

| Tool | What it does | Implementation |
|---|---|---|
| `bash` | Execute shell commands | `tokio::process` with timeout |
| `read_file` | Read files with line numbers | `std::fs` with offset/limit |
| `write_file` | Write files atomically | `std::fs` with parent dir creation |
| `edit_file` | Find-and-replace in files | String matching with uniqueness check |
| `glob` | Find files by pattern | `ignore` crate (same as ripgrep) |
| `grep` | Search file contents | `regex` + `ignore` (respects .gitignore) |
| `git` | Git operations | Shell passthrough |

All tools run as **native Rust function calls** inside the same process. No subprocess spawning, no JSON-RPC overhead, no MCP protocol negotiation.

### Conversation Runtime

The harness manages the agent loop:

```
User prompt
    ↓
System prompt + tool definitions (350 tokens, not 10,000)
    ↓
LLM generates response + tool calls
    ↓
Permission check (read-only / workspace-write / full-access)
    ↓
Native tool execution (no subprocess overhead)
    ↓
Results fed back to LLM
    ↓
Loop until done or turn limit
    ↓
Session compaction if context grows large
```

## Roadmap

### v0.1 — Solo Agent (current)
- [x] Native Rust tool implementations
- [x] Conversation runtime with turn loop
- [x] Permission gating (read-only → workspace-write → full-access)
- [x] Session compaction
- [x] Anthropic API client
- [x] CLI with `dxos run`
- [x] CLAUDE.md / DXOS.md project instruction loading

### v0.2 — Fleet + Brain
- [ ] Multi-agent fleet on isolated git worktrees
- [ ] Persistent SQLite-backed memory (brain)
- [ ] Real-time TUI dashboard (Ratatui)
- [ ] OpenAI provider
- [ ] Session logging and cost tracking
- [ ] SSE streaming output

### v0.3 — Local + Enterprise
- [ ] Local model support (Ollama, vLLM)
- [ ] Agent governance and audit trails
- [ ] Web dashboard
- [ ] Plugin system

## Contributing

We welcome contributions. The codebase is designed to be approachable:

- **Add a provider**: Implement `ApiClient` trait in `crates/api/`
- **Add a tool**: Add a function in `crates/tools/` and register in `registry.rs`
- **Improve the harness**: The conversation loop is in `crates/harness/src/runtime.rs`

```bash
cargo test          # Run all tests
cargo clippy        # Lint
cargo build --release  # Build release binary (~15MB)
```

## Acknowledgments

DXOS builds on ideas from the AI coding agent community — particularly the architectural patterns documented in [claw-code](https://github.com/instructkr/claw-code) and the orchestration patterns from [dx-terminal](https://github.com/pdaxt/dx-terminal).

## License

Apache-2.0
