---
name: new-instruction
description: Scaffold a complete Anchor instruction — Accounts struct, handler, errors, event, LiteSVM tests, and Codama client regen — following this repo's layout. Use when asked to "add an instruction", "add an ix", "implement <verb> for <program>".
argument-hint: <program> <instruction_name> [one-line behaviour]
allowed-tools: Bash(anchor build*) Bash(anchor test*) Bash(cargo *) Bash(pnpm run *) Bash(npx codama*)
---

Scaffold the instruction described by: $ARGUMENTS

## Current repo state
Programs: !`ls programs 2>/dev/null`
Existing instructions in the target program: !`ls programs/$0/src/instructions 2>/dev/null || echo "(program dir not found — check the name)"`
State structs: !`grep -rhoE 'pub struct [A-Za-z0-9_]+' programs/$0/src/state 2>/dev/null | sort -u || true`

## Procedure (do all steps; stop and ask only if the behaviour is ambiguous)

1. **Design first (write it out before touching files):**
   - Accounts table: name · type (`Signer` / `Account<T>` / `InterfaceAccount` / `Program` / `SystemAccount`) · mut? · constraints · why each constraint exists.
   - Args struct fields with types and validation rules.
   - State mutations, CPIs, emitted event fields.
   - Failure cases → error variants (reuse existing ones from `errors.rs` where they fit).
   Check the design against `.claude/rules/security.md` and name any item that applies.

2. **Implement**
   - `programs/$0/src/instructions/$1.rs`: `#[derive(Accounts)] #[instruction(args: <Name>Args)] pub struct <Name><'info>`, `#[derive(AnchorSerialize, AnchorDeserialize, Clone)] pub struct <Name>Args`, `pub fn handler(ctx: Context<<Name>>, args: <Name>Args) -> Result<()>`.
   - Export in `instructions/mod.rs`; add the one-line delegation in `lib.rs`.
   - New error variants in `errors.rs`, new event in `events.rs`, new seeds/limits in `constants.rs`.
   - Checked math, stored bumps, `emit!`, `require!` macros per the Anchor rules.

3. **Tests** — `programs/$0/tests/$1.rs` (LiteSVM): happy path, one test per constraint/error, boundary values, event assertion. Mirror in `tests/$1.test.ts` only if the TS suite is the primary suite for this repo.

4. **Build & verify**
   ```bash
   NO_DNA=1 anchor build
   cargo clippy --all-targets -- -D warnings
   cargo fmt --all
   NO_DNA=1 anchor test
   ```
   Then regenerate the client (`codama:generate` script) and run `tsc --noEmit`.

5. **Report**: accounts table, error codes added, CU consumed by the new ix (from logs), files changed, and anything you deferred.
