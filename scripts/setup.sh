#!/usr/bin/env bash
# One-time workspace setup for Claude Code + Solana.
# Idempotent. Run from the repo root:  bash scripts/setup.sh
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$ROOT"

ok()   { printf '  \033[32m✓\033[0m %s\n' "$*"; }
warn() { printf '  \033[33m!\033[0m %s\n' "$*"; }
miss() { printf '  \033[31m✗\033[0m %s\n' "$*"; }

echo "== Toolchain =="
need() { # name, cmd, hint
  if command -v "$2" >/dev/null 2>&1; then ok "$1: $("$2" --version 2>/dev/null | head -n1)"; else miss "$1 missing — $3"; fi
}
need rust    rustc    "https://rustup.rs"
need solana  solana   "curl --proto '=https' --tlsv1.2 -sSfL https://solana-install.solana.workers.dev | bash  (installs Agave CLI + Anchor + Surfpool)"
need anchor  anchor   "cargo install --git https://github.com/otter-sec/anchor avm --force && avm install latest && avm use latest"
need surfpool surfpool "https://docs.surfpool.run  (or the one-liner above)"
need node    node     "Node >= 20.18 — https://nodejs.org"
need jq      jq       "brew install jq | apt-get install jq   (hooks need it)"
need claude  claude   "https://code.claude.com/docs/en/setup"

if command -v node >/dev/null 2>&1; then
  v=$(node -p 'process.versions.node.split(".")[0]')
  [ "$v" -ge 20 ] || warn "Node $v is below 20.18 (Kit + Anchor 1.x TS need >= 20.18)"
fi

echo
echo "== Hooks =="
chmod +x .claude/hooks/*.sh && ok "hook scripts executable"

echo
echo "== Official solana-dev skill (Solana Foundation) =="
if [ -d .claude/skills/solana-dev ]; then
  ok "already installed at .claude/skills/solana-dev"
else
  if command -v npx >/dev/null 2>&1; then
    if npx --yes skills add solana-foundation/solana-dev-skill --project 2>/dev/null \
       || npx --yes skills add solana-foundation/solana-dev-skill 2>/dev/null; then
      ok "installed via skills CLI"
    fi
  fi
  if [ ! -d .claude/skills/solana-dev ]; then
    warn "skills CLI install didn't land in .claude/skills — falling back to git clone"
    tmp=$(mktemp -d)
    if git clone --depth 1 https://github.com/solana-foundation/solana-dev-skill "$tmp/sds" >/dev/null 2>&1; then
      mkdir -p .claude/skills
      cp -R "$tmp/sds/skills/solana-dev" .claude/skills/solana-dev
      ok "copied skills/solana-dev into .claude/skills/"
    else
      miss "could not fetch solana-dev-skill; install later: npx skills add solana-foundation/solana-dev-skill"
    fi
    rm -rf "$tmp"
  fi
fi

echo
echo "== MCP =="
ok ".mcp.json declares solana-mcp (https://mcp.solana.com/mcp) and context7 — Claude Code will ask to trust them on first launch"

echo
echo "== Local overrides =="
if [ ! -f .claude/settings.local.json ]; then
  cp .claude/settings.local.json.example .claude/settings.local.json
  ok "created .claude/settings.local.json (gitignored) — tune personal permissions there"
else
  ok ".claude/settings.local.json present"
fi
[ -f CLAUDE.local.md ] || { cp CLAUDE.local.md.example CLAUDE.local.md; ok "created CLAUDE.local.md (gitignored) for personal notes"; }

echo
echo "== Anchor project =="
if [ -f Anchor.toml ]; then
  ok "Anchor.toml found"
else
  warn "no Anchor.toml here. For a new project:  anchor init <name> --test-template litesvm --package-manager pnpm --install-agent-skills"
  warn "then move/merge the files from this scaffold into it (see README.md)."
fi

echo
echo "Next: run 'claude', then '/context' to confirm CLAUDE.md + rules loaded, and '/mcp' to trust the Solana MCP server."
echo "CLAUDE.md placeholders are already filled in for the workshop workspace."
