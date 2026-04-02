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

<p align="center">
  <img src="assets/demo.gif" alt="DXOS demo — fixing a null pointer in 4 seconds" width="800">
</p>

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
- **DXOS**: 8 native Rust tools = ~400 tokens overhead

At $3/MTok input, that's **$0.03 vs $0.001 per session** just for tool definitions. Over thousands of sessions, DXOS saves real money.

## Install

```bash
git clone https://github.com/pdaxt/dxos
cd dxos
cargo install --path dxos-cli
```

**No API key needed** — DXOS auto-detects the best available model:

```bash
# Option 1: Free local models (recommended)
brew install ollama && ollama pull qwen2.5-coder:32b
dxos setup   # interactive model picker

# Option 2: Any API key
export ANTHROPIC_API_KEY=sk-ant-...   # Claude
export OPENAI_API_KEY=sk-...          # GPT-4o
export OPENROUTER_API_KEY=sk-...      # 100+ models
```

## Quick Start

```bash
# Interactive REPL (recommended)
dxos chat

# One-shot task
dxos run "fix the auth bug in handler.rs"

# Use a specific model
dxos chat --model qwen2.5-coder:32b

# Full access mode (no permission prompts)
dxos run --permission full-access "run the test suite and fix failures"

# Initialize project config
dxos init
```

### Features

- **SSE streaming** — text appears token-by-token as the model generates it
- **Animated spinner** — cycling dots with rotating verbs while thinking
- **REPL history** — up-arrow recall, persistent across sessions
- **3-layer context compression** — MicroCompact → AutoCompact → Emergency
- **Agent mode** — uses tools directly, never suggests commands for you to copy
- **Smart model detection** — auto-picks the best available: Ollama → Anthropic → OpenAI → OpenRouter

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

### Why 8 tools instead of 200?

Every tool you add to the system prompt costs tokens and adds decision complexity for the model. 8 tools cover 95% of coding tasks:

| Tool | What it does | Implementation |
|---|---|---|
| `bash` | Execute shell commands | `tokio::process` with timeout |
| `read_file` | Read files with line numbers | `std::fs` with offset/limit |
| `write_file` | Write files atomically | `std::fs` with parent dir creation |
| `edit_file` | Find-and-replace in files | String matching with uniqueness check |
| `glob` | Find files by pattern | `ignore` crate (same as ripgrep) |
| `grep` | Search file contents | `regex` + `ignore` (respects .gitignore) |
| `git` | Git operations | Shell passthrough |
| `web_fetch` | Fetch URL content | `reqwest` with HTML stripping |

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
- [x] 8 native Rust tool implementations
- [x] Conversation runtime with turn loop
- [x] Permission gating (read-only → workspace-write → full-access)
- [x] 3-layer context compression (MicroCompact → AutoCompact → Emergency)
- [x] SSE streaming output (token-by-token)
- [x] Animated spinner with cycling dots and rotating verbs
- [x] Interactive REPL with readline history
- [x] Multi-provider: Anthropic, OpenAI, OpenRouter, Ollama/local
- [x] Smart model auto-detection
- [x] Interactive model setup with system detection
- [x] Agent-mode system prompt (uses tools, doesn't suggest commands)
- [x] Text-based tool extraction (works with any model)
- [x] CLAUDE.md / DXOS.md project instruction loading
- [x] 37 tests

### v0.2 — Fleet + Brain
- [ ] Multi-agent fleet on isolated git worktrees
- [ ] Persistent SQLite-backed memory (brain)
- [ ] Real-time TUI dashboard (Ratatui)
- [ ] Session logging and cost tracking
- [ ] Extended thinking mode display

### v0.3 — Enterprise
- [ ] Agent governance and audit trails
- [ ] Web dashboard
- [ ] Plugin system
- [ ] IDE bridge (VS Code, JetBrains)

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

DXOS is original work built from scratch in Rust, informed by publicly documented agent design patterns and the open-source AI coding community.

## License

Apache-2.0
