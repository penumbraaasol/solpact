//! LiteSVM tests for the Pact program.
//! One module per instruction; each constraint/error has a negative test.
// LiteSVM's `TransactionResult` carries full tx metadata in its Err variant; not ours to shrink.
#![allow(clippy::result_large_err)]

use {
    anchor_lang::{
        prelude::{Clock, Pubkey},
        solana_program::{instruction::Instruction, system_program},
        AccountDeserialize, AnchorDeserialize, Discriminator, InstructionData, ToAccountMetas,
    },
    base64::{engine::general_purpose::STANDARD, Engine},
    litesvm::{types::TransactionResult, LiteSVM},
    pact::{
        errors::ErrorCode,
        events::{PactCreated, PactForfeited, PactKept},
        state::Pact,
        CreatePactArgs, MIN_STAKE_LAMPORTS,
    },
    solana_keypair::Keypair,
    solana_message::{Message, VersionedMessage},
    solana_signer::Signer,
    solana_transaction::versioned::VersionedTransaction,
};

const SOL: u64 = 1_000_000_000;
const STAKE: u64 = SOL / 2;
const DAY: i64 = 24 * 60 * 60;
/// CU budget per instruction; tests fail if usage exceeds it by >10%.
const CU_BUDGET: u64 = 20_000;

struct Env {
    svm: LiteSVM,
    creator: Keypair,
    referee: Keypair,
    /// Deliberately never funded: payouts to a brand-new account must work.
    beneficiary: Pubkey,
}

fn setup() -> Env {
    let mut svm = LiteSVM::new();
    let bytes = include_bytes!(concat!(env!("CARGO_TARGET_TMPDIR"), "/../deploy/pact.so"));
    svm.add_program(pact::id(), bytes).unwrap();
    let mut clock: Clock = svm.get_sysvar();
    clock.unix_timestamp = 1_800_000_000;
    svm.set_sysvar(&clock);

    let creator = Keypair::new();
    let referee = Keypair::new();
    svm.airdrop(&creator.pubkey(), 10 * SOL).unwrap();
    svm.airdrop(&referee.pubkey(), SOL).unwrap();
    Env {
        svm,
        creator,
        referee,
        beneficiary: Pubkey::new_unique(),
    }
}

fn now(svm: &LiteSVM) -> i64 {
    svm.get_sysvar::<Clock>().unix_timestamp
}

fn warp_to(svm: &mut LiteSVM, ts: i64) {
    let mut clock: Clock = svm.get_sysvar();
    clock.unix_timestamp = ts;
    svm.set_sysvar(&clock);
}

fn send(svm: &mut LiteSVM, ix: Instruction, signers: &[&Keypair]) -> TransactionResult {
    svm.expire_blockhash();
    let msg =
        Message::new_with_blockhash(&[ix], Some(&signers[0].pubkey()), &svm.latest_blockhash());
    let tx = VersionedTransaction::try_new(VersionedMessage::Legacy(msg), signers).unwrap();
    let res = svm.send_transaction(tx);
    if let Ok(meta) = &res {
        assert!(
            meta.compute_units_consumed <= CU_BUDGET * 11 / 10,
            "CU {} over budget",
            meta.compute_units_consumed
        );
    }
    res
}

fn assert_err(res: TransactionResult, code: impl Into<u32>) {
    let code = code.into();
    let err = res.expect_err("expected the transaction to fail");
    let dbg = format!("{:?}", err.err);
    assert!(
        dbg.contains(&format!("Custom({code})")),
        "expected Custom({code}), got {dbg}\nlogs: {:#?}",
        err.meta.logs
    );
}

fn event<E: AnchorDeserialize + Discriminator>(res: &TransactionResult) -> E {
    let logs = &res.as_ref().expect("tx should succeed").logs;
    logs.iter()
        .filter_map(|l| l.strip_prefix("Program data: "))
        .filter_map(|b64| STANDARD.decode(b64).ok())
        .find(|data| data.starts_with(E::DISCRIMINATOR))
        .map(|data| E::deserialize(&mut &data[E::DISCRIMINATOR.len()..]).unwrap())
        .expect("event not found in logs")
}

fn pact_pda(creator: &Pubkey, id: u64) -> Pubkey {
    Pubkey::find_program_address(
        &[Pact::SEED_PREFIX, creator.as_ref(), &id.to_le_bytes()],
        &pact::id(),
    )
    .0
}

