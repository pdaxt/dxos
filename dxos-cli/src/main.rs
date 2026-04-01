use anyhow::Result;
use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(
    name = "dxos",
    about = "The open-source AI agent operating system",
    version,
    long_about = "DXOS — One Rust binary. Any model. From solo dev to agent fleet.\n\nhttps://github.com/pdaxt/dxos"
)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Run a single agent session with a prompt
    Run {
        /// The task or question for the agent
        prompt: Vec<String>,

        /// Model to use (default: claude-sonnet-4-20250514)
        #[arg(short, long)]
        model: Option<String>,

        /// Permission mode: read-only, workspace-write, full-access
        #[arg(short, long, default_value = "workspace-write")]
        permission: String,

        /// Maximum turn iterations
        #[arg(long, default_value = "16")]
        max_turns: usize,
    },

    /// Spawn a fleet of agents on isolated worktrees
    Fleet {
        /// The mission for the fleet
        prompt: Vec<String>,

        /// Number of agents to spawn
        #[arg(short = 'n', long, default_value = "4")]
        agents: usize,
    },

    /// Query persistent memory across sessions
    Brain {
        /// What to recall
        query: Vec<String>,
    },

    /// Open the real-time TUI dashboard
    Dash,

    /// Show session log — what agents did and what it cost
    Log {
        /// Number of recent sessions to show
        #[arg(short, long, default_value = "10")]
        limit: usize,
    },

    /// Initialize .dxos/ in the current project
    Init,

    /// Show current configuration
    Config,
}

fn main() -> Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::from_default_env()
                .add_directive("dxos=info".parse()?),
        )
        .with_target(false)
        .init();

    let cli = Cli::parse();

    match cli.command {
        Commands::Run {
            prompt,
            model,
            permission,
            max_turns,
        } => cmd_run(prompt.join(" "), model, permission, max_turns),

        Commands::Fleet { prompt, agents } => {
            eprintln!("Fleet mode coming in v0.2 — {agents} agents on isolated worktrees");
            eprintln!("Mission: {}", prompt.join(" "));
            Ok(())
        }

        Commands::Brain { query } => {
            eprintln!("Brain coming in v0.2 — persistent cross-session memory");
            eprintln!("Query: {}", query.join(" "));
            Ok(())
        }

        Commands::Dash => {
            eprintln!("Dashboard coming in v0.2 — real-time TUI with Ratatui");
            Ok(())
        }

        Commands::Log { limit } => {
            eprintln!("Session log coming in v0.2 — last {limit} sessions");
            Ok(())
        }

        Commands::Init => cmd_init(),
        Commands::Config => cmd_config(),
    }
}

fn cmd_run(prompt: String, model: Option<String>, permission: String, max_turns: usize) -> Result<()> {
    if prompt.is_empty() {
        anyhow::bail!("No prompt provided. Usage: dxos run \"fix the auth bug\"");
    }

    let cwd = std::env::current_dir()?;

    // Load config
    let mut config = dxos_core::DxosConfig::load();
    if let Some(m) = model {
        config.provider.model = m;
    }

    // Build permission policy
    let mode = match permission.as_str() {
        "read-only" => dxos_harness::PermissionMode::ReadOnly,
        "full-access" => dxos_harness::PermissionMode::FullAccess,
        _ => dxos_harness::PermissionMode::WorkspaceWrite,
    };

    let policy = dxos_harness::PermissionPolicy::new(mode)
        .with_tool("read_file", dxos_harness::PermissionMode::ReadOnly)
        .with_tool("glob", dxos_harness::PermissionMode::ReadOnly)
        .with_tool("grep", dxos_harness::PermissionMode::ReadOnly)
        .with_tool("write_file", dxos_harness::PermissionMode::WorkspaceWrite)
        .with_tool("edit_file", dxos_harness::PermissionMode::WorkspaceWrite)
        .with_tool("git", dxos_harness::PermissionMode::WorkspaceWrite)
        .with_tool("bash", dxos_harness::PermissionMode::FullAccess);

    // Build tool definitions
    let registry = dxos_tools::ToolRegistry::default_cli();
    let tools = registry.to_api_definitions();

    // Build system prompt
    let system_prompt = build_system_prompt(&cwd);

    // Create API client
    let client = dxos_api::ProviderClient::from_config(&config.provider)?;

    // Create runtime
    let mut runtime = dxos_harness::ConversationRuntime::new(
        client,
        policy,
        system_prompt,
        tools,
        cwd,
    )
    .with_max_iterations(max_turns);

    eprintln!("dxos v{} — model: {} — mode: {permission}", env!("CARGO_PKG_VERSION"), config.provider.model);
    eprintln!();

    // Run the turn
    let summary = runtime.run_turn(&prompt, None)?;

    // Print output
    println!("{}", summary.text);

    eprintln!();
    eprintln!(
        "--- {} tool calls | {} iterations | {} tokens ---",
        summary.tool_calls,
        summary.iterations,
        summary.usage.total_tokens()
    );

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
        dxos_dir.join("config.toml"),
        r#"# DXOS project configuration
[provider]
provider = "anthropic"
model = "claude-sonnet-4-20250514"

[settings]
max_turns = 16
permission_mode = "workspace-write"
"#,
    )?;

    eprintln!("Initialized .dxos/ in {}", cwd.display());
    Ok(())
}

fn cmd_config() -> Result<()> {
    let config = dxos_core::DxosConfig::load();
    println!("{}", serde_json::to_string_pretty(&config)?);
    Ok(())
}

fn build_system_prompt(cwd: &std::path::Path) -> Vec<String> {
    let mut parts = vec![
        "You are DXOS, an AI coding agent. You help developers by reading, writing, and editing code.".to_string(),
        format!("Working directory: {}", cwd.display()),
    ];

    // Load CLAUDE.md / DXOS.md if present
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
