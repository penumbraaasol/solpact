use anchor_lang::prelude::*;

use crate::{errors::ErrorCode, events::PactKept, state::Pact};

/// Referee confirms the goal was met before the deadline.
/// The pact closes and stake + rent return to the creator.
#[derive(Accounts)]
pub struct ConfirmPact<'info> {
    pub referee: Signer<'info>,
    /// CHECK: only receives lamports on close; key is pinned by `has_one = creator`.
    #[account(mut)]
    pub creator: UncheckedAccount<'info>,
    #[account(
        mut,
        seeds = [Pact::SEED_PREFIX, pact.creator.as_ref(), &pact.id.to_le_bytes()],
        bump = pact.bump,
        has_one = referee,
        has_one = creator,
        close = creator,
    )]
    pub pact: Account<'info, Pact>,
}

pub fn handle_confirm_pact(ctx: Context<ConfirmPact>) -> Result<()> {
    let now = Clock::get()?.unix_timestamp;
    let pact = &ctx.accounts.pact;
    require_gt!(pact.deadline, now, ErrorCode::DeadlinePassed);

    emit!(PactKept {
        pact: pact.key(),
        creator: pact.creator,
        referee: pact.referee,
        stake: pact.stake,
        confirmed_at: now,
    });
    Ok(())
}
