# ADR 0012: The development machine and the agents in the threat model

Status: Proposed. Needs the owner's signature, because it is a decision of lane H: where the production controller lives
Date: 2026-11-10
Task: the review of plan and architecture of 2026-11-10, finding P5
Decided by: the owner

## Question

In stage A the controller identity can replace the code of the vault. Where may that identity live, given that agents and build tools run code on the development machine?

## Facts, checked on 2026-11-10

1. Threat model, A8: a thief of the deployer key "can replace the code", and "a planted interface could capture factors at the next unlock". A10 puts malware on an unlocked device out of scope. No row covers the machine where the software is built.
2. `docs/operations.md` 11.5, step 2: "Create `vault-staging`, `vault-prod` and the backup controller." The README sets up one WSL for all work. Nothing says that `vault-prod` lives anywhere else.
3. On that machine, `cargo build`, Vite and the test runners execute code from dependencies. Rule 17 and `--ignore-scripts` lower that risk. They do not remove it.
4. Agent sessions can be linked to the owner's computer. The session that wrote this ADR was linked to it and was offered tools for a shell, files, screenshots and the clipboard. It used none of them.
5. Risk K10 already puts the tag signing key on a hardware token. Nothing asks the same for the controller.

One bad package, or one agent that follows planted instructions, would reach the key that can replace the vault code.

## Options for the production controller

| | Where `vault-prod` lives | Good | Bad |
|---|---|---|---|
| A | A second WSL instance that holds only the release scripts, Docker and icp-cli, and a hardware key for the identity if icp-cli supports one on that setup | Clean separation at no cost. No agent and no build of unreviewed code runs there | Two environments to keep updated |
| B | A separate Windows user with its own WSL | Stronger separation | More effort on every upgrade |
| C | The same WSL as development, with the identity on a hardware key | One environment | Whatever runs there can still ask the key to sign while it is plugged in |

## Decision

Proposed: option A.

- A new adversary A15 and a new owner mistake U29 in `docs/threat-model.md`.
- Operations 11.5: step 2 creates both mainnet identities and the backup controller in the operating environment. The development environment works on the local network only, so the staging rehearsal also tests the operating environment. The setup-day checklist gets one line.
- The operating environment runs only releases that passed the gate. It builds from a signed tag, and compares the hash with the one CI printed, as `build:release` already allows.
- No agent session is linked to a machine or a user account that can reach `vault-prod`.
- Whether icp-cli can keep an identity on a hardware key under WSL is not known. The staging rehearsal finds out, next to the keyring question of 11.4.

Signed: not yet.

## Consequences, once signed

- `docs/threat-model.md`: rows A15 and U29, and the count in "What the assessment shows" stays as it is, because U29 ends Low.
- `docs/operations.md` 11.5: step 2 and the checklist. `VAULT-PLAN.md`: one bullet in 9.6 and one comment in Appendix C.
- `docs/runbooks/setup-day.md` and `doctor.sh` take it up when their tasks write them, M4-03 and M4-05.
