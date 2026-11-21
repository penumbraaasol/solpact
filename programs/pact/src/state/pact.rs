use anchor_lang::prelude::*;

use crate::constants::{MAX_GOAL_LEN, PACT_SEED};

/// A commitment: `creator` locks `stake` lamports in this account until either
/// `referee` confirms the goal was met (stake returns to creator) or `deadline`
/// passes (stake goes to `beneficiary`).
#[account]
#[derive(InitSpace)]
pub struct Pact {
    pub creator: Pubkey,
    pub referee: Pubkey,
    pub beneficiary: Pubkey,
    pub id: u64,
    pub stake: u64,
    pub created_at: i64,
    pub deadline: i64,
    pub bump: u8,
    #[max_len(MAX_GOAL_LEN)]
    pub goal: String,
    pub _reserved: [u8; 32],
}

impl Pact {
    pub const SEED_PREFIX: &'static [u8] = PACT_SEED;
}
