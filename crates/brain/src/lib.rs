// Brain — persistent cross-session memory.
// Coming in v0.2: SQLite-backed knowledge graph that learns your codebase.

pub struct Brain {
    // Will use rusqlite for persistence
}

impl Brain {
    pub fn new() -> Self {
        Self {}
    }
}

impl Default for Brain {
    fn default() -> Self {
        Self::new()
    }
}
