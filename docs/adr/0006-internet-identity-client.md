# ADR 0006: The plan and version 11 of the Internet Identity client

Status: Accepted, by the merge of the pull request that carries it
Date: 2026-11-10
Task: the review of plan and architecture of 2026-11-10, finding D4
Decided by: the pull request that merges it. It changes rule 15 of `AGENTS.md`, so it needs the owner's approval, level R2

## Question

The plan describes an Internet Identity client that signs out when idle, keeps one entry in IndexedDB and holds one delegation for 8 hours. ADR 0001 pins `@icp-sdk/auth` 11. What has to change?

## Facts, checked on 2026-11-10

All from the pinned internet-identity skill, `SKILL.md` and `references/older-api.md`. Nothing was run.

1. "`IdleManager` and its options (`idleOptions`, `onIdle`, `idleTimeout`, `disableIdle`) were removed" in version 9. Correction C23 and task M3-01 switch off something that no longer exists.
2. "`maxTimeToLive` bounded a delegation and capped every sign-in at 8 hours; it now bounds the session, and unset means the provider's own 30 days." Unset, Internet Identity applies "seven days idle and thirty days in total". Decision D7 says 8 hours.
3. "The delegation your calls are signed with is short-lived and replaced by the client."
4. Storage: "`credentialStorage` with `IdbCredentialStorage` (the default)", and "`stateStorage` is new and holds the record of who is signed in". The skill does not say where the default `stateStorage` keeps that record.
5. Rule 15 and spec 8.2 allow exactly one stored thing, the delegation in IndexedDB. No test checks that: the canary of section 5.3 only looks for a marker string. Spec 13 speaks of "the two allowed IndexedDB entries", which contradicts both.

## Decision

- **C23 is withdrawn.** There is no idle logout to switch off. The vault's own auto-lock is unchanged.
- **D7 stays 8 hours** and is now enforced on purpose: every sign-in passes `maxTimeToLive` of 8 hours. `maxTimeToIdle` stays unset, because 8 hours is shorter than the idle limit of Internet Identity. Spike SP6 shows that the sign-in really ends then.
- **Rule 15 is restated.** The browser stores what the Internet Identity client keeps for the sign-in and nothing else: its credential storage in IndexedDB, and its record of who is signed in. Spike SP6 records the exact entries and where they live. A new end-to-end scenario, the storage inventory, allows that list and nothing more.
- **Spike SP6 is widened**: which entries the client writes, with default storages, and whether the sign-in ends after `maxTimeToLive`.
- If SP6 shows that the record of who is signed in lands in a place the project does not want, such as a cookie, a new ADR picks another `stateStorage` before M3-01.

## Consequences

- `AGENTS.md`: rule 15.
- `docs/implementation-plan.md`: the SP6 row, a scenario in 5.3, the row of C23, the "Signed out" row of 3.6, and task M3-01.
- `VAULT-PLAN.md`: two bullets in 8.2, one line in section 13, and the row of D7 in section 16.
- Not changed here: step 2 of setup in spec 6.6 still lists "persistent storage" in the device check. It stays true, because the sign-in is stored.
