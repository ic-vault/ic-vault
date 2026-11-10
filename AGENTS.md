# AGENTS.md

ic-vault is a single-user encrypted vault on the Internet Computer. It is security software, written with AI agents, and nobody has reviewed it independently. Nothing is released.

Read these before changing anything:

1. `docs/threat-model.md` and `docs/crypto-spec.md`.
2. The row of your task and section 3 of `docs/implementation-plan.md`.
3. The decision records in `docs/adr/` that touch your area.

## Sources of truth

| What | Where |
|---|---|
| Threat model, owner mistakes | `docs/threat-model.md` |
| Cryptography | `docs/crypto-spec.md` |
| Setup and upkeep | `docs/operations.md` |
| Tasks, interface, state, review levels, prompts | `docs/implementation-plan.md` |
| Decisions made since | `docs/adr/` |
| The rest of the design and its reasons | `VAULT-PLAN.md` |

- The four files of `docs/` named above win over `VAULT-PLAN.md`. They change only through an ADR, except the tick on a task row. Other files under `docs/`, such as runbooks, review packages and upkeep reports, are ordinary task output.
- `VAULT-PLAN.md` also changes only through an ADR. A change to its section 7, or to `docs/crypto-spec.md`, needs the owner's approval before any code is written.
- `IMPLEMENTATION-PLAN.md` in the root is frozen. Do not edit or tick it.
- A pinned skill knows current API names and tool usage better than your memory, so use it for those. Where a skill disagrees with this file, `docs/` or `VAULT-PLAN.md` on a rule or a design choice, the project files win. Say so in your pull request. ADR 0001 lists the known cases.

## Commands

Work inside WSL on Windows. The README has the setup.

| Command | What it does | Arrives with |
|---|---|---|
| `pnpm install --frozen-lockfile --ignore-scripts` | Installs the pinned dependencies and runs no package script | M0-01 |
| `pnpm exec icp --version` | Runs the pinned icp-cli | M0-01 |
| `pnpm run setup` | Installs toolchains and dependencies | M0-03 |
| `pnpm run dev` | Local network, canister, web app | M0-03 |
| `pnpm run check` | The only gate. It must pass before every commit | M0-03 |
| `pnpm run e2e` | End-to-end tests on the local network | M0-06 |
| `pnpm run build:release` | Reproducible release build. Prints the Wasm hash | M4-02 |

- Until `pnpm run check` exists there is no gate. Say so in your pull request.
- Call project scripts with `pnpm run <name>`. A bare `pnpm doctor` runs pnpm's own check, not `scripts/doctor.sh`. pnpm releases before 12 also run their own `pnpm setup`, which edits the shell profile.

## Hard rules

1. Vault keys exist only in `apps/web/src/crypto`. Nothing else imports noble, and `platform/` uses WebCrypto only through the SDK, for identities.
2. The canister stores and returns ciphertext. It never decrypts and never logs arguments.
3. `docs/crypto-spec.md` is law. Changing a constant, label, format or flow needs an ADR and human approval.
4. Every canister method starts with its gate. A new method needs a row in the authorization matrix test first.
5. No new runtime dependency without an ADR. Exact versions only.
6. Never weaken, skip or delete a failing test. Report it.
7. No requests to any origin other than the IC. No analytics, fonts or CDNs.
8. Never call `fetchRootKey`. Never use `innerHTML`, `eval` or inline scripts.
9. Load the pinned ICP skills before writing canister or SDK code.
10. One task, one branch `task/<id>-<slug>`. Mainnet deploys are done by the human.
11. No irreversible step in one action. A destructive feature first needs a row in the owner-mistake table of `docs/threat-model.md` and its safeguard.
12. `scripts/` never contains a canister deletion or reinstall mode.
13. Nothing about a deployed vault enters this repository: no canister ids, no `.icp/` directory, no logs.
14. No log line containing arguments anywhere, including the web app and the worker.
15. The browser stores nothing except what the Internet Identity client keeps for the sign-in: its credential storage in IndexedDB and its record of who is signed in, spec 8.2 and ADR 0006. Nothing else in IndexedDB, localStorage or any cache.
16. No real secrets anywhere: not in tests, fixtures, examples or screenshots. Use generated values.
17. Every new version of anything is at least 7 days old: packages, crates, tools, container images, skill commits. pnpm enforces it for npm packages. Apply it by hand to the rest.
18. Text files use LF line endings. A CRLF changes the release hash. `.gitattributes` enforces it.
19. Where a skill says to install a tool globally, use the pinned copy, such as `pnpm exec icp`.

## Dependencies

- **Runtime.** What ships in the canister or the web app comes only from the allowlist of spec 8.4. Anything else needs an ADR, rule 5.
- **Development.** Build tools, test tools and their crates or packages enter with the task that needs them, and only what that task needs. Exact version, at least 7 days old. The pull request lists each with its version and release date. The task prompt in section 8 of the plan says "Do not add a dependency". It means nothing beyond this rule.
- **Toolchain and skills.** A new version of Node, pnpm, Rust or icp-cli, or a new skills commit, needs a new ADR that supersedes the matching part of ADR 0001.

## Tasks

- Take one task from section 4 of `docs/implementation-plan.md`. Section 8 there has the prompts for a task, a review and a spike.
- One pull request per task. Do not touch files of another lane. Ask for a task in that lane instead.
- Lanes A and B write the tests of the proof column first and watch them fail.
- Mark your row as done in the same pull request: add ✓ after its ID in `docs/implementation-plan.md`, as in `| M0-01 ✓ |`. Tick only when every item of the proof column is met. If the task has a part in lane H, tick only after the owner's step is recorded.
- If the plan or the spec is wrong or unclear, stop. Write an ADR with the question and your proposal, as `docs/adr/README.md` describes, and report. Do not work around the problem.
- Work in lane H is never done by an agent. An ADR that needs the owner's signature stays Proposed until the owner signs it.
- The monthly upkeep task follows the prompt of spec Appendix E. It ends in a report under `docs/upkeep/` and at most one pull request, and it needs no mainnet access.
- Never run anything against mainnet.

## Skills

Six official ICP skills are pinned at one commit of `dfinity/icskills`: icp-cli, canister-security, internet-identity, stable-memory, certified-variables and cycles-management. Claude Code reads `.claude/skills/`. Cursor and Codex read `.agents/skills/`. The two folders hold identical copies.

Do not edit them. `docs/adr/0001-toolchain.md` records the commit and how to update it.

## Review

Section 6 of `docs/implementation-plan.md` has the levels R0 to R3. A task takes the highest level of any file it touches.

These need a fresh-context review session and the owner's approval before merge, level R2:

- Changes in `apps/web/src/crypto`, `apps/web/src/platform` or `scripts/`.
- Changes in `access.rs`, `unlock.rs`, `sessions.rs`, `slots.rs`, `claim.rs`, `http.rs` or `upgrade.rs`, or in `vault.did`.
- Changes to section 3 of `docs/implementation-plan.md`.
- Changes to the instructions agents follow: `AGENTS.md`, `CLAUDE.md`, `docs/adr/README.md`, `.claude/` and `.agents/`.
- A new version of the toolchain, of the skills commit, or of a crypto or SDK package: `@noble/*`, `@icp-sdk/*`, `ed25519-dalek`, `sha2`, `candid` and the `ic-*` crates.

The last two items are stricter than section 6.1 of the plan, which does not name them.
