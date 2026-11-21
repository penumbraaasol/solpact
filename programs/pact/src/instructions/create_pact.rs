use anchor_lang::prelude::*;
use anchor_lang::system_program::{transfer, Transfer};

use crate::{constants::*, errors::ErrorCode, events::PactCreated, state::Pact};

#[derive(AnchorSerialize, AnchorDeserialize, Clone)]
pub struct CreatePactArgs {
    /// Creator-chosen id so one wallet can hold many pacts.
    pub id: u64,
    pub stake: u64,
    /// Unix timestamp (seconds).
    pub deadline: i64,
    pub referee: Pubkey,
    pub beneficiary: Pubkey,
    pub goal: String,
}

#[derive(Accounts)]
#[instruction(args: CreatePactArgs)]
pub struct CreatePact<'info> {
    #[account(mut)]
    pub creator: Signer<'info>,
    #[account(
        init,
        payer = creator,
        space = 8 + Pact::INIT_SPACE,
        seeds = [Pact::SEED_PREFIX, creator.key().as_ref(), &args.id.to_le_bytes()],
        bump,
    )]
    pub pact: Account<'info, Pact>,
    pub system_program: Program<'info, System>,
}

pub fn handle_create_pact(ctx: Context<CreatePact>, args: CreatePactArgs) -> Result<()> {
    let creator = ctx.accounts.creator.key();
    let now = Clock::get()?.unix_timestamp;

    require_gte!(args.stake, MIN_STAKE_LAMPORTS, ErrorCode::StakeTooSmall);
    require!(
        !args.goal.is_empty() && args.goal.len() <= MAX_GOAL_LEN,
        ErrorCode::InvalidGoal
    );
    require_gt!(args.deadline, now, ErrorCode::DeadlineInPast);
    let latest = now
        .checked_add(MAX_DURATION_SECS)
        .ok_or(ErrorCode::MathOverflow)?;
    require_gte!(latest, args.deadline, ErrorCode::DeadlineTooFar);
    require_keys_neq!(args.referee, creator, ErrorCode::SelfReferee);
    require_keys_neq!(args.beneficiary, creator, ErrorCode::SelfBeneficiary);

    let pact = &mut ctx.accounts.pact;
    pact.set_inner(Pact {
        creator,
        referee: args.referee,
        beneficiary: args.beneficiary,
        id: args.id,
        stake: args.stake,
        created_at: now,
        deadline: args.deadline,
        bump: ctx.bumps.pact,
        goal: args.goal,
        _reserved: [0; 32],
    });

    // Lock the stake inside the pact PDA (on top of its rent-exempt balance).
    transfer(
        CpiContext::new(
            ctx.accounts.system_program.key(),
            Transfer {
                from: ctx.accounts.creator.to_account_info(),
                to: ctx.accounts.pact.to_account_info(),
            },
        ),
        args.stake,
    )?;

    emit!(PactCreated {
        pact: ctx.accounts.pact.key(),
        creator,
        referee: args.referee,
        beneficiary: args.beneficiary,
        id: args.id,
        stake: args.stake,
        deadline: args.deadline,
    });
    Ok(())
}
