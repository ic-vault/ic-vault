# ADR 0007: The public status shows a pending recovery

Status: Proposed. Needs the owner's signature, because it changes spec 7.2 and 7.3 and what the public can see
Date: 2026-11-10
Task: the review of plan and architecture of 2026-11-10, finding D5
Decided by: the owner

## Question

Should the watchdog be able to see that a recovery with the kit alone is running?

## Facts, checked on 2026-11-10

1. Recovery without Internet Identity waits 48 hours, spec 6.6. The wait exists so that "an owner who still has a working device" can cancel and re-key.
2. The owner learns of a pending recovery from `slots_public` and from the unlock result. Both need the owner to open the vault. Spec 6.7: "The vault speaks only when it is open."
3. The watchdog of `docs/operations.md` 11.6 calls `status` once a day and "holds no key". `Status` carries version, environment, vault id, health, kit number and kit date. Nothing about a recovery.
4. `status` is a query, answered by one node. A lying node could hide the flag. The certified unlock result stays the answer that decisions hang on.

So if the owner does not open the vault for two days, a thief with a photo of the kit gets in unseen. For a personal vault two quiet days are normal.

## Options

| | `Status` gets | Good | Bad |
|---|---|---|---|
| A | `recovery_pending : bool` | The watchdog warns within a day, inside the wait | Anyone who knows the address sees that a recovery is running |
| B | That flag, and `alerts_waiting : bool` | Also warns of a new unlock key or an owner change | Tells the public more. Every routine alert would trip the watchdog until acknowledged |
| C | Nothing | Nothing new is public | The wait protects only an owner who opens the vault every other day |

## Decision

Proposed: option A.

- `Status` gets `recovery_pending : bool`. It is true from the first valid call of the kit principal until the recovery is used, cancelled, replaced or lapsed. It carries no time.
- `watch.sh` fails when the flag is true. That is its fifth cause.
- The runbook for that alarm says: open the vault on a device you trust, read the blocking warning, and choose kit replacement or re-key.
- The default wait stays 48 hours. With a daily watchdog the owner gets at least one alarm inside it.

What a stranger learns: that someone, perhaps the owner, started a recovery. Not who, not when, and nothing about the contents.

Signed: not yet.

## Consequences, once signed

- `docs/implementation-plan.md`: `Status` in 3.2, and the proofs of M1-14 and M4-03.
- `docs/operations.md` 11.6: the watchdog's list of causes.
- `VAULT-PLAN.md`: the first row of the table in 7.2, and the `status` bullet in 7.3.
- Tests. PocketIC: the flag follows the life of a recovery through every ending of correction C31. Scripts: `watch.sh` fails on the flag.
