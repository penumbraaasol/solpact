---
name: deploy
description: Guarded deploy / upgrade of this repo's Anchor programs to localnet, devnet, or mainnet. The only sanctioned way to run anchor deploy. Walks preflight checks, confirms cluster + authority, deploys, uploads IDL, and verifies.
argument-hint: <localnet|devnet|mainnet> [program-name] [--verifiable]
disable-model-invocation: true
allowed-tools: Bash(anchor build*) Bash(anchor keys list*) Bash(anchor idl *) Bash(solana config get*) Bash(solana address*) Bash(solana balance*) Bash(solana program show*) Bash(solana rent*) Bash(git status*) Bash(git rev-parse*)
---

Deploy request: $ARGUMENTS

## Preflight (read-only — gather before asking anything)
Cluster from CLI: !`solana config get 2>/dev/null | grep -E 'RPC URL|Keypair Path' || echo "solana CLI not found"`
Anchor.toml provider: !`grep -A3 '^\[provider\]' Anchor.toml 2>/dev/null`
Program keys: !`anchor keys list 2>/dev/null`
Deployer pubkey: !`solana address 2>/dev/null`
Git: !`git status --porcelain | wc -l | tr -d ' '` uncommitted changes @ !`git rev-parse --short HEAD 2>/dev/null`

## Procedure

### 0. Refuse early
- Target cluster `$0` must be exactly `localnet`, `devnet`, or `mainnet`. Anything else → stop.
- `mainnet` → the user must have typed "mainnet" in **this** conversation turn. If not, stop and ask for explicit confirmation. Also require a clean git tree and `--verifiable`.
- Uncommitted changes + devnet/mainnet → stop; ask the user to commit first (deploys must be reproducible).

### 1. Consistency checks
- `anchor keys list` IDs == `declare_id!()` in each `programs/*/src/lib.rs` == `[programs.$0]` in `Anchor.toml`. Mismatch → stop; never run `anchor keys sync` against devnet/mainnet (it would point the source at a different on-chain program).
- `NO_DNA=1 anchor build` (add `--verifiable` for mainnet) — must succeed with zero warnings about stack offsets.
- Report binary sizes and estimated rent: `solana rent <bytes>`.
- For an **upgrade**: `solana program show <id> -u <cluster>` → confirm the upgrade authority == deployer pubkey, and that the existing program data account is large enough (otherwise plan `solana program extend`).

### 2. Confirm with the user (AskUserQuestion, one prompt)
Show: cluster + RPC URL, deployer pubkey + SOL balance, program name(s) + ID(s), fresh deploy vs upgrade, estimated cost, git sha. Wait for an explicit yes.

### 3. Execute (each command is an `ask` permission — the user approves)
```bash
NO_DNA=1 anchor deploy --provider.cluster $0 [--program-name $1] [--verifiable]
```
Anchor 0.32+ uploads the IDL on deploy by default; if it fails, follow with `anchor idl init` / `anchor idl upgrade` for the program.
For mainnet with `--verifiable`: `anchor verify <program-id> --provider.cluster mainnet` afterwards.

### 4. Post-deploy
- `solana program show <id> -u <cluster>` → record program data address, authority, last deployed slot.
- Update `README`/`deployments.md` table (cluster · program · ID · slot · git sha · date).
- Regenerate the client if the IDL changed. Suggest a git tag: `deploy/<cluster>/<program>/<date>`.

Never: close a program, change the upgrade authority, use `--final`, or deploy with a keypair other than the configured deployer.
