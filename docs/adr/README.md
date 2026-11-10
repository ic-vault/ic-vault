# Decision records

An ADR records one decision that settles or changes the spec, the plan, a pin or a dependency. Agents write one when they find the plan or the spec wrong or unclear, and then stop.

## Rules

- The file name is `NNNN-slug.md`. Take the next number that is free on `main`. If two open pull requests take the same number, the one that merges second renumbers its ADR.
- An ADR and the change it allows go in the same pull request.
- These need the owner's signature: every decision of lane H, every change to `docs/crypto-spec.md` or to section 7 of `VAULT-PLAN.md`, and every change to section 3 of `docs/implementation-plan.md` after task M1-01. Until the owner signs, the status is Proposed. A pull request that depends on it merges only after the signature.
- Other ADRs are accepted when their pull request merges. Their status already says Accepted in that pull request.
- An accepted ADR is not rewritten. A later ADR replaces it in whole or in part. The old one then gets the status "Superseded by NNNN", or "Partly superseded by NNNN" with the replaced section named.

## Index

| ADR | Title | Status |
|---|---|---|
| [0001](0001-toolchain.md) | Toolchain and pinned skills | Accepted |
| [0002](0002-gateway-domain.md) | Canonical gateway domain | Accepted |
| [0003](0003-local-root-key.md) | How the app learns the root key of a local network | Accepted |
| [0004](0004-settings-hash.md) | What the settings hash covers | Accepted |
| [0005](0005-worker-kit-recovery.md) | Worker messages for recovery with the kit | Accepted |
| [0006](0006-internet-identity-client.md) | The plan and version 11 of the Internet Identity client | Accepted |
| [0007](0007-status-recovery-flag.md) | The public status shows a pending recovery | Accepted |
| [0008](0008-exclude-list-at-rekey.md) | No exclude list when a re-key creates the new passkey | Accepted |
| [0009](0009-sp1-prf-salt-rows.md) | Spike SP1 tests the PRF salt per unlock key | Accepted |
| [0010](0010-clearing-failed-tries.md) | What clears the count of wrong tries, and when the app asks | Accepted |
| [0011](0011-thin-slice-spike.md) | Spike SP9, one thin path before the freeze | Accepted |
| [0012](0012-four-gaps-before-freeze.md) | The four gaps of section 3 that spike SP9 found | Accepted |

The numbers 0011 and 0012 were first used by two proposals that the owner dropped on 2026-11-10, before any merge. One was about GitHub enforcing the review rules, the other about a separate environment for operating the vault. The words "Drop 0011 and 0012 for now" in the signatures of ADR 0003, 0004, 0007, 0008 and 0010 mean those two proposals, not the records above.

## Template

```markdown
# ADR NNNN: Title

Status: Proposed, Accepted, Superseded by NNNN, or Partly superseded by NNNN
Date: YYYY-MM-DD
Task: the task that raised it
Decided by: the owner, or the pull request that merged it

## Question

## Facts

What was checked, where, and on which day.

## Options

## Decision

## Consequences

What changes in the spec, the plan, the code or the procedures.
```
