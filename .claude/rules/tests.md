---
paths:
  - "tests/**"
  - "programs/**/tests/**"
  - "**/*.test.ts"
  - "**/*.spec.ts"
---

# Test rules (loaded when editing tests)

## Which harness
| Need | Use |
|---|---|
| Fast unit tests of one program, Rust | `litesvm` in `programs/<name>/tests/*.rs` (`cargo test` / `anchor test`) |
| Single-instruction harness, Rust | `mollusk-svm` |
| TS tests against program only | `litesvm` (TS) via the Anchor LiteSVM test template |
| Mainnet state, time travel, oracle mocking, CU profiling | Surfpool: `createClient().use(surfpool())` from `@solana/surfpool/kit`, then `client.cheatcodes.*` |
| Full validator fidelity (rare) | `solana-test-validator` — justify in a comment |

## Rules
- Hermetic: no public RPC calls inside `anchor test`. Mainnet fixtures come from Surfpool's lazy cloning or checked-in account dumps (`solana account --output json`), never live calls.
- One `describe`/`mod` per instruction. Test names read as specs: `rejects_when_authority_is_not_signer`, `transfers_exact_amount_minus_fee`.
- Every Anchor constraint gets a negative test asserting the specific error code (`ErrorCode::X` / `AnchorError`), not just "it failed".
- Boundary math: zero, one, max-of-type, rounding direction, fee = 0 and fee = max.
- Assert emitted events (decode from logs or CPI event data), not only state.
- Add a CU assertion per instruction once behaviour is stable: fail if `consumed > budget * 1.1`.
- Use Surfpool cheatcodes (`surfnet_setAccount`, `surfnet_setTokenAccount`, `surfnet_timeTravel`) instead of long setup transaction chains.
- Fuzz every instruction that parses client-controlled data: `anchor fuzz init <program>` (Trident); keep fuzz targets under `trident-tests/`.

## TS specifics
- `@solana/kit` + Codama client builders only. No `@coral-xyz/anchor` `Program` objects in tests; if you need IDL-based helpers use `@anchor-lang/core`.
- Generate signers with `generatedSigner()` and fund via `airdropSigner`; never load `~/.config/solana/id.json` in tests.
- Keep the test runner declared in `Anchor.toml` `[scripts] test` in sync with `package.json`.
