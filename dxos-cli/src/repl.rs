use std::io::{self, BufRead, Write};

use anyhow::Result;

use crate::models;

pub fn run_repl(model: Option<String>) -> Result<()> {
    let cwd = std::env::current_dir()?;

    // Auto-detect or setup model
    let (client, model_name) = match dxos_api::ProviderClient::auto_detect(model.as_deref()) {
        Ok(result) => result,
        Err(_) => {
            eprintln!("No model available. Let's set one up.\n");
            let model_id = models::interactive_setup()?;
            dxos_api::ProviderClient::auto_detect(Some(&model_id))?
        }
    };

    // Build permission policy
    let policy = dxos_harness::PermissionPolicy::new(dxos_harness::PermissionMode::WorkspaceWrite)
        .with_tool("read_file", dxos_harness::PermissionMode::ReadOnly)
        .with_tool("glob", dxos_harness::PermissionMode::ReadOnly)
        .with_tool("grep", dxos_harness::PermissionMode::ReadOnly)
        .with_tool("write_file", dxos_harness::PermissionMode::WorkspaceWrite)
        .with_tool("edit_file", dxos_harness::PermissionMode::WorkspaceWrite)
        .with_tool("git", dxos_harness::PermissionMode::WorkspaceWrite)
        .with_tool("bash", dxos_harness::PermissionMode::FullAccess);

    let registry = dxos_tools::ToolRegistry::default_cli();
    let tools = registry.to_api_definitions();
    let system_prompt = build_system_prompt(&cwd);

    let mut runtime = dxos_harness::ConversationRuntime::new(
        client,
        policy,
        system_prompt,
        tools,
        cwd.clone(),
    )
    .with_max_iterations(16);

    // Banner
    eprintln!();
    eprintln!("  \x1b[1;36mdxos\x1b[0m v{} — interactive mode", env!("CARGO_PKG_VERSION"));
    eprintln!("  \x1b[2mmodel: {model_name} | dir: {}\x1b[0m", cwd.display());
    eprintln!("  \x1b[2mtype /help for commands, /quit to exit\x1b[0m");
    eprintln!();

    let stdin = io::stdin();
    let mut lines = stdin.lock().lines();
    let mut turn_count: usize = 0;

    loop {
        // Prompt
        eprint!("\x1b[1;32m❯\x1b[0m ");
        io::stderr().flush()?;

        let line = match lines.next() {
            Some(Ok(line)) => line,
            Some(Err(e)) => {
                eprintln!("Input error: {e}");
                break;
            }
            None => break, // EOF
        };

        let input = line.trim().to_string();

        if input.is_empty() {
            continue;
        }

        // Handle slash commands
        if input.starts_with('/') {
            match input.as_str() {
                "/quit" | "/exit" | "/q" => {
                    eprintln!("\x1b[2mbye.\x1b[0m");
                    break;
                }
                "/help" | "/h" => {
                    eprintln!();
                    eprintln!("  \x1b[1mCommands:\x1b[0m");
                    eprintln!("    /help       — show this help");
                    eprintln!("    /quit       — exit the session");
                    eprintln!("    /clear      — reset conversation history");
                    eprintln!("    /model      — show current model");
                    eprintln!("    /turns      — show turn count and token usage");
                    eprintln!("    /compact    — compress conversation history");
                    eprintln!();
                    continue;
                }
                "/clear" => {
                    runtime = dxos_harness::ConversationRuntime::new(
                        // Can't reuse client after move — create new one
                        dxos_api::ProviderClient::auto_detect(Some(&model_name))?.0,
                        dxos_harness::PermissionPolicy::new(dxos_harness::PermissionMode::WorkspaceWrite)
                            .with_tool("read_file", dxos_harness::PermissionMode::ReadOnly)
                            .with_tool("glob", dxos_harness::PermissionMode::ReadOnly)
                            .with_tool("grep", dxos_harness::PermissionMode::ReadOnly)
                            .with_tool("write_file", dxos_harness::PermissionMode::WorkspaceWrite)
                            .with_tool("edit_file", dxos_harness::PermissionMode::WorkspaceWrite)
                            .with_tool("git", dxos_harness::PermissionMode::WorkspaceWrite)
                            .with_tool("bash", dxos_harness::PermissionMode::FullAccess),
                        build_system_prompt(&cwd),
                        registry.to_api_definitions(),
                        cwd.clone(),
                    )
                    .with_max_iterations(16);
                    turn_count = 0;
                    eprintln!("\x1b[2mconversation cleared.\x1b[0m");
                    continue;
                }
                "/model" => {
                    eprintln!("  model: {model_name}");
                    continue;
                }
                "/turns" => {
                    eprintln!("  turns: {turn_count}");
                    continue;
                }
                "/compact" => {
                    eprintln!("\x1b[2mcompacting conversation...\x1b[0m");
                    // Compaction happens automatically in the runtime
                    continue;
                }
                _ => {
                    eprintln!("\x1b[33munknown command: {input}\x1b[0m (try /help)");
                    continue;
                }
            }
        }

        // Run the turn
        turn_count += 1;
        match runtime.run_turn(&input, None) {
            Ok(summary) => {
                eprintln!();
                println!("{}", summary.text);
                eprintln!();
                eprintln!(
                    "\x1b[2m--- {} tool calls | {} iterations | {} tokens ---\x1b[0m",
                    summary.tool_calls,
                    summary.iterations,
                    summary.usage.total_tokens()
                );
                eprintln!();
            }
            Err(e) => {
                eprintln!("\x1b[31merror: {e}\x1b[0m");
                eprintln!();
            }
        }
    }

    Ok(())
}

fn build_system_prompt(cwd: &std::path::Path) -> Vec<String> {
    let mut parts = vec![
        "You are DXOS, an AI coding agent. You help developers by reading, writing, and editing code. Keep responses concise.".to_string(),
        format!("Working directory: {}", cwd.display()),
    ];

    for name in &["CLAUDE.md", "DXOS.md", ".dxos/instructions.md"] {
        let path = cwd.join(name);
        if path.exists() {
            if let Ok(content) = std::fs::read_to_string(&path) {
                parts.push(format!("Project instructions from {name}:\n{content}"));
            }
        }
    }

    parts
}
