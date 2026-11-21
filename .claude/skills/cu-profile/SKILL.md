---
name: cu-profile
description: Measure and reduce compute-unit usage of an instruction — captures CU from LiteSVM/Surfpool logs, finds hot spots, applies known Anchor/Solana CU optimizations, and reports before/after. Use for "optimize CU", "this ix is expensive", "profile compute".
argument-hint: <program> <instruction>
---

Profile `$1` in program `$0`.

## Measure
1. Find or write a LiteSVM test that executes `$1` once with realistic account sizes. Read `consumed` from `TransactionMetadata.compute_units_consumed` (LiteSVM) or `surfnet_profileTransaction` (Surfpool) and print it. Record the baseline.
2. If Surfpool is available, use `surfnet_profileTransaction` for a per-CPI breakdown.

## Known wins, in order of payoff
- Remove `msg!` / `format!` from the hot path; log with `emit!` or `sol_log_data` only.
- Avoid `find_program_address` in the handler — use the stored bump with `create_program_address` via Anchor `seeds`/`bump = x.bump`.
- Box large accounts; avoid cloning `Account` data; prefer `&mut` borrows.
- Replace `Vec` iteration over client data with fixed-size arrays or bounded loops.
- Use `anchor_lang::solana_program::program::invoke_signed` via Anchor CPI builders rather than hand-built `Instruction` + `AccountMeta` vectors; or `solana-invoke` for heavy CPI paths.
- Zero-copy (`#[account(zero_copy)]` + `AccountLoader`) for large/hot state.
- Prefer `u64` math over `u128` where safe; avoid floating point entirely.
- Trim account count in the Accounts struct (each account costs CUs to deserialize); use `LazyAccount` for accounts you only read one field from.
- Check `anchor build` output for stack warnings — spills cost CUs too.

## Report
| ix | before CU | after CU | Δ | changes |
Plus the test that pins the new budget (`assert!(consumed < BUDGET)`).