fn default_args(env: &Env) -> CreatePactArgs {
    CreatePactArgs {
        id: 1,
        stake: STAKE,
        deadline: now(&env.svm) + 7 * DAY,
        referee: env.referee.pubkey(),
        beneficiary: env.beneficiary,
        goal: "Run 5k every day this week".to_string(),
    }
}

fn create_ix(creator: &Pubkey, args: CreatePactArgs) -> Instruction {
    Instruction::new_with_bytes(
        pact::id(),
        &pact::instruction::CreatePact { args: args.clone() }.data(),
        pact::accounts::CreatePact {
            creator: *creator,
            pact: pact_pda(creator, args.id),
            system_program: system_program::ID,
        }
        .to_account_metas(None),
    )
}

fn confirm_ix(referee: &Pubkey, creator: &Pubkey, pact: Pubkey) -> Instruction {
    Instruction::new_with_bytes(
        pact::id(),
        &pact::instruction::ConfirmPact {}.data(),
        pact::accounts::ConfirmPact {
            referee: *referee,
            creator: *creator,
            pact,
        }
        .to_account_metas(None),
    )
}

fn forfeit_ix(
    cranker: &Pubkey,
    creator: &Pubkey,
    beneficiary: &Pubkey,
    pact: Pubkey,
) -> Instruction {
    Instruction::new_with_bytes(
        pact::id(),
        &pact::instruction::ForfeitPact {}.data(),
        pact::accounts::ForfeitPact {
            cranker: *cranker,
            creator: *creator,
            beneficiary: *beneficiary,
            pact,
        }
        .to_account_metas(None),
    )
}

/// Creates the default pact and returns (pact address, deadline).
fn create_default(env: &mut Env) -> (Pubkey, i64) {
    let args = default_args(env);
    let deadline = args.deadline;
    let ix = create_ix(&env.creator.pubkey(), args);
    send(&mut env.svm, ix, &[&env.creator]).unwrap();
    (pact_pda(&env.creator.pubkey(), 1), deadline)
}

fn balance(svm: &LiteSVM, key: &Pubkey) -> u64 {
    svm.get_account(key).map(|a| a.lamports).unwrap_or(0)
}

fn is_closed(svm: &LiteSVM, key: &Pubkey) -> bool {
    svm.get_account(key)
        .is_none_or(|a| a.lamports == 0 && a.data.is_empty())
}

mod create_pact {
    use super::*;

    fn create_with(env: &mut Env, f: impl FnOnce(&mut CreatePactArgs)) -> TransactionResult {
        let mut args = default_args(env);
        f(&mut args);
        let ix = create_ix(&env.creator.pubkey(), args);
        send(&mut env.svm, ix, &[&env.creator])
    }

    #[test]
    fn locks_stake_and_stores_state() {
        let mut env = setup();
        let args = default_args(&env);
        let creator = env.creator.pubkey();
        let before = balance(&env.svm, &creator);
        let res = send(
            &mut env.svm,
            create_ix(&creator, args.clone()),
            &[&env.creator],
        );
        println!(
            "create_pact CU: {}",
            res.as_ref().unwrap().compute_units_consumed
        );

        let pda = pact_pda(&creator, 1);
        let acct = env.svm.get_account(&pda).unwrap();
        let rent = env.svm.minimum_balance_for_rent_exemption(acct.data.len());
        assert_eq!(acct.lamports, rent + STAKE);
        assert!(before - balance(&env.svm, &creator) >= rent + STAKE);

        let state = Pact::try_deserialize(&mut acct.data.as_slice()).unwrap();
        assert_eq!(state.creator, creator);
        assert_eq!(state.referee, args.referee);
        assert_eq!(state.beneficiary, args.beneficiary);
        assert_eq!(state.stake, STAKE);
        assert_eq!(state.deadline, args.deadline);
        assert_eq!(state.created_at, now(&env.svm));
        assert_eq!(state.goal, args.goal);

        let ev: PactCreated = event(&res);
        assert_eq!(ev.pact, pda);
        assert_eq!(ev.stake, STAKE);
        assert_eq!(ev.deadline, args.deadline);
    }

    #[test]
    fn allows_multiple_pacts_per_creator() {
        let mut env = setup();
        create_with(&mut env, |a| a.id = 1).unwrap();
        create_with(&mut env, |a| a.id = 2).unwrap();
    }

