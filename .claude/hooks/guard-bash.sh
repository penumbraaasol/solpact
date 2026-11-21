#!/usr/bin/env bash
# PreToolUse (Bash): cluster + key safety for shell commands.
#  - deny: dumping keypair/secret files, overwriting keypairs, piped installers
#  - ask:  anything that targets mainnet (deploy/transfer/config set/...), program closes/upgrades
# Prints structured JSON (exit 0) for "ask"; uses exit 2 for hard denies.
set -euo pipefail

INPUT=$(cat)
CMD=$(printf '%s' "$INPUT" | jq -r '.tool_input.command // empty')
[ -z "$CMD" ] && exit 0

deny() {
  echo "Blocked: $1" >&2
  exit 2
}

ask() {
  jq -cn --arg reason "$1" '{
    hookSpecificOutput: {
      hookEventName: "PreToolUse",
      permissionDecision: "ask",
      permissionDecisionReason: $reason
    }
  }'
  exit 0
}

# ---------- hard denies ----------
# Reading/printing secrets through the shell
if printf '%s' "$CMD" | grep -Eq '(cat|less|more|head|tail|bat|xxd|hexdump|strings|base64|cp|scp|rsync|echo|printf)[^|;&]*(-keypair\.json|/id\.json|\.config/solana/|\.env\b|\.pem\b)'; then
  deny "that command would read, print or copy a keypair/.env file. Use 'solana-keygen pubkey <file>' or 'solana address' for the public key only."
fi

# Overwriting an existing keypair
if printf '%s' "$CMD" | grep -Eq 'solana-keygen\s+new[^|;&]*--force'; then
  deny "'solana-keygen new --force' would overwrite an existing keypair. Generate into a new path instead."
fi
if printf '%s' "$CMD" | grep -Eq 'solana-keygen\s+new[^|;&]*-o\s+(~|\$HOME)/\.config/solana/id\.json'; then
  deny "refusing to regenerate the default CLI keypair (~/.config/solana/id.json)."
fi

# Destroying program keypairs / irreversible program ops
if printf '%s' "$CMD" | grep -Eq 'rm\s+(-[a-zA-Z]*\s+)*[^|;&]*target/deploy'; then
  deny "deleting target/deploy would destroy program keypairs. Use 'anchor clean' or remove only .so/.json artifacts you can regenerate."
fi
if printf '%s' "$CMD" | grep -Eq 'solana\s+program\s+close'; then
  deny "'solana program close' is irreversible. The user must run this themselves."
fi
if printf '%s' "$CMD" | grep -Eq 'set-upgrade-authority[^|;&]*--final'; then
  deny "making a program immutable (--final) is irreversible. The user must run this themselves."
fi

# Piped installers
if printf '%s' "$CMD" | grep -Eq '(curl|wget)[^|]*\|\s*(sudo\s+)?(ba)?sh'; then
  deny "piping a remote script into a shell is not allowed. Download it, show the user, and let them run it."
fi

# ---------- ask ----------
MAINNET_RE='(mainnet|mainnet-beta|-u\s*m\b|--url\s+m\b|--url\s+https?://[^ ]*(mainnet|rpc\.helius|triton|quicknode|alchemy))'
MUTATING_RE='(anchor\s+(deploy|upgrade|migrate|idl\s+(init|upgrade|set-authority|close|erase-authority))|solana\s+(program\s+(deploy|write-buffer|set-upgrade-authority|set-buffer-authority|extend|upgrade)|transfer|config\s+set|create-nonce-account|stake|withdraw)|spl-token\s+(create|mint|transfer|burn|authorize|close|approve|revoke|freeze|thaw))'

if printf '%s' "$CMD" | grep -Eiq "$MAINNET_RE"; then
  if printf '%s' "$CMD" | grep -Eiq "$MUTATING_RE"; then
    ask "This command targets MAINNET and mutates on-chain state. Confirm cluster, payer, and program ID before approving. Deploys should go through /deploy mainnet."
  fi
  if printf '%s' "$CMD" | grep -Eq '\bsolana\s+config\s+set'; then
    ask "This switches the default Solana CLI cluster to mainnet for every later command. Approve only if intended."
  fi
fi

# Any deploy/upgrade on any cluster that didn't come through /deploy still asks (settings.json also lists these under ask)
if printf '%s' "$CMD" | grep -Eq 'anchor\s+(deploy|upgrade)|solana\s+program\s+(deploy|upgrade)'; then
  CLUSTER=$(printf '%s' "$CMD" | grep -Eo -- '(--provider\.cluster|-u|--url)\s+\S+' | head -n1 || true)
  ask "Deploy/upgrade detected (${CLUSTER:-cluster from Anchor.toml/solana config}). Verify 'anchor keys list' matches declare_id! and the upgrade authority is the intended wallet."
fi

exit 0
