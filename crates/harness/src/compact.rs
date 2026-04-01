use dxos_core::Session;

#[derive(Debug, Clone)]
pub struct CompactionConfig {
    pub max_messages: usize,
    pub keep_recent: usize,
}

impl Default for CompactionConfig {
    fn default() -> Self {
        Self {
            max_messages: 100,
            keep_recent: 20,
        }
    }
}

pub fn should_compact(session: &Session, config: &CompactionConfig) -> bool {
    session.messages.len() > config.max_messages
}

pub fn compact_session(session: &mut Session, config: &CompactionConfig) {
    if session.messages.len() <= config.keep_recent {
        return;
    }

    let keep_from = session.messages.len() - config.keep_recent;
    session.messages = session.messages.split_off(keep_from);
}