    #[test]
    fn accepts_minimum_stake_and_max_goal() {
        let mut env = setup();
        create_with(&mut env, |a| {
            a.stake = MIN_STAKE_LAMPORTS;
            a.goal = "x".repeat(64);
        })
        .unwrap();
    }

    #[test]
    fn rejects_duplicate_id() {
        let mut env = setup();
        create_with(&mut env, |_| {}).unwrap();
        assert!(create_with(&mut env, |_| {}).is_err());
    }

    #[test]
    fn rejects_stake_below_minimum() {
        let mut env = setup();
        assert_err(
            create_with(&mut env, |a| a.stake = MIN_STAKE_LAMPORTS - 1),
            ErrorCode::StakeTooSmall,
        );
    }

    #[test]
    fn rejects_empty_goal() {
        let mut env = setup();
        assert_err(
            create_with(&mut env, |a| a.goal.clear()),
            ErrorCode::InvalidGoal,
        );
    }

    #[test]
    fn rejects_goal_too_long() {
        let mut env = setup();
        assert_err(
            create_with(&mut env, |a| a.goal = "x".repeat(65)),
            ErrorCode::InvalidGoal,
        );
    }

    #[test]
    fn rejects_deadline_now() {
        let mut env = setup();
        let t = now(&env.svm);
        assert_err(
            create_with(&mut env, |a| a.deadline = t),
            ErrorCode::DeadlineInPast,
        );
    }

    #[test]
    fn rejects_deadline_over_a_year() {
        let mut env = setup();
        let t = now(&env.svm);
        create_with(&mut env, |a| {
            a.id = 7;
            a.deadline = t + 365 * DAY;
        })
        .unwrap();
        assert_err(
            create_with(&mut env, |a| a.deadline = t + 365 * DAY + 1),
            ErrorCode::DeadlineTooFar,
        );
    }

    #[test]
    fn rejects_self_referee() {
        let mut env = setup();
        let me = env.creator.pubkey();
        assert_err(
            create_with(&mut env, |a| a.referee = me),
            ErrorCode::SelfReferee,
        );
    }

    #[test]
    fn rejects_self_beneficiary() {
        let mut env = setup();
        let me = env.creator.pubkey();
        assert_err(
            create_with(&mut env, |a| a.beneficiary = me),
            ErrorCode::SelfBeneficiary,
        );
    }
}

mod confirm_pact {
    use super::*;

    #[test]
    fn returns_stake_and_rent_to_creator() {
        let mut env = setup();
        let (pda, _) = create_default(&mut env);
        let creator = env.creator.pubkey();
        let locked = balance(&env.svm, &pda);
        let before = balance(&env.svm, &creator);

        let res = send(
            &mut env.svm,
            confirm_ix(&env.referee.pubkey(), &creator, pda),
            &[&env.referee],
        );
        println!(
            "confirm_pact CU: {}",
            res.as_ref().unwrap().compute_units_consumed
        );

        assert_eq!(balance(&env.svm, &creator), before + locked);
        assert!(is_closed(&env.svm, &pda));
        let ev: PactKept = event(&res);
        assert_eq!(ev.pact, pda);
        assert_eq!(ev.stake, STAKE);
    }

    #[test]
    fn works_one_second_before_deadline() {
        let mut env = setup();
        let (pda, deadline) = create_default(&mut env);
        warp_to(&mut env.svm, deadline - 1);
        send(
            &mut env.svm,
            confirm_ix(&env.referee.pubkey(), &env.creator.pubkey(), pda),
            &[&env.referee],
        )
        .unwrap();
    }

    #[test]
    fn rejects_non_referee_signer() {
        let mut env = setup();
        let (pda, _) = create_default(&mut env);
        let impostor = Keypair::new();
        env.svm.airdrop(&impostor.pubkey(), SOL).unwrap();
        let res = send(
            &mut env.svm,
            confirm_ix(&impostor.pubkey(), &env.creator.pubkey(), pda),
            &[&impostor],
        );
        assert_err(res, anchor_lang::error::ErrorCode::ConstraintHasOne);
    }

    #[test]
    fn rejects_creator_self_confirm() {
        let mut env = setup();
        let (pda, _) = create_default(&mut env);
        let c = env.creator.pubkey();
        let res = send(&mut env.svm, confirm_ix(&c, &c, pda), &[&env.creator]);
        assert_err(res, anchor_lang::error::ErrorCode::ConstraintHasOne);
    }

