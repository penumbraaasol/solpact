#!/usr/bin/env bash
# PostToolUse (Edit|Write): format the file Claude just touched. Silent on success, never blocks.
set -uo pipefail

INPUT=$(cat)
FILE_PATH=$(printf '%s' "$INPUT" | jq -r '.tool_input.file_path // empty')
[ -z "$FILE_PATH" ] && exit 0
[ -f "$FILE_PATH" ] || exit 0

case "$FILE_PATH" in
  */clients/js/src/generated/*|*/target/*|*/node_modules/*) exit 0 ;;
esac

PM="pnpm"
command -v pnpm >/dev/null 2>&1 || { command -v bun >/dev/null 2>&1 && PM="bunx"; } || PM="npx"
[ "$PM" = "pnpm" ] && PM="pnpm exec"

case "$FILE_PATH" in
  *.rs)
    if command -v rustfmt >/dev/null 2>&1; then
      rustfmt --edition 2021 "$FILE_PATH" >/dev/null 2>&1 || true
    fi
    ;;
  *.ts|*.tsx|*.js|*.mjs|*.cjs|*.json|*.md|*.yml|*.yaml)
    # Only run prettier if the project has it; skip settings.json (jq-formatted) and CLAUDE.md
    case "$FILE_PATH" in
      */.claude/settings*.json|*/CLAUDE.md|*/.claude/*.md|*/.claude/rules/*|*/.claude/skills/*) exit 0 ;;
    esac
    if [ -f "${CLAUDE_PROJECT_DIR:-.}/node_modules/.bin/prettier" ]; then
      (cd "${CLAUDE_PROJECT_DIR:-.}" && $PM prettier --write --log-level silent "$FILE_PATH" >/dev/null 2>&1) || true
    fi
    ;;
  *.toml)
    command -v taplo >/dev/null 2>&1 && taplo fmt "$FILE_PATH" >/dev/null 2>&1 || true
    ;;
esac

exit 0
