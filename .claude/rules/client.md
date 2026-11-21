---
paths:
  - "app/**"
  - "clients/**"
  - "scripts/**/*.ts"
  - "codama.*"
  - "package.json"
---

# Client / frontend rules (loaded when editing TS)

## Stack
- `@solana/kit` plugin clients: `createClient().use(signer(...)).use(solanaRpc({ rpcUrl, wsUrl, transactionConfig: { version: 1 } }))`. Reads use `maxSupportedTransactionVersion: 1`.
- Wallets: `walletSigner()` from `@solana/kit-plugin-wallet`; React via `@solana/react` (`ClientProvider`, `useClient`). Check `connected.supportedTransactionVersions.has(1)` and fall back to `version: 0` when a wallet lacks v1.
- Program access: Codama-generated builders in `clients/js/src/generated/` (regenerate with `pnpm codama:generate`; never edit by hand). Wrap them in `clients/js/src/index.ts` for ergonomic helpers.
- Compute budget on v1 lives in `message.config`; do not add ComputeBudget instructions to v1 transactions.
- Never import `@solana/web3.js` v1, `@coral-xyz/anchor`, `@solana/wallet-adapter-*`, `@solana/web3-compat`.

## Scripts (`scripts/`)
- Every script: parse `--cluster` (default `localnet`), build the client, **simulate**, print a summary (cluster, payer, program, accounts touched, lamports/tokens moved), then require `--yes` to send.
- Signers come from `signerFromFile(process.env.KEYPAIR_PATH)` — never a hardcoded path and never read/print the file contents.
- Exit non-zero on simulation failure; print decoded Anchor error names.

## Codama
- `codama.mjs` at repo root reads `target/idl/<program>.json` → `clients/js/src/generated`. Keep the IDL → client step in `package.json` and in `Anchor.toml` `[scripts]` so `anchor build` + regen stays one command.
- Commit generated code; CI checks it's in sync (`git diff --exit-code clients/js/src/generated`).

## Quality gates
- `tsc --noEmit` clean, `eslint` clean, Prettier formatted (hook runs it on edit).
- Address values are `Address`; amounts are `bigint`/`Lamports`, never JS `number` for token quantities.
