# ADR 0011: GitHub enforces the review rules

Status: Accepted, by the merge of the pull request that carries it
Date: 2026-11-10
Task: the review of plan and architecture of 2026-11-10, finding P1. Widens M0-04
Decided by: the pull request that merges it. It changes `AGENTS.md`, so it needs the owner's approval, level R2. The settings themselves are a step of lane H

## Question

`AGENTS.md` says which changes need a fresh review and the owner's approval before merge. What makes sure that this happens?

## Facts, checked on 2026-11-10

1. The branch `main` of `ic-vault/ic-vault` is not protected. The GitHub API reports `protected: false`.
2. There is no workflow yet. Task M0-04 adds the gate to every pull request and blocks a merge when it is red. It says nothing about who may merge, or about direct pushes.
3. Pull requests 1 to 3 were opened under the owner's account, by agent sessions that act with the owner's GitHub access.
4. Those sessions have a tool that merges a pull request. No rule in `AGENTS.md` says that an agent must not use it.
5. GitHub does not let the author of a pull request approve it. While agents open pull requests as the owner, a required approval by the owner cannot be met.

So today the review rules hold only as long as every agent follows its instructions. Text from a web page, a dependency or a pull request comment can change what an agent does.

## Options for who opens pull requests

| | Setup | Good | Bad |
|---|---|---|---|
| A | A second GitHub account for agents, with write access and nothing more. The owner is the code owner and approves | The owner's approval is a real second step that GitHub enforces. An agent cannot merge an R2 change | One more account to keep safe |
| B | One account. `main` needs a pull request and a green gate, and no approval | Nothing to set up | The merge click is the only approval, and an agent can make it |

## Decision

Option A, and a rule that holds from today under either option.

**New hard rule 20 in `AGENTS.md`:** agents never merge a pull request and never change a repository setting. The owner merges.

**Task M0-04 is widened** and gets a part in lane H:

- The agent adds `.github/CODEOWNERS`, which names the owner for the paths of level R2: `/apps/web/src/crypto/`, `/apps/web/src/platform/`, `/scripts/`, `/canisters/vault/`, `/docs/`, `/VAULT-PLAN.md`, `/AGENTS.md`, `/CLAUDE.md`, `/.claude/`, `/.agents/`, `/.github/`, and the files that pin versions.
- The owner sets a ruleset for `main`: changes only through a pull request, the gate as a required check, one approval that must come from a code owner, a new approval after every new push, no force push, no deletion, and no bypass for anyone.
- The owner creates the account for agents and connects the agent sessions to it.
- The owner protects the own GitHub account with a hardware security key, because that account can now do what no agent can.
- `docs/runbooks/github-settings.md` lists the settings, so that `doctor.sh` or a monthly check can compare them later.

The part of lane H can be done today. It does not have to wait for the workflow: the required check is added once M0-04 has created it.

## Consequences

- `AGENTS.md`: rule 20.
- `docs/implementation-plan.md`: the row of M0-04.
- The proof of M0-04 grows: a direct push to `main` is refused, and a pull request that touches an R2 path cannot merge without the owner's approval.
