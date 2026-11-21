# Pact: put your SOL where your mouth is

An on-chain commitment escrow on Solana, written in Anchor 1.2.0.

1. **Create**: you lock SOL behind a goal ("Run 5k every day this week") with a deadline, a **referee** you trust, and a **beneficiary** (a friend, a charity, a rival).
2. **Keep it**: if the referee confirms before the deadline, the pact closes and your stake and rent come back to you.
3. **Break it**: once the deadline passes, _anyone_ can crank `forfeit_pact`. The stake goes to the beneficiary and the rent goes back to you.

## Program

| Instruction                                                    | Signer  | Rules                                                                                                        |
| -------------------------------------------------------------- | ------- | ------------------------------------------------------------------------------------------------------------ |
| `create_pact(id, stake, deadline, referee, beneficiary, goal)` | creator | stake ≥ 0.01 SOL,  goal 1–64 bytes,  `now < deadline ≤ now + 1y`, referee ≠ creator, beneficiary ≠ creator |
| `confirm_pact`                                                 | referee | `now < deadline`; closes to creator                                                                          |
| `forfeit_pact`                                                 | anyone  | `now ≥ deadline`; stake → beneficiary, rent → creator                                                        |

- PDA: `["pact", creator, id_le_bytes]`, canonical bump stored and reused.
- Events: `PactCreated`, `PactKept`, `PactForfeited`.
- CU: create ≈ 11.3k, confirm ≈ 4.7k, forfeit ≈ 5.1k.

## Layout

```
programs/pact/src/     lib.rs · instructions/ · state/ · errors.rs · events.rs · constants.rs
programs/pact/tests/   LiteSVM suite (22 tests: happy paths, every constraint, deadline boundaries, events, CU budget)
clients/js/            Codama-generated @solana/kit client + findPactPda helper
.claude/ CLAUDE.md     Claude Code workspace from https://github.com/brimigs/workshop
```

## Commands

```bash
NO_DNA=1 anchor build --arch v0   # Anchor 1.2.0; --arch v0 is required (v3 default won't load in LiteSVM/devnet)
cargo test -p pact                # LiteSVM tests
yarn install && yarn codama:generate && yarn typecheck
```
