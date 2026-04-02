use anyhow::Result;
use clap::{Parser, Subcommand};

mod display;
mod models;
mod prompt;
mod repl;
mod setup;

#[derive(Parser)]
#[command(
    name = "dxos",
    about = "AI coding agent. No API key. No setup. Just works.",
    version,
    long_about = "DXOS — One binary. Works offline. Free forever.\n\nhttps://github.com/pdaxt/dxos"
)]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand)]
enum Commands {
    /// Interactive chat with the agent
    Chat {
        #[arg(short, long)]
        model: Option<String>,
    },

    /// Run a single prompt
    Run {
        prompt: Vec<String>,
        #[arg(short, long)]
        model: Option<String>,
        #[arg(short, long, default_value = "workspace-write")]
        permission: String,
        #[arg(long, default_value = "16")]
        max_turns: usize,
    },

    /// Find and fix issues automatically
    Fix,

    /// Review uncommitted changes
    Review,

    /// Explain the current codebase
    Explain,

    /// Run tests and fix failures
    Test,

    /// Generate commit message and commit
    Commit,

    /// Generate PR description and create PR
    Pr,

    /// Download and configure a model
    Setup,

    /// Initialize .dxos/ in project
    Init,

    /// Show config
    Config,
}

fn main() -> Result<()> {
    // Minimal logging — don't spam the user
    tracing_subscriber::fmt()
        .with_env_filter("dxos=warn")
        .with_target(false)
        .without_time()
        .init();

    let cli = Cli::parse();

    match cli.command {
        // No subcommand = default to chat
        None => repl::run_repl(None),

        Some(Commands::Chat { model }) => repl::run_repl(model),

        Some(Commands::Run { prompt, model, permission, max_turns }) => {
            cmd_run(prompt.join(" "), model, permission, max_turns)
        }

        Some(Commands::Fix) => {
            cmd_run(
                "Find all bugs, issues, and code smells in this codebase. Fix them. Run tests to verify.".into(),
                None, "workspace-write".into(), 16,
            )
        }

        Some(Commands::Review) => {
            cmd_run(
                "Run `git diff` to see uncommitted changes. Review each change for bugs, security issues, and code quality. Be specific and actionable.".into(),
                None, "read-only".into(), 8,
            )
        }

        Some(Commands::Explain) => {
            cmd_run(
                "Read the key files in this project (start with README, Cargo.toml or package.json, then main entry point). Give a concise explanation of what this project does, its architecture, and key design decisions.".into(),
                None, "read-only".into(), 8,
            )
        }

        Some(Commands::Test) => {
            cmd_run(
                "Run the test suite (detect the test command from package.json, Cargo.toml, or Makefile). If any tests fail, read the failing test and the code it tests, then fix the issue. Re-run tests to verify.".into(),
                None, "full-access".into(), 16,
            )
        }

        Some(Commands::Commit) => {
            cmd_run(
                "Run `git diff --staged` to see staged changes. If nothing is staged, run `git diff` for unstaged changes. Generate a concise, conventional commit message (e.g. 'fix: ...', 'feat: ...', 'refactor: ...'). Then run `git add -A && git commit -m \"<message>\"`.".into(),
                None, "full-access".into(), 4,
            )
        }

        Some(Commands::Pr) => {
            cmd_run(
                "Run `git log --oneline main..HEAD` to see commits on this branch. Generate a PR title and description with a summary and test plan. Then run `gh pr create --title \"<title>\" --body \"<description>\"`.".into(),
                None, "full-access".into(), 4,
            )
        }

        Some(Commands::Setup) => {
            setup::auto_setup()?;
            Ok(())
        }

        Some(Commands::Init) => cmd_init(),
        Some(Commands::Config) => cmd_config(),
    }
}

fn cmd_run(prompt_text: String, model: Option<String>, permission: String, max_turns: usize) -> Result<()> {
    if prompt_text.is_empty() {
        anyhow::bail!("No prompt. Usage: dxos run \"fix the bug\"");
    }

    let cwd = std::env::current_dir()?;

    // Auto-detect or auto-setup model
    let (client, model_name) = match dxos_api::ProviderClient::auto_detect(model.as_deref()) {
        Ok(result) => result,
        Err(_) => {
            let model_id = setup::ensure_ready()?;
            dxos_api::ProviderClient::auto_detect(Some(&model_id))?
        }
    };

    let mode = match permission.as_str() {
        "read-only" => dxos_harness::PermissionMode::ReadOnly,
        "full-access" => dxos_harness::PermissionMode::FullAccess,
        _ => dxos_harness::PermissionMode::WorkspaceWrite,
    };

    let policy = dxos_harness::PermissionPolicy::new(mode)
        .with_tool("read_file", dxos_harness::PermissionMode::ReadOnly)
        .with_tool("glob", dxos_harness::PermissionMode::ReadOnly)
        .with_tool("grep", dxos_harness::PermissionMode::ReadOnly)
        .with_tool("web_fetch", dxos_harness::PermissionMode::ReadOnly)
        .with_tool("write_file", dxos_harness::PermissionMode::WorkspaceWrite)
        .with_tool("edit_file", dxos_harness::PermissionMode::WorkspaceWrite)
        .with_tool("git", dxos_harness::PermissionMode::WorkspaceWrite)
        .with_tool("bash", dxos_harness::PermissionMode::FullAccess);

    let registry = dxos_tools::ToolRegistry::default_cli();
    let tools = registry.to_api_definitions();
    let system_prompt = prompt::build_system_prompt(&cwd);

    let mut runtime = dxos_harness::ConversationRuntime::new(
        client, policy, system_prompt, tools, cwd,
    ).with_max_iterations(max_turns);

    eprintln!("\x1b[2mdxos v{} | {model_name} | {permission}\x1b[0m", env!("CARGO_PKG_VERSION"));
    eprintln!();

    let start = std::time::Instant::now();
    let summary = runtime.run_turn(&prompt_text, None)?;

    // Only print text if it wasn't already streamed
    if !summary.was_streamed && !summary.text.is_empty() {
        println!("{}", summary.text);
    }

    let elapsed = start.elapsed().as_secs_f64();
    display::print_summary(summary.tool_calls, summary.iterations, summary.usage.total_tokens(), elapsed);

    Ok(())
}

fn cmd_init() -> Result<()> {
    let cwd = std::env::current_dir()?;
    let dxos_dir = cwd.join(".dxos");

    if dxos_dir.exists() {
        eprintln!(".dxos/ already exists");
        return Ok(());
    }

    std::fs::create_dir_all(&dxos_dir)?;
    std::fs::write(
        dxos_dir.join("instructions.md"),
        "# Project Instructions\n\n<!-- Add project-specific instructions for the AI agent here -->\n",
    )?;

    eprintln!("Initialized .dxos/ in {}", cwd.display());
    Ok(())
}

fn cmd_config() -> Result<()> {
    let config = dxos_core::DxosConfig::load();
    println!("{}", serde_json::to_string_pretty(&config)?);
    Ok(())
}
