#!/bin/bash
set -e

# Run End-to-End tests with provided credentials
# Usage: ./run_e2e.sh

# Ensure we are in the project root
cd "$(dirname "$0")"

export ANTHROPIC_AUTH_TOKEN="e82dcca26eea48cbad581f957ee356a1.t6fdlA7ozx7F9YeG"
export ANTHROPIC_BASE_URL="https://open.bigmodel.cn/api/anthropic"
export ANTHROPIC_DEFAULT_HAIKU_MODEL="glm-4.7"
export ANTHROPIC_DEFAULT_OPUS_MODEL="glm-4.7"
export ANTHROPIC_DEFAULT_SONNET_MODEL="glm-4.7"
export ANTHROPIC_MODEL="glm-4.7"

echo "Running E2E Tests against: $ANTHROPIC_BASE_URL"
echo "Model: $ANTHROPIC_MODEL"

# Ensure cargo build is fresh
cargo build --tests

# Run ignored tests in the e2e_live suite
# We use --nocapture to see the stdout of the agent interactions
cargo test -p claude-agent-api --test e2e_live -- --ignored --nocapture
