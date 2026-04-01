#!/bin/bash
# Fake dxos binary for demo recording
# Routes to real binary or mock output based on args

if [[ "$1" == "--version" ]]; then
    echo "dxos 0.1.0"
    exit 0
fi

if [[ "$1" == "--help" ]]; then
    echo "The open-source AI agent operating system"
    echo ""
    echo "Usage: dxos <COMMAND>"
    echo ""
    echo "Commands:"
    echo "  run     Run a single agent session with a prompt"
    echo "  fleet   Spawn a fleet of agents on isolated worktrees"
    echo "  brain   Query persistent memory across sessions"
    echo "  dash    Open the real-time TUI dashboard"
    echo "  log     Show session log — what agents did and what it cost"
    echo "  init    Initialize .dxos/ in the current project"
    echo "  config  Show current configuration"
    echo ""
    echo "Options:"
    echo "  -h, --help     Print help"
    echo "  -V, --version  Print version"
    exit 0
fi

if [[ "$1" == "run" ]]; then
    shift
    echo -e "\033[2mdxos v0.1.0 — model: claude-sonnet-4 — mode: workspace-write\033[0m"
    echo ""
    sleep 0.5

    echo -e "\033[36m◆\033[0m Reading src/api/handler.rs..."
    sleep 0.6

    echo -e "\033[36m◆\033[0m Found the issue at \033[1mhandler.rs:47\033[0m"
    sleep 0.4

    echo ""
    echo "The \`user\` variable is unwrapped without a None check."
    echo "When the session token expires, \`get_user()\` returns None"
    echo "and the handler panics instead of returning 401."
    echo ""
    sleep 0.5

    echo -e "\033[33m◆\033[0m Editing src/api/handler.rs..."
    sleep 0.5

    echo -e "\033[2m   45 │\033[0m let token = req.headers().get(\"Authorization\");"
    echo -e "\033[2m   46 │\033[0m let user = get_user(token);"
    echo -e "\033[31m   47 │- let name = user.unwrap().name;\033[0m"
    echo -e "\033[32m   47 │+ let Some(user) = get_user(token) else {\033[0m"
    echo -e "\033[32m   48 │+     return Err(ApiError::Unauthorized);\033[0m"
    echo -e "\033[32m   49 │+ };\033[0m"
    echo -e "\033[32m   50 │+ let name = user.name;\033[0m"
    echo ""
    sleep 0.5

    echo -e "\033[36m◆\033[0m Running cargo test..."
    sleep 0.8

    echo -e "\033[32m   ✓\033[0m test_handler_valid_token ............ ok"
    echo -e "\033[32m   ✓\033[0m test_handler_expired_token .......... ok"
    echo -e "\033[32m   ✓\033[0m test_handler_missing_token .......... ok"
    echo -e "\033[32m   ✓\033[0m 20 more tests ....................... ok"
    echo ""
    sleep 0.3

    echo "Fixed. The handler now returns 401 instead of panicking"
    echo "on expired sessions. All 23 tests pass."
    echo ""
    echo -e "\033[2m--- 6 tool calls | 3 iterations | 4,218 tokens (\$0.02) ---\033[0m"
    exit 0
fi

if [[ "$1" == "init" ]]; then
    echo "Initialized .dxos/ in $(pwd)"
    exit 0
fi

if [[ "$1" == "config" ]]; then
    echo '{'
    echo '  "provider": {'
    echo '    "provider": "anthropic",'
    echo '    "model": "claude-sonnet-4-20250514"'
    echo '  },'
    echo '  "max_turns": 16,'
    echo '  "permission_mode": "workspace-write"'
    echo '}'
    exit 0
fi
