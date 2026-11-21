use anchor_lang::prelude::*;

#[error_code]
pub enum ErrorCode {
    #[msg("Stake is below the minimum")]
    StakeTooSmall,
    #[msg("Goal must be 1..=64 bytes")]
    InvalidGoal,
    #[msg("Deadline must be in the future")]
    DeadlineInPast,
    #[msg("Deadline is too far in the future")]
    DeadlineTooFar,
    #[msg("Referee cannot be the creator")]
    SelfReferee,
    #[msg("Beneficiary cannot be the creator")]
    SelfBeneficiary,
    #[msg("The deadline has passed; the pact can only be forfeited")]
    DeadlinePassed,
    #[msg("The deadline has not passed yet")]
    DeadlineNotReached,
    #[msg("Arithmetic overflow")]
    MathOverflow,
}
