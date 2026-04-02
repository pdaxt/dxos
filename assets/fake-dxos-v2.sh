#!/bin/bash
# Fake dxos output for demo recording — v2
# Shows: version → explain → fix flow

if [[ "$1" == "--version" ]]; then
    echo "dxos 0.3.0"
    exit 0
fi

if [[ "$1" == "--help" ]]; then
    echo "DXOS — One binary. Works offline. Free forever."
    echo ""
    echo "Usage: dxos [COMMAND]"
    echo ""
    echo "Commands:"
    echo "  chat     Interactive chat with the agent"
    echo "  run      Run a single prompt"
    echo "  fix      Find and fix issues automatically"
    echo "  review   Review uncommitted changes"
    echo "  explain  Explain the current codebase"
    echo "  test     Run tests and fix failures"
    echo "  commit   Generate commit message and commit"
    echo "  pr       Generate PR description and create PR"
    echo "  setup    Download and configure a model"
    echo ""
    echo "No API key required. Works offline with local models."
    exit 0
fi

if [[ "$1" == "fix" ]]; then
    echo -e "\033[2mdxos v0.3.0 | qwen2.5-coder:32b | workspace-write\033[0m"
    echo ""
    sleep 0.3

    # Spinner simulation
    for i in 1 2 3 4; do
        printf "\r\033[36m⠋\033[0m \033[2mScanning...\033[0m   "
        sleep 0.15
        printf "\r\033[36m⠙\033[0m \033[2mScanning.  \033[0m   "
        sleep 0.15
        printf "\r\033[36m⠹\033[0m \033[2mScanning.. \033[0m   "
        sleep 0.15
    done

    # Tool calls
    printf "\r\033[K\033[36m◆\033[0m \033[1mglob\033[0m \033[2m**/*.rs\033[0m\n"
    sleep 0.3
    printf "\033[36m◆\033[0m \033[1mread_file\033[0m \033[2msrc/handler.rs\033[0m\n"
    sleep 0.3
    printf "\033[36m◆\033[0m \033[1mgrep\033[0m \033[2m/unwrap\\(\\)/\033[0m\n"
    sleep 0.4

    echo ""
    echo -e "  Found 2 issues:"
    echo ""
    sleep 0.2

    echo -e "  \033[31m1.\033[0m \033[1msrc/handler.rs:47\033[0m — unwrap on Option that can be None"
    echo -e "     \033[2mlet user = get_user(token).unwrap();\033[0m"
    sleep 0.3

    echo -e "  \033[31m2.\033[0m \033[1msrc/db.rs:23\033[0m — SQL injection via string interpolation"
    echo -e "     \033[2mlet query = format!(\"SELECT * WHERE id = {}\", input);\033[0m"
    sleep 0.4

    echo ""
    printf "\033[33m◆\033[0m \033[1medit_file\033[0m \033[2msrc/handler.rs\033[0m\n"
    sleep 0.3
    printf "\033[33m◆\033[0m \033[1medit_file\033[0m \033[2msrc/db.rs\033[0m\n"
    sleep 0.3

    echo ""
    printf "\033[35m◆\033[0m \033[1mbash\033[0m \033[2m\$ cargo test\033[0m\n"
    sleep 0.5

    echo -e "  \033[32m✓\033[0m All 47 tests passing"
    echo ""
    echo -e "  Fixed 2 issues. Run \033[2mgit diff\033[0m to review."
    echo ""
    echo -e "\033[2m─── 6 tool calls │ 3 iterations │ 4,218 tokens │ 8.2s ───\033[0m"
    exit 0
fi

if [[ "$1" == "explain" ]]; then
    echo -e "\033[2mdxos v0.3.0 | qwen2.5-coder:32b | read-only\033[0m"
    echo ""

    for i in 1 2 3; do
        printf "\r\033[36m⠋\033[0m \033[2mReading...  \033[0m   "
        sleep 0.15
        printf "\r\033[36m⠙\033[0m \033[2mReading.   \033[0m   "
        sleep 0.15
        printf "\r\033[36m⠹\033[0m \033[2mReading..  \033[0m   "
        sleep 0.15
    done

    printf "\r\033[K\033[36m◆\033[0m \033[1mglob\033[0m \033[2m**/Cargo.toml\033[0m\n"
    sleep 0.2
    printf "\033[36m◆\033[0m \033[1mread_file\033[0m \033[2mREADME.md\033[0m\n"
    sleep 0.2
    printf "\033[36m◆\033[0m \033[1mrepo_map\033[0m \033[2m9 files, 47 definitions\033[0m\n"
    sleep 0.3

    echo ""
    echo "  This is a Rust workspace with 9 crates implementing an AI"
    echo "  coding agent. The CLI (dxos-cli) provides one-word commands"
    echo "  like fix, review, and test. The harness crate manages the"
    echo "  conversation loop with 3-layer context compression. Tools"
    echo "  are native Rust (no subprocesses). Supports Ollama, Anthropic,"
    echo "  OpenAI, and OpenRouter."
    echo ""
    echo -e "\033[2m─── 3 tool calls │ 2 iterations │ 2,891 tokens │ 4.1s ───\033[0m"
    exit 0
fi
