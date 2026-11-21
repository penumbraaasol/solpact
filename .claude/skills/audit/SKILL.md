---
name: audit
description: Security audit of the Solana programs in this repo (or a diff / specific program) using the security-auditor subagent in an isolated context. Use after changing constraints, CPIs, token transfers, authority logic, or before any deploy.
argument-hint: [program-name | --diff | path/to/file.rs]
context: fork
agent: security-auditor
background: false
---

Audit target: $ARGUMENTS (empty = every program under `programs/`; `--diff` = only files changed vs the default branch).

## Context
Changed files vs main: !`git diff --name-only origin/main...HEAD -- programs 2>/dev/null || git diff --name-only HEAD -- programs 2>/dev/null || echo "(no git diff available)"`
Programs: !`ls programs 2>/dev/null`

## Task
Perform a read-only security review and return a findings report. Do not edit files.

Work through, per instruction:
1. Account validation — signer, owner, PDA seeds/bump canonicality, `address` pins, `has_one` chains, `/// CHECK:` justifications on every `UncheckedAccount`/`AccountInfo`.
2. CPI safety — typed program accounts, signer seeds correctness, return-data handling, reentrancy via callbacks/transfer hooks.
3. Arithmetic — unchecked ops, truncating casts, rounding direction, fee math, overflow in `u64` lamport/token sums, division by zero.
4. State lifecycle — `init` vs `init_if_needed`, `close`, revival attacks, realloc, discriminator confusion between account types, stale bumps.
5. Token handling — mint/program consistency, Token-2022 extensions (transfer fee, transfer hook, permanent delegate, non-transferable, confidential), ATA assumptions, `transfer_checked` usage.
6. Authority & upgrade — two-step transfers, event emission, `remaining_accounts` validation, admin-only paths.
7. Denial-of-service & CU — unbounded loops, large `Vec`s, account size limits, stack usage.
8. Freshness: compare APIs used against the `solana-dev` skill references (`references/security.md`, `references/programs/anchor.md`) and flag deprecated patterns.

## Output format
```
# Audit: <scope>  (commit <sha>)
## Summary — N critical / N high / N medium / N low / N info
## Findings
### [SEV-#] <title>
- Location: programs/<p>/src/instructions/<f>.rs:<line>
- Issue: ...
- Impact: ...
- Recommendation: concrete code-level fix
- Test to add: name + what it asserts
## Checked and OK
- bullet list of what was verified clean (so the reviewer knows coverage)
## Follow-ups
```
Severity rubric: critical = loss of funds / full authority takeover; high = partial loss or privileged bypass; medium = griefing / DoS / incorrect accounting; low = hardening; info = style.
