use anchor_lang::prelude::*;

/// PDA seed prefix for pact accounts: [PACT_SEED, creator, id.to_le_bytes()].
#[constant]
pub const PACT_SEED: &[u8] = b"pact";

/// Maximum length (bytes) of the goal description stored on-chain.
pub const MAX_GOAL_LEN: usize = 64;

/// Minimum stake: 0.01 SOL. Keeps a forfeit payout above rent-exempt minimum
/// even if the beneficiary account doesn't exist yet.
#[constant]
pub const MIN_STAKE_LAMPORTS: u64 = 10_000_000;

/// Deadlines may be at most one year out.
#[constant]
pub const MAX_DURATION_SECS: i64 = 365 * 24 * 60 * 60;
