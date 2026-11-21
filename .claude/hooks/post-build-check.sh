#!/usr/bin/env bash
# PostToolUse (Bash, if: anchor build*): after a build, verify program IDs line up and report binary sizes.
# Output on stdout is shown to Claude as additional context. Never blocks.
set -uo pipefail

ROOT="${CLAUDE_PROJECT_DIR:-.}"
cd "$ROOT" || exit 0
[ -d target/deploy ] || exit 0

echo "== post-build check =="

# 1. program IDs: keypair vs declare_id! vs Anchor.toml
MISMATCH=0
for so in target/deploy/*.so; do
  [ -e "$so" ] || continue
  name=$(basename "$so" .so)
  kp="target/deploy/${name}-keypair.json"
  [ -f "$kp" ] || continue
  kp_id=$(solana-keygen pubkey "$kp" 2>/dev/null || echo "?")
  src_id=$(grep -rhoE 'declare_id!\("[1-9A-HJ-NP-Za-km-z]{32,44}"\)' "programs/${name}/src/lib.rs" 2>/dev/null | grep -oE '[1-9A-HJ-NP-Za-km-z]{32,44}' | head -n1 || true)
  toml_id=$(grep -E "^${name}\s*=" Anchor.toml 2>/dev/null | grep -oE '[1-9A-HJ-NP-Za-km-z]{32,44}' | head -n1 || true)
  size=$(wc -c < "$so" | tr -d ' ')
  printf '%s: %s bytes\n' "$name" "$size"
  if [ -n "$src_id" ] && [ "$src_id" != "$kp_id" ]; then
    echo "  ! declare_id! ($src_id) != keypair ($kp_id) — run 'anchor keys sync' on localnet, or fix declare_id! if this is a deployed program"
    MISMATCH=1
  fi
  if [ -n "$toml_id" ] && [ "$toml_id" != "$kp_id" ]; then
    echo "  ! Anchor.toml ($toml_id) != keypair ($kp_id)"
    MISMATCH=1
  fi
done

# 2. IDL freshness vs generated client
if [ -d clients/js/src/generated ] && ls target/idl/*.json >/dev/null 2>&1; then
  newest_idl=$(ls -t target/idl/*.json | head -n1)
  newest_gen=$(find clients/js/src/generated -type f -newer "$newest_idl" | head -n1 || true)
  if [ -z "$newest_gen" ]; then
    echo "  ! IDL is newer than clients/js/src/generated — regenerate the Codama client"
  fi
fi

[ "$MISMATCH" -eq 0 ] && echo "program IDs consistent"
exit 0
