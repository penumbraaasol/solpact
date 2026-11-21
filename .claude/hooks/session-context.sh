#!/usr/bin/env bash
# SessionStart: inject toolchain + cluster context (no secrets). stdout becomes Claude's context.
set -uo pipefail
export NO_DNA=1

ROOT="${CLAUDE_PROJECT_DIR:-.}"
cd "$ROOT" || exit 0

v() { command -v "$1" >/dev/null 2>&1 && "$@" 2>/dev/null | head -n1 || echo "not installed"; }

echo "== Solana workspace context =="
echo "rust:     $(v rustc --version)"
echo "solana:   $(v solana --version)"
echo "anchor:   $(v anchor --version)"
echo "surfpool: $(v surfpool --version)"
echo "node:     $(v node --version)"

if command -v solana >/dev/null 2>&1; then
  url=$(solana config get 2>/dev/null | awk -F': ' '/RPC URL/ {print $2}')
  kp=$(solana config get 2>/dev/null | awk -F': ' '/Keypair Path/ {print $2}')
  echo "cli cluster: ${url:-unknown}   (keypair path: ${kp:-unknown} — contents are off-limits)"
  case "$url" in *mainnet*) echo "!! CLI is pointed at MAINNET. Do not run mutating commands without explicit user confirmation." ;; esac
fi

if [ -f Anchor.toml ]; then
  cluster=$(awk -F'"' '/^\[provider\]/{p=1} p&&/^cluster/{print $2; exit}' Anchor.toml)
  echo "Anchor.toml cluster: ${cluster:-unset}"
  echo "programs: $(awk '/^\[programs\./{gsub(/[\[\]]/,""); print}' Anchor.toml | tr '\n' ' ')"
fi

if git rev-parse --is-inside-work-tree >/dev/null 2>&1; then
  echo "git: $(git branch --show-current 2>/dev/null) @ $(git rev-parse --short HEAD 2>/dev/null) — $(git status --porcelain | wc -l | tr -d ' ') uncommitted changes"
fi

[ -d .claude/skills/solana-dev ] || echo "note: official solana-dev skill not installed — run scripts/setup.sh"
echo "Deploys only via /deploy <cluster>. Keypairs and .env are off-limits."
exit 0
