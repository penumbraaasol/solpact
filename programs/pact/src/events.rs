use anchor_lang::prelude::*;

#[event]
pub struct PactCreated {
    pub pact: Pubkey,
    pub creator: Pubkey,
    pub referee: Pubkey,
    pub beneficiary: Pubkey,
    pub id: u64,
    pub stake: u64,
    pub deadline: i64,
}

#[event]
pub struct PactKept {
    pub pact: Pubkey,
    pub creator: Pubkey,
    pub referee: Pubkey,
    pub stake: u64,
    pub confirmed_at: i64,
}

#[event]
pub struct PactForfeited {
    pub pact: Pubkey,
    pub creator: Pubkey,
    pub beneficiary: Pubkey,
    pub cranker: Pubkey,
    pub stake: u64,
    pub forfeited_at: i64,
}
