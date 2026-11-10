# ADR 0009: Spike SP1 tests the PRF salt per unlock key

Status: Accepted, by the merge of the pull request that carries it
Date: 2026-11-10
Task: the review of plan and architecture of 2026-11-10, finding D7
Decided by: the pull request that merges it. It changes section 2 of the plan

## Question

Correction C27 made the PRF salt "random per slot". Does that work on real devices, and does SP1 find out before the freeze?

## Facts, checked on 2026-11-10

1. With one salt per slot and more than one passkey slot, an unlock that offers all passkeys at once must send a different salt for each credential. WebAuthn has a field for that, `evalByCredential`. The plan and the spec do not name it, and SP1 does not test it.
2. Spec 6.4 assumes: "A synced passkey gives the same PRF output on every device that holds it." SP1 does not test it.
3. SP1 tests each device with its own authenticator. A phone used as the passkey for a desktop browser is a different path and is not tested.
4. Review item RV5 says of C27: "It closes a real hole, but it was designed in an hour."

## Decision

SP1 gets three more tests, at both addresses of ADR 0002:

1. Two passkeys with different salts on one device, read in one request that offers both.
2. One synced passkey read on two devices. The two outputs are compared.
3. A phone as the passkey for a desktop browser, where you would use that.

If the first fails on a device you use, there are two ways out, and an ADR picks one before task M1-01:

- **One salt per vault and key epoch.** Every passkey of the vault gets the same salt. Different passkeys still give different outputs, and a re-key still changes the salt.
- **Pick the key first.** The unlock screen asks which unlock key to use, then sends one credential with its salt.

If the second fails, synced devices cannot share a slot, and each needs its own enrolment. That changes spec 6.4 by ADR.

## Consequences

- `docs/implementation-plan.md`: the SP1 row.
- Nothing else changes until the spike has run.
