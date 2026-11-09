# Decision records

An ADR records one decision that settles or changes the spec, the plan, a pin or a dependency. Agents write one when they find the plan or the spec wrong or unclear, and then stop.

## Rules

- The file name is `NNNN-slug.md`. Take the next free number.
- An ADR and the change it allows go in the same pull request.
- These need the owner's signature: every decision of lane H, every change to `docs/crypto-spec.md`, and every change to section 3 of `docs/implementation-plan.md` after task M1-01. Until the owner signs, the status is Proposed and nothing may depend on it.
- Other ADRs are accepted when their pull request merges.
- An accepted ADR is not rewritten. A later ADR replaces it, and the old one gets the status "Superseded by NNNN".

## Index

| ADR | Title | Status |
|---|---|---|
| [0001](0001-toolchain.md) | Toolchain and pinned skills | Accepted |
| [0002](0002-gateway-domain.md) | Canonical gateway domain | Proposed |

## Template

```markdown
# ADR NNNN: Title

Status: Proposed, Accepted, or Superseded by NNNN
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
