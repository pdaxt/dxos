// Fleet orchestration — multi-agent coordination on isolated worktrees.
// Coming in v0.2: spawn N agents, auto-PR, real-time TUI monitoring.

pub struct FleetConfig {
    pub agents: usize,
    pub strategy: FleetStrategy,
}

pub enum FleetStrategy {
    /// Each agent takes one issue from the backlog
    Swarm,
    /// Agents work on different parts of the same feature
    Parallel,
    /// Pipeline: agent 1 → agent 2 → agent 3
    Pipeline,
}
