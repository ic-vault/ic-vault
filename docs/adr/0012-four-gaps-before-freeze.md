# ADR 0012: The four gaps of section 3 that spike SP9 found

Status: Accepted. Signed by the owner on 2026-11-10
Date: 2026-11-10
Task: spike SP9, ADR 0011. The review of plan and architecture of 2026-11-10, finding P4
Decided by: the owner. It changes `docs/crypto-spec.md` 6.3 and 6.4 and section 7.2 of `VAULT-PLAN.md`

## Question

Spike SP9 ran one path through section 3 of the plan and found four places where the text does not say what happens. ADR 0011 lists them with a proposal each. Task M1-01 freezes section 3, so each needs an answer first.

1. How long is the encrypted label of an unlock key?
2. What does `tries_left` in `BadProof` count?
3. What happens when a sixth full session is opened?
4. Does the message hook check the code of `claim`?

## Facts, checked on 2026-11-10

1. **Label.** Spec 6.3: `label_ct = AEAD(k_meta, pad(label), ...)` and `pad(x) = u32_be(len(x)) || x || zero bytes up to the bucket size`. The buckets are defined for headers and bodies only. Without a bucket the length of the ciphertext shows the length of the name, and the canister has nothing to check.
2. **`tries_left`.** Section 3.2 has the field and no definition. Spec 6.4 says "the tries that are left". Three limits can lock a key, spec 6.4: five wrong tries in a row, ten wrong tries of the whole vault within thirty days, and 30 wrong tries that nobody cleared. A number that follows only the first limit says "4" when the next wrong try locks the key for 24 hours.
3. **Sessions.** Section 3.1: "5 full, 1 enrolment". Nothing says what the sixth unlock gets. A session ends at the canister after 30 idle minutes. A tab that crashed, a device that went offline or a `lock` call that was lost leaves a session behind for that long. The spike refused the sixth unlock with `LimitReached`. With that rule five such leftovers lock the owner out for up to 30 minutes.
4. **Who can open a session.** Only someone who passes `unlock` or finishes an enrolment, so someone who can decrypt the vault key. Such a person can already end every other session with `rekey`. Ending one through the limit gives them nothing new.
5. **Claim.** `claim(code)` compares the SHA-256 of the code with the claim hash, section 3.2. In the spike a wrong code reached the method, so it cost the vault cycles. The hook runs on one node and counts nothing. That is safe only for a secret nobody can guess. The hook already compares the token of `enroll_begin`, which has 256 bits, correction C8.
6. **Length of the claim code.** No document states it. Spec 6.6 says "a random claim code".
7. **What moves the idle time of a session.** `renew` does. The plan does not say whether other calls do. This record does not need the answer and does not give one.

## Options

**1. Label**

| | Rule | Good | Bad |
|---|---|---|---|
| A | One bucket of 64 bytes. At most 60 bytes of text | Every label has the same length. One number to check | A very long name does not fit |
| B | One bucket of 128 bytes | Longer names | 64 more bytes per key for names nobody types |
| C | Multiples of 32 bytes | Short labels stay short | The length still tells short names from long ones |

**2. `tries_left`**

| | Rule | Good | Bad |
|---|---|---|---|
| A | Five minus the wrong tries in a row | Simple | Wrong whenever one of the other two limits is closer |
| B | The smallest of what the three limits leave. Zero when the key is locked | The number is always true | The canister computes three values |
| C | Remove the field. The app reads `slots_public` | Less interface | The app cannot warn before the last try |

**3. A sixth full session**

| | Rule | Good | Bad |
|---|---|---|---|
| A | Refuse the unlock | Nothing ends by surprise | Leftover sessions lock the owner out for up to 30 minutes |
| B | End the session that would end first anyway | The owner always gets in. No new state | With six devices in use at once, one of them has to unlock again |

**4. The code of `claim`**

| | Rule | Good | Bad |
|---|---|---|---|
| A | Only the method checks it | Nothing to add | Between deploy and claim, anyone can spend the vault's cycles with wrong codes |
| B | The hook checks it too | A wrong code costs nothing. Same pattern as `enroll_begin` | A refused claim carries no reason |

## Decision