    #[test]
    fn rejects_wrong_refund_destination() {
        let mut env = setup();
        let (pda, _) = create_default(&mut env);
        let thief = Pubkey::new_unique();
        let res = send(
            &mut env.svm,
            confirm_ix(&env.referee.pubkey(), &thief, pda),
            &[&env.referee],
        );
        assert_err(res, anchor_lang::error::ErrorCode::ConstraintHasOne);
    }

    #[test]
    fn rejects_at_deadline() {
        let mut env = setup();
        let (pda, deadline) = create_default(&mut env);
        warp_to(&mut env.svm, deadline);
        let res = send(
            &mut env.svm,
            confirm_ix(&env.referee.pubkey(), &env.creator.pubkey(), pda),
            &[&env.referee],
        );
        assert_err(res, ErrorCode::DeadlinePassed);
    }
}

mod forfeit_pact {
    use super::*;

    fn cranker(env: &mut Env) -> Keypair {
        let k = Keypair::new();
        env.svm.airdrop(&k.pubkey(), SOL).unwrap();
        k
    }

    #[test]
    fn pays_beneficiary_and_refunds_rent_to_creator() {
        let mut env = setup();
        let (pda, deadline) = create_default(&mut env);
        let creator = env.creator.pubkey();
        let rent = balance(&env.svm, &pda) - STAKE;
        let creator_before = balance(&env.svm, &creator);
        assert_eq!(balance(&env.svm, &env.beneficiary), 0);

        warp_to(&mut env.svm, deadline);
        let c = cranker(&mut env);
        let res = send(
            &mut env.svm,
            forfeit_ix(&c.pubkey(), &creator, &env.beneficiary, pda),
            &[&c],
        );
        println!(
            "forfeit_pact CU: {}",
            res.as_ref().unwrap().compute_units_consumed
        );

        assert_eq!(balance(&env.svm, &env.beneficiary), STAKE);
        assert_eq!(balance(&env.svm, &creator), creator_before + rent);
        assert!(is_closed(&env.svm, &pda));
        let ev: PactForfeited = event(&res);
        assert_eq!(ev.beneficiary, env.beneficiary);
        assert_eq!(ev.cranker, c.pubkey());
        assert_eq!(ev.stake, STAKE);
    }

    #[test]
    fn rejects_before_deadline() {
        let mut env = setup();
        let (pda, deadline) = create_default(&mut env);
        warp_to(&mut env.svm, deadline - 1);
        let c = cranker(&mut env);
        let res = send(
            &mut env.svm,
            forfeit_ix(&c.pubkey(), &env.creator.pubkey(), &env.beneficiary, pda),
            &[&c],
        );
        assert_err(res, ErrorCode::DeadlineNotReached);
    }

    #[test]
    fn rejects_wrong_beneficiary() {
        let mut env = setup();
        let (pda, deadline) = create_default(&mut env);
        warp_to(&mut env.svm, deadline);
        let c = cranker(&mut env);
        let res = send(
            &mut env.svm,
            forfeit_ix(&c.pubkey(), &env.creator.pubkey(), &c.pubkey(), pda),
            &[&c],
        );
        assert_err(res, anchor_lang::error::ErrorCode::ConstraintHasOne);
    }

    #[test]
    fn rejects_wrong_rent_destination() {
        let mut env = setup();
        let (pda, deadline) = create_default(&mut env);
        warp_to(&mut env.svm, deadline);
        let c = cranker(&mut env);
        let res = send(
            &mut env.svm,
            forfeit_ix(&c.pubkey(), &c.pubkey(), &env.beneficiary, pda),
            &[&c],
        );
        assert_err(res, anchor_lang::error::ErrorCode::ConstraintHasOne);
    }

    #[test]
    fn cannot_forfeit_twice_or_after_confirm() {
        let mut env = setup();
        let (pda, deadline) = create_default(&mut env);
        warp_to(&mut env.svm, deadline);
        let c = cranker(&mut env);
        let ix = forfeit_ix(&c.pubkey(), &env.creator.pubkey(), &env.beneficiary, pda);
        send(&mut env.svm, ix.clone(), &[&c]).unwrap();
        assert_err(
            send(&mut env.svm, ix, &[&c]),
            anchor_lang::error::ErrorCode::AccountNotInitialized,
        );
        let res = send(
            &mut env.svm,
            confirm_ix(&env.referee.pubkey(), &env.creator.pubkey(), pda),
            &[&env.referee],
        );
        assert_err(res, anchor_lang::error::ErrorCode::AccountNotInitialized);
    }
}
