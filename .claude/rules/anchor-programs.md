---
paths:
  - "programs/**/*.rs"
  - "programs/**/Cargo.toml"
---

# Anchor program rules (loaded when editing programs/)

## Structure
- `lib.rs` holds only `declare_id!`, `#[program] mod` with one-line handler delegations (`instructions::foo::handler(ctx, args)`), and `pub mod` declarations.
- One file per instruction in `instructions/`: `pub struct Foo<'info>` (Accounts) + `pub fn handler(ctx: Context<Foo>, args: FooArgs) -> Result<()>`. Export via `instructions/mod.rs`.
- State in `state/`: `#[account] #[derive(InitSpace)] pub struct X { ... }` with `impl X { pub const SEED_PREFIX: &'static [u8] = b"x"; }`. Reserve padding (`_reserved: [u8; N]`) for future fields.
- Errors in `errors.rs` as one `#[error_code] pub enum ErrorCode`, each variant with a `#[msg("...")]`. Events in `events.rs`.
- Constants (seed prefixes, limits, bps denominators) in `constants.rs`. No magic numbers in handlers.

## Writing an instruction
1. Accounts struct first. Write every constraint as code, then re-read it against `.claude/rules/security.md`.
2. `#[instruction(args: FooArgs)]` when seeds/space depend on args; argument order must match the handler signature (Anchor 1.x enforces this).
3. Handler: validate args → compute with checked math → mutate state → CPI → `emit!(FooEvent { .. })`. Return early with `require!` / `require_keys_eq!` / `require_gte!`.
4. `Box<Account<'info, T>>` for anything over ~1 KiB or when the Accounts struct has > 10 accounts.
5. CPI helpers live beside the instruction or in `utils/cpi.rs`; PDA signer seeds as `&[&[&[u8]]]` built from stored bumps.

## Dependencies
- `anchor-lang = "1"`, `anchor-spl = "1"` (match the installed `anchor --version`) (features `token`, `associated_token`, `token_2022` as needed, `idl-build` in the `idl-build` feature list).
- Do **not** add `solana-program` / `solana-sdk` directly; use `anchor_lang::solana_program::*` / `anchor_lang::prelude::*`.
- `[features] idl-build = ["anchor-lang/idl-build", "anchor-spl/idl-build"]` must stay in sync with every anchor crate you add.
- `overflow-checks = true` in the workspace `[profile.release]`.

## Verification before finishing
- Run the Solana MCP `program_autofixer` on every program file you wrote/changed (pass `filename`); fix and re-run until `require_another_tool_call_after_fixing` is false.
- `NO_DNA=1 anchor build` (watch for stack-offset warnings → box accounts), `cargo clippy --all-targets -- -D warnings`, `cargo fmt --all --check`.
- Add/refresh LiteSVM tests in `programs/<name>/tests/` covering: happy path, each failing constraint, boundary math.
- `anchor keys list` program ID == `declare_id!()` == `Anchor.toml` `[programs.*]`.
- If the IDL changed: regenerate `clients/js` and update TS tests.
- Report CU usage for the changed instruction (from test logs or `surfnet_profileTransaction`).
