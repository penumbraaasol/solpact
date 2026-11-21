#!/usr/bin/env bash
# PreToolUse (Read|Edit|Write): hard-block access to keypairs, env files, and generated code.
# Exit 2 = block; stderr is fed back to Claude as the reason.
set -euo pipefail

INPUT=$(cat)
FILE_PATH=$(printf '%s' "$INPUT" | jq -r '.tool_input.file_path // .tool_input.path // empty')
TOOL=$(printf '%s' "$INPUT" | jq -r '.tool_name // empty')
[ -z "$FILE_PATH" ] && exit 0

FILE_PATH="${FILE_PATH//\\//}"   # normalize Windows separators
HOME_DIR="${HOME:-/nonexistent}"

# Templates are fine (no secrets by convention)
case "$FILE_PATH" in *.env.example|*.env.sample|*.env.template) exit 0 ;; esac

# Patterns that are secrets regardless of tool (Read/Edit/Write)
SECRET_PATTERNS=(
  "-keypair.json"
  "/id.json"
  "$HOME_DIR/.config/solana/"
  "/.config/solana/"
  "/.env"
  ".env."
  ".pem"
  "/wallet.json"
  "/mnemonic"
  "/seed"
)

for p in "${SECRET_PATTERNS[@]}"; do
  if [[ "$FILE_PATH" == *"$p"* ]]; then
    echo "Blocked: '$FILE_PATH' looks like a keypair/secret file (matched '$p'). Never read, print, move or edit keypairs or .env files. Use 'solana-keygen pubkey <file>' or 'solana address' if you only need the public key." >&2
    exit 2
  fi
done

# Generated / build output: block edits, allow reads
if [[ "$TOOL" == "Edit" || "$TOOL" == "Write" ]]; then
  GENERATED_PATTERNS=(
    "/clients/js/src/generated/"
    "/target/"
    "/.anchor/"
    "/node_modules/"
  )
  for p in "${GENERATED_PATTERNS[@]}"; do
    if [[ "$FILE_PATH" == *"$p"* ]]; then
      echo "Blocked: '$FILE_PATH' is generated/build output. Change the source (IDL via the program, codama.mjs, or Cargo.toml) and regenerate instead." >&2
      exit 2
    fi
  done
fi

exit 0
