#!/usr/bin/env bash
# Stop: if program sources changed this session and no build/test ran since, nudge Claude once.
# Uses the session_id to track state in the scratch dir; never loops (respects stop_hook_active).
set -uo pipefail

INPUT=$(cat)
ACTIVE=$(printf '%s' "$INPUT" | jq -r '.stop_hook_active // false')
[ "$ACTIVE" = "true" ] && exit 0

ROOT="${CLAUDE_PROJECT_DIR:-.}"
cd "$ROOT" || exit 0
git rev-parse --is-inside-work-tree >/dev/null 2>&1 || exit 0

CHANGED_RS=$(git status --porcelain -- 'programs/**/*.rs' 'programs/**/Cargo.toml' 2>/dev/null | wc -l | tr -d ' ')
[ "$CHANGED_RS" -eq 0 ] && exit 0

# Was anything built after the newest source change?
newest_src=$(git status --porcelain -- 'programs/' 2>/dev/null | awk '{print $2}' | xargs -I{} stat -f '%m {}' {} 2>/dev/null | sort -n | tail -n1 | awk '{print $1}')
newest_so=$(ls -t target/deploy/*.so 2>/dev/null | head -n1)
so_time=0
[ -n "$newest_so" ] && so_time=$(stat -f '%m' "$newest_so" 2>/dev/null || echo 0)

if [ -z "$newest_src" ] || [ "$so_time" -ge "$newest_src" ]; then
  exit 0
fi

jq -cn '{
  decision: "block",
  reason: "Program sources under programs/ changed but have not been built since. Run `NO_DNA=1 anchor build`, `cargo clippy --all-targets -- -D warnings` and `NO_DNA=1 anchor test` (or explain why they cannot run yet), then summarize results and any CU deltas."
}'
exit 0