Chosen: A, B, B, B. These are the four proposals of ADR 0011.

**1. The label of an unlock key pads to 64 bytes.**

- The plaintext is `pad(label)` with one bucket of 64 bytes: four bytes of length, at most 60 bytes of UTF-8 text, then zero bytes.
- The ciphertext is always 92 bytes: nonce 12, padded text 64, tag 16.
- The worker refuses a longer label. It does not cut one, because a cut can split a character. The canister refuses a slot whose label is not 92 bytes. This holds for every kind of slot, the kit slot too.

**2. `tries_left` is the number of wrong tries this key still takes until it is locked.**

- The count includes the try that locks it. One means: the next wrong try locks this key. Zero means: this try locked it.
- It is the smallest of three numbers, and never below zero:
  - 5 minus the wrong tries in a row on this key,
  - 11 minus the wrong tries of the whole vault within the last thirty days,
  - 30 minus the wrong tries on this key that nobody cleared.
- Each number counts the try that is being answered. So the value is zero exactly when the key is locked now. The largest value is 4.
- The value can show that wrong tries were made on another key. Only someone who holds the passkey of this key ever gets a `BadProof`, and the number is no secret.

**3. A sixth full session ends the one that would end first.**

- No unlock is refused because of the number of sessions.
- Wherever a full session opens, at `unlock` and at `enroll_finish`, the canister first drops the sessions that have run out. If five are still live, it ends one: the session whose time runs out first, by idle time or by the eight hours, whichever is sooner. On a tie it is the one that was opened first, then the principal that sorts first.
- In daily use that is the session that has been idle the longest.
- The page that lost its session sees what it sees after an upgrade: its next call is refused without a reason, it reads `slots_public` and asks for an unlock.
- The limit of one enrolment session is not touched.

**4. The message hook checks the code of `claim`.**

- The claim code is 32 random bytes. The canister keeps its SHA-256.
- For `claim` the hook refuses the anonymous principal, a vault that already has an owner, a code that is not 32 bytes, and a code whose SHA-256 is not the claim hash.
- The method repeats all of it, because a call from another canister skips the hook.
- A refused claim comes back without a reason. The setup screen has one text for it: the link is wrong or was already used.

Signed: Manuel Rüedi, the owner, on 2026-11-10, in the chat with the agent that wrote this record. The agent had put the four proposals of ADR 0011 to him as four questions, each with its pick. The answer, in the owner's words: "continue with recomendation.". The agent recorded it.

One point goes beyond the four proposals: the length of the claim code, fact 6. The hook check is safe only with a code that cannot be guessed, so this record fixes it. The pull request names it.

## What the spike shows of this

`spikes/thin-slice/` was changed to follow this record, and its tests pass.

- A setup with a label of another length is refused.
- `tries_left` counts down 4, 3, 2, 1 on wrong PINs. After nine wrong tries in the vault with two good unlocks in between, the tenth is answered with 1 and the eleventh with 0, and the key is locked for 24 hours.
- A sixth unlock opens a session and ends the one that would have ended first. The other four keep working.
- A wrong claim code never reaches the method.

It does not show the method's own check of the claim code, because the spike has no second canister to call from. The matrix test of M1-03 and M1-04 does that.

## Consequences

- `docs/crypto-spec.md` 6.3: the "Padding buckets" bullet names the label. 6.4: the "Answer to a wrong proof" bullet defines the tries that are left.
- `VAULT-PLAN.md` 7.2: "What the hook does elsewhere" names `claim`.
- `docs/implementation-plan.md`:
  - 3.1: a row for the label, a row for the claim code, and the "Sessions" row.
  - 3.2: the comments on `claim_hash`, `BadProof` and `label`. No type and no method changes, so the Candid interface is the same.
  - 3.4: the hook bullet names `claim`, and the slot check names the label.
  - Proofs of M1-03, M1-04, M1-05 and M1-11.
  - M1-01 depends on SP9 and no longer on an open answer.
- `packages/shared/limits.json`, when M1-01 writes it: label bucket 64, claim code 32.
- Web app: the field for the name of an unlock key takes at most 60 bytes of UTF-8. The text for a refused claim. Both arrive with M3-04 and M5-02.
