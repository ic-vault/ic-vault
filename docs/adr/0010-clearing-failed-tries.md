# ADR 0010: What clears the count of wrong tries, and when the app asks

Status: Proposed. Needs the owner's signature, because it changes `docs/crypto-spec.md`
Date: 2026-11-10
Task: the review of plan and architecture of 2026-11-10, finding D8
Decided by: the owner

## Question

The third limit locks a key "after 30 wrong tries that nobody cleared". What clears that count? And how does the app know which tries were "not yours"?

## Facts, checked on 2026-11-10

1. Spec 6.4: "A full session can clear a lock, and the alert about tries that were not yours says when to do so." The interface has `slot_clear_lock`, and `SlotInfo` has `needs_clearing`. The state keeps "failed tries that nobody cleared" per slot, section 3.3.
2. No document says whether a successful unlock changes that count, or whether the app calls `slot_clear_lock` by itself.
3. If every success cleared it, the limit would stop nobody who waits for the owner's next unlock. That is the hole the limit was added for, finding F5.
4. If nothing but a manual step cleared it, the owner's own typos would add up to 30 over the years and lock the key.
5. Rule 15: the app stores nothing. After a reload it cannot know which tries it made. What it has: the unlock result gives `failed_since`, the failed tries on any key since the previous unlock. And the page knows how many wrong tries it made itself since it loaded.

## Options for the app

| | Rule | Good | Bad |
|---|---|---|---|
| A | The app clears by itself when every failed try since the previous unlock was made by this page. Otherwise it asks | Own typos never add up. A question appears only when something is unexplained | A typo made before a reload leads to a question |
| B | The app always asks when `failed_since` is above zero | Simplest rule | A question after every typo. People learn to click it away, owner mistake U17 |
| C | The app never clears by itself. Clearing is a button on the unlock keys screen | Nothing automatic | Own typos add up to a lockout |

## Decision

Proposed: the canister rule below, and option A for the app.

**In the canister**

- Each key counts its wrong tries that nobody cleared. Every counted wrong try adds one. A successful unlock does not change the count.
- `slot_clear_lock` sets the count of that key to zero and ends its lock, of whatever kind. It does not touch the limit of ten tries in thirty days, which belongs to the vault.
- `needs_clearing` is true once the count has reached 30.

**In the app, after every unlock**

- `failed_since` is zero: nothing happens.
- Every failed try since the previous unlock was made by this page: the app calls `slot_clear_lock` for the key it just used, and shows one quiet line.
- Otherwise a blocking alert says how many wrong tries were made elsewhere, and since when. "They were mine" clears the key. "Not mine" clears nothing and leads to "Remove a device" and its one question, which ends in a re-key.

Signed: not yet.

## Consequences, once signed

- `docs/crypto-spec.md` 6.4: the end of the "Lockout" bullet.
- `docs/implementation-plan.md`: the proofs of M1-05, M1-11 and M5-06.
- Tests. PocketIC: a success leaves the count, `slot_clear_lock` resets it, and the vault limit stays. End to end: own typos lead to no question, and a wrong try from a second browser context leads to the blocking alert.
