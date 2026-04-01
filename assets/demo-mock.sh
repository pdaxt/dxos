#!/bin/bash
# Mock dxos output for demo recording
# Simulates a real agent session with tool calls

echo -e "\033[2mdxos v0.1.0 — model: claude-sonnet-4 — mode: workspace-write\033[0m"
echo ""

sleep 0.3

# Simulate reading
echo -e "\033[36m◆\033[0m Reading src/api/handler.rs..."
sleep 0.4

echo -e "\033[36m◆\033[0m Found the issue at \033[1mhandler.rs:47\033[0m"
sleep 0.3

echo ""
echo "The \`user\` variable is unwrapped without a None check."
echo "When the session token expires, \`get_user()\` returns None"
echo "and the handler panics instead of returning 401."
echo ""
sleep 0.3

# Simulate editing
echo -e "\033[33m◆\033[0m Editing src/api/handler.rs..."
sleep 0.4

echo -e "\033[2m   45 │     let token = req.headers().get(\"Authorization\");\033[0m"
echo -e "\033[2m   46 │     let user = get_user(token);\033[0m"
echo -e "\033[31m   47 │-    let name = user.unwrap().name;\033[0m"
echo -e "\033[32m   47 │+    let Some(user) = get_user(token) else {\033[0m"
echo -e "\033[32m   48 │+        return Err(ApiError::Unauthorized);\033[0m"
echo -e "\033[32m   49 │+    };\033[0m"
echo -e "\033[32m   50 │+    let name = user.name;\033[0m"
echo ""
sleep 0.3

# Simulate test run
echo -e "\033[36m◆\033[0m Running cargo test..."
sleep 0.6

echo -e "\033[32m   ✓\033[0m test_handler_valid_token ............ ok"
echo -e "\033[32m   ✓\033[0m test_handler_expired_token .......... ok"
echo -e "\033[32m   ✓\033[0m test_handler_missing_token .......... ok"
echo -e "\033[32m   ✓\033[0m 20 more tests ....................... ok"
echo ""
sleep 0.2

echo "Fixed. The handler now returns 401 Unauthorized instead of"
echo "panicking on expired sessions. All 23 tests pass."
echo ""
echo -e "\033[2m--- 6 tool calls | 3 iterations | 4,218 tokens (\$0.02) ---\033[0m"
