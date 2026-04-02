<p align="center">
  <h1 align="center">DXOS</h1>
  <p align="center"><strong>AI coding agent that just works. No API key. No subscription. One binary.</strong></p>
</p>

<p align="center">
  <a href="#install">Install</a> ·
  <a href="#quick-start">Quick Start</a> ·
  <a href="#why-dxos">Why DXOS</a> ·
  <a href="#architecture">Architecture</a> ·
  <a href="#roadmap">Roadmap</a> ·
  <a href="#contributing">Contributing</a>
</p>

<p align="center">
  <a href="https://github.com/pdaxt/dxos/actions/workflows/ci.yml"><img src="https://github.com/pdaxt/dxos/actions/workflows/ci.yml/badge.svg" alt="CI"></a>
  <a href="https://github.com/pdaxt/dxos/blob/main/LICENSE"><img src="https://img.shields.io/badge/license-Apache--2.0-blue.svg" alt="License"></a>
  <a href="https://github.com/pdaxt/dxos/releases/latest"><img src="https://img.shields.io/github/v/release/pdaxt/dxos?label=release" alt="Release"></a>
  <a href="https://github.com/pdaxt/dxos/stargazers"><img src="https://img.shields.io/github/stars/pdaxt/dxos?style=flat" alt="Stars"></a>
</p>

---

<p align="center">
  <img src="assets/demo.gif" alt="dxos fix — finds and patches a null pointer bug in 4 seconds" width="800">
</p>

```
$ dxos fix
[scanning 847 files...]
[found: null pointer dereference in src/handler.rs:42]
[patched: added None check with early return]
[verified: cargo test passes]
Done. 1 file changed, 3 insertions.
```

