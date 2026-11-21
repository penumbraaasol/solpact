---
name: test
description: Run the program test suite (LiteSVM/Surfpool) and triage failures with Anchor-aware diagnosis. Use for "run the tests", "why is this test failing", or before declaring a change done.
argument-hint: [--skip-build] [test-name-filter]
allowed-tools: Bash(anchor build*) Bash(anchor test*) Bash(cargo test*) Bash(cargo clippy*) Bash(pnpm test*) Bash(pnpm run *) Bash(surfpool *)
---

Run and triage tests. Filter / flags: $ARGUMENTS

## Run
```bash
NO_DNA=1 anchor test $ARGUMENTS 2>&1 | tail -n 200
```
If the suite is Rust-only (`programs/*/tests` and no `tests/*.ts`), `cargo test -- --nocapture $ARGUMENTS` is equivalent and faster.

## Triage failures — map the symptom before editing anything
| Symptom | Usual cause |
|---|---|
| `AnchorError ... ConstraintSeeds / 2006` | seeds or bump mismatch between client and program; check seed order and PDA derivation |
| `ConstraintHasOne / 2001`, `ConstraintOwner / 2004` | wrong account passed or stale state |
| `AccountDiscriminatorMismatch / 3002` | wrong account type or uninitialized account |
| `AccountNotInitialized / 3012` | missing `init` ix in setup, or wrong PDA |
| `custom program error: 0x1` from Token program | insufficient funds / wrong mint |
| `Program failed to complete` + stack warning on build | stack overflow → `Box<Account>` large accounts |
| `exceeded CUs meter` | add CU assertion failure context; profile with `surfnet_profileTransaction` |
| `Transaction simulation failed: Blockhash not found` | Surfpool not running / wrong RPC in test |
| `DeclaredProgramIdMismatch / 4100` | `anchor keys sync` on localnet |
| TS `Cannot find module 'clients/js/...'` | regenerate Codama client after IDL change |
| `signature verification failed` | missing Signer in client tx, or PDA used as signer without seeds |

## Rules
- Fix the root cause in program or test, never by loosening assertions.
- A failing negative test that now passes means a constraint was removed — flag it loudly.
- Re-run the full suite after the fix. Report: pass/fail counts, CU per instruction if logged, and any flaky tests.
