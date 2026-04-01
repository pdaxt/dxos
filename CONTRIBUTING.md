# Contributing to DXOS

Thanks for considering contributing. DXOS is designed to be easy to hack on.

## Setup

```bash
git clone https://github.com/pdaxt/dxos
cd dxos
cargo build
cargo test
```

That's it. No npm, no Docker, no config files. Just Rust.

## Where to Start

| Want to... | Look at... |
|---|---|
| Add a new tool | `crates/tools/src/` — add a file, register in `lib.rs` and `registry.rs` |
| Add a new LLM provider | `crates/api/src/` — implement the `ApiClient` trait |
| Improve the conversation loop | `crates/harness/src/runtime.rs` |
| Add permission logic | `crates/harness/src/permissions.rs` |
| Work on fleet mode | `crates/fleet/src/` |
| Work on persistent memory | `crates/brain/src/` |
| Work on the TUI dashboard | `crates/dashboard/src/` |
| Fix CLI UX | `dxos-cli/src/main.rs` |

## Adding a Tool (5 minutes)

1. Create `crates/tools/src/your_tool.rs`
2. Define `YourInput` and `YourOutput` structs with serde
3. Write a function `pub fn your_tool(input: YourInput, cwd: &Path) -> Result<YourOutput>`
4. Add it to `crates/tools/src/lib.rs` (mod + pub use + match arm in `execute_tool`)
5. Add a `ToolSpec` entry in `crates/tools/src/registry.rs`

## Adding an LLM Provider (10 minutes)

1. Create `crates/api/src/your_provider.rs`
2. Implement `ApiClient` for your client struct
3. Add a variant to `ProviderClient` in `crates/api/src/provider.rs`
4. Add a `ModelProvider` variant in `crates/core/src/config.rs`

## Guidelines

- **No unsafe code.** It's forbidden in `Cargo.toml`.
- **Keep it lean.** Every dependency is a cost. Justify new ones.
- **7 tools is a feature, not a limitation.** Don't add tools unless they cover a genuinely new capability.
- **Tests are welcome** but not blocking for a PR.
- **Clippy clean.** Run `cargo clippy` before submitting.

## PR Process

1. Fork and branch
2. Make your changes
3. `cargo test && cargo clippy`
4. Open a PR with a clear description of what and why

We review PRs quickly. Small, focused PRs get merged fastest.