**Zero setup.** Run `dxos` in any project directory. It auto-installs [Ollama](https://ollama.com), detects your hardware, downloads the best model, and starts working. No account. No credit card. No config file.

---

## Install

One command. Takes about 30 seconds.

```bash
curl -fsSL https://raw.githubusercontent.com/pdaxt/dxos/main/install.sh | sh
```

Or build from source:

```bash
git clone https://github.com/pdaxt/dxos && cd dxos
cargo install --path dxos-cli
```

The release binary is ~15MB with full LTO. No runtime dependencies.

---

## Quick Start

Just type `dxos`. That is it. You get an interactive AI coding agent.

```bash
dxos                    # interactive chat (default)
dxos fix                # find and fix issues
dxos review             # review uncommitted changes
dxos explain            # explain the codebase
dxos test               # run tests and fix failures
dxos commit             # generate commit message and commit
dxos pr                 # generate PR description and create PR
```

Every command is one word. No flags to memorize. No YAML to write.

```bash
# Want a specific model?
dxos chat --model qwen2.5-coder:32b

# Want to run a one-shot task?
dxos run "refactor the auth module to use middleware"

# Full-access mode (no permission prompts)
dxos run --permission full-access "run the test suite and fix all failures"
```

**First run?** DXOS walks you through setup:

```bash
dxos setup
# -> Detects your GPU (NVIDIA/Apple Silicon/CPU-only)
# -> Installs Ollama if missing
# -> Downloads the best model for your hardware
# -> Ready in under 2 minutes
```

---

## Why DXOS

| | DXOS | Claude Code | Cursor | OpenCode |
|---|---|---|---|---|
| **Open source** | Yes (Apache-2.0) | No | No | Yes |
| **Price** | Free forever | $200/mo | $20/mo | Free |
| **Works offline** | Yes | No | No | No |
| **Provider lock-in** | None — any model | Anthropic only | OpenAI default | Any model |
| **Dependencies** | Zero. One binary. | Node.js + npm | Electron | Go runtime |
| **Tool overhead** | 8 tools, ~400 tokens | ~206 tools, ~10k tokens | Unknown | ~30 tools |
| **Language** | Rust | TypeScript | TypeScript | Go |
| **Setup time** | 0 seconds | Account + billing | Download + account | Config file |

### The token tax

Every AI coding tool pays a hidden cost: describing its tools to the model on every single request. More tools means more tokens burned before the model even reads your code.

```
DXOS:       8 tools  x  ~50 tokens each  =    ~400 tokens
Claude Code: 206 tools x  ~50 tokens each  = ~10,000 tokens
```

At $3 per million input tokens, that is $0.03 wasted per request with Claude Code versus $0.001 with DXOS. Over a thousand requests, DXOS saves $29 in pure overhead.

But the real cost is not money -- it is context window. Those 10,000 tokens of tool definitions are 10,000 tokens that could have been your code.

### Why 8 tools is enough

These 8 tools cover 95% of everything a coding agent needs to do:

| Tool | Purpose | Implementation |
|---|---|---|
| `bash` | Run any shell command | `tokio::process` with configurable timeout |
| `read_file` | Read files with line numbers | `std::fs` with offset and limit |
| `write_file` | Create or overwrite files | Atomic write with parent directory creation |
| `edit_file` | Surgical find-and-replace | String matching with uniqueness validation |
| `glob` | Find files by pattern | `ignore` crate (same engine as ripgrep) |
| `grep` | Search file contents | `regex` + `ignore` (respects `.gitignore`) |
| `git` | All git operations | Direct shell passthrough |
| `web_fetch` | Fetch URL content | `reqwest` with automatic HTML-to-text |

All tools execute as native Rust function calls inside the same process. No subprocess spawning for tool dispatch. No JSON-RPC. No MCP protocol negotiation. Just function calls.

---

## Architecture

DXOS is a Cargo workspace. Each crate is independent and reusable.

```
dxos/
+-  dxos-cli/              # The `dxos` binary — CLI entry point
+-  crates/
|   +-  core/              # Shared types, config, error handling
|   +-  tools/             # 8 native Rust tool implementations
|   +-  harness/           # Conversation runtime, permissions, compaction
|   +-  api/               # Multi-provider LLM client
|   +-  fleet/             # Multi-agent orchestration       [planned]
|   +-  brain/             # Persistent cross-session memory  [planned]
|   +-  dashboard/         # Real-time TUI monitoring         [planned]
+-  Cargo.toml             # Workspace root
```

### Supported providers

DXOS connects to any OpenAI-compatible API out of the box:

| Provider | Models | Setup |
|---|---|---|
| **Ollama** (local) | Qwen 2.5 Coder, DeepSeek, Llama, Codestral, etc. | Auto-detected, no key needed |
| **Anthropic** | Claude 4, Opus, Sonnet, Haiku | `ANTHROPIC_API_KEY` |
| **OpenAI** | GPT-4o, o1, o3 | `OPENAI_API_KEY` |
| **OpenRouter** | 100+ models from every provider | `OPENROUTER_API_KEY` |

### Conversation runtime

The harness manages the full agent loop with three layers of context compression:

```
User prompt
    |
    v
System prompt + tool definitions (400 tokens, not 10,000)
    |
    v
LLM generates response with tool calls
    |
    v
Permission gate (read-only / workspace-write / full-access)
    |
    v
Native tool execution (zero subprocess overhead)
    |
    v
Results fed back to LLM
    |
    v
Loop until complete or turn limit reached
    |
    v
Context compression if window grows large
    MicroCompact  ->  AutoCompact  ->  Emergency
```

### What makes it fast

- **Single binary, single process.** No Node.js. No Python. No Docker. Just a statically-linked Rust executable.
- **Native tool calls.** Tools are Rust functions, not subprocesses. A `read_file` call is a function call, not a fork+exec.
- **SSE streaming.** Tokens appear as they are generated. An animated spinner with cycling verbs shows activity while the model thinks.
- **Smart model detection.** On first run, DXOS probes Ollama, then checks for API keys, and selects the best available model automatically.
- **REPL with history.** Arrow keys, persistent history across sessions, readline bindings.
- **Project instructions.** Reads `CLAUDE.md` or `DXOS.md` from your project root and feeds it as context automatically.

---

## Roadmap

### v0.1 -- Solo Agent (current)

- [x] 8 native Rust tool implementations
- [x] Conversation runtime with agentic turn loop
- [x] Permission gating: read-only, workspace-write, full-access
- [x] 3-layer context compression
- [x] SSE streaming with animated spinner
- [x] Interactive REPL with readline history
- [x] Multi-provider support: Ollama, Anthropic, OpenAI, OpenRouter
- [x] Smart model auto-detection and hardware-aware setup
- [x] One-word commands: fix, review, explain, test, commit, pr
- [x] Project instruction loading (CLAUDE.md / DXOS.md)
- [x] Text-based tool extraction (works with any model, not just function-calling models)
- [x] 39 tests, zero warnings, zero `unsafe`

### v0.2 -- Fleet + Memory

- [ ] Multi-agent fleet on isolated git worktrees
- [ ] Persistent SQLite-backed memory across sessions
- [ ] Real-time TUI dashboard (Ratatui)
- [ ] Session logging and cost tracking
- [ ] Extended thinking mode display

### v0.3 -- Ecosystem

- [ ] Plugin system for custom tools
- [ ] Web dashboard for fleet monitoring
- [ ] IDE extensions (VS Code, JetBrains)
- [ ] Agent governance and audit trails
- [ ] `dxos deploy` -- ship code end-to-end

---

## Contributing

The codebase is designed to be readable and hackable. Here is where to start:

**Add a new LLM provider:** Implement the `ApiClient` trait in `crates/api/`. Any OpenAI-compatible endpoint works with minimal code.

**Add a new tool:** Write a function in `crates/tools/` and register it in `registry.rs`. The model sees it on the next run.

**Improve the agent loop:** The conversation runtime lives in `crates/harness/src/runtime.rs`.

```bash
cargo test                 # run all 39 tests
cargo clippy               # lint (zero warnings policy)
cargo build --release      # build release binary (~15MB)
```

See [CONTRIBUTING.md](CONTRIBUTING.md) for detailed guidelines.

---

## Acknowledgments

DXOS is original work, built from scratch in Rust. The design is informed by publicly documented agent patterns, the open-source AI tooling community, and the belief that developer tools should be free, fast, and transparent.

## License

[Apache-2.0](LICENSE)

---

<p align="center">
  <strong>Star the repo if you believe AI coding tools should be open.</strong>
</p>
