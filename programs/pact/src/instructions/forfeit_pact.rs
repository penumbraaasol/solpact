use anchor_lang::prelude::*;

use crate::{errors::ErrorCode, events::PactForfeited, state::Pact};

/// Permissionless crank: once the deadline passes without confirmation, anyone
/// can send the stake to the beneficiary. Rent goes back to the creator.
#[derive(Accounts)]
pub struct ForfeitPact<'info> {
    /// Anyone; just pays the transaction fee.
    pub cranker: Signer<'info>,
    /// CHECK: only receives the rent refund on close; key pinned by `has_one = creator`.
    #[account(mut)]
    pub creator: UncheckedAccount<'info>,
    /// CHECK: only receives the stake; key pinned by `has_one = beneficiary`.
    /// Unchecked (not SystemAccount) so a program-owned treasury can be a beneficiary.
    #[account(mut)]
    pub beneficiary: UncheckedAccount<'info>,
    #[account(
        mut,
        seeds = [Pact::SEED_PREFIX, pact.creator.as_ref(), &pact.id.to_le_bytes()],
        bump = pact.bump,
        has_one = creator,
        has_one = beneficiary,
        close = creator,
    )]
    pub pact: Account<'info, Pact>,
}

pub fn handle_forfeit_pact(ctx: Context<ForfeitPact>) -> Result<()> {
    let now = Clock::get()?.unix_timestamp;
    let stake = ctx.accounts.pact.stake;
    require_gte!(
        now,
        ctx.accounts.pact.deadline,
        ErrorCode::DeadlineNotReached
    );

    // The pact PDA is owned by this program, so it can debit itself directly.
    ctx.accounts.pact.sub_lamports(stake)?;
    ctx.accounts.beneficiary.add_lamports(stake)?;

    emit!(PactForfeited {
        pact: ctx.accounts.pact.key(),
        creator: ctx.accounts.pact.creator,
        beneficiary: ctx.accounts.pact.beneficiary,
        cranker: ctx.accounts.cranker.key(),
        stake,
        forfeited_at: now,
    });
    // `close = creator` then refunds the remaining rent to the creator.
    Ok(())
}
