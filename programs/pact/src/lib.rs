pub mod constants;
pub mod errors;
pub mod events;
pub mod instructions;
pub mod state;

use anchor_lang::prelude::*;

pub use constants::*;
pub use instructions::*;
pub use state::*;

declare_id!("HryksMC4gfLR9Q7BNu1bh33VcVhLBm1o1xC45pGxNXNa");

#[program]
pub mod pact {
    use super::*;

    pub fn create_pact(ctx: Context<CreatePact>, args: CreatePactArgs) -> Result<()> {
        instructions::create_pact::handle_create_pact(ctx, args)
    }

    pub fn confirm_pact(ctx: Context<ConfirmPact>) -> Result<()> {
        instructions::confirm_pact::handle_confirm_pact(ctx)
    }

    pub fn forfeit_pact(ctx: Context<ForfeitPact>) -> Result<()> {
        instructions::forfeit_pact::handle_forfeit_pact(ctx)
    }
}
