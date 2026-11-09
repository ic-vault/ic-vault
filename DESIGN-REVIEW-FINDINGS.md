# Design review findings

Reviewed on 2026-11-09 by Claude, at the owner's instruction, instead of an outside specialist.
Reviewed: `VAULT-PLAN.md` revision 4 and section 3 of `IMPLEMENTATION-PLAN.md`, against the eleven questions of `DESIGN-REVIEW-BRIEF.md`.

## 0. What this review is and is not

- **It is not independent.** I wrote the design. A second pass by the author finds slips and missed consequences. It does not find what the author cannot see.
- **It is not a specialist's review.** Nobody with a record in reviewing password managers or key management has read this. The external audit before real secrets is now the first such reading.
- **Method.** For each question I asked who holds what, which call they can make, and what comes back. A finding is listed only when I could write down the steps.
- **Result.** Eighteen findings. Two are high. Both concern the waiting period and the lockout, not the confidentiality of stored data. No finding lets a single thief read the vault.

## 1. Verdicts at a glance

| # | Question | Verdict | Findings | How sure |
|---|---|---|---|---|
| 1 | The PIN | The offline case is acceptable for a first version if it is written down. The lock-out by a stolen Internet Identity is not, and has a cheap fix | F2, F5, F7 | Fairly sure |
| 2 | The network key | Sound as specified, but it protects one narrow case and is the most complex layer. I would defer it | F6 | Medium. This is a judgement about cost and benefit |
| 3 | The slot keys | Sound. Two small tightenings | F8, F9 | Sure about the derivations. Not sure about library behaviour |
| 4 | The unlock protocol | Two real problems: the nonce is shared by all keys, and a failed try is written after the check | F1, F3, F4 | Sure |
| 5 | Re-key | Does what it can. The screen must say what it cannot undo. A deeper re-key would add nothing | F4, F12 | Sure |
| 6 | Adding a device | Holds after correction C28 | None | Fairly sure |
| 7 | The vault id and restore | Holds. One limit that no design removes: an older genuine backup cannot be told from the newest | F13 | Fairly sure |
| 8 | The recovery kit | The derivations hold. The waiting period can be defeated by keeping the owner out, see question 4 | F1, F14 | Sure |
| 9 | The client boundary | The worker is a discipline, not a wall. Three of its messages are as good as a key export | F10, F17 | Sure |
| 10 | The claim | Honest now, but the list of who can change the code is incomplete | F11 | Sure |
| 11 | What was not asked | Sessions that outlive their unlock key, and three small gaps | F4, F15, F18 | Medium |

## 2. Question by question

### 1. The PIN

**What holds.** With one theft nobody can test a PIN offline. The stored proof key and a recorded unlock request are useless without the passkey secret, which has 256 bits. Someone who holds both passkeys gets counted tries only.

**What does not.**

- **Known and unchanged.** The passkey secret plus a state copy, or plus one recorded unlock request, allows offline testing. A recorded request is visible to boundary nodes and also to any proxy that inspects TLS on a managed machine. This is item RV1.
- **New, F2.** A try on a PIN key can be spent by anyone who holds an owner principal. No passkey is needed for a wrong try. A stolen Internet Identity can therefore keep every PIN key locked, with one wrong try per lock period. That is a lock-out by a single theft.
- **F5.** The limits are counted per key. The same PIN on three keys gives three times the guesses. And one try a day for ever is about 370 guesses a year.
- **F7.** Argon2id costs the owner a second or two at every unlock on a phone. It costs an offline attacker, who must already hold the passkey secret, about a day for all million PINs. The security of a PIN key rests on the passkey and the counter, not on the stretching.

**Recommendation.** Accept the offline case for the first version and write it into the threat model. Fix the lock-out: every unlock of a passkey key carries two proofs. The first is made from the passkey secret alone and is checked in the message hook, where a wrong one costs nothing and counts nothing. The second is made from passkey secret and PIN and is counted. Only someone who holds the passkey can then spend a try.

### 2. The network key

**What holds.** The key is verified in the browser against a public key computed from the canister id, so neither the canister nor a node can substitute a key it knows. The transport key is bound into the unlock proof. Since correction C15 the wrap format is our own.

**What it buys.** It turns one pair of thefts from enough into not enough: a copy of the canister state plus the secret of the vault passkey. That pair then still needs the Internet Identity, or the disk of an enrolled device. If both passkeys sit with the same provider, whoever has the vault passkey has the Internet Identity too, and the network key adds nothing.

**What it costs.** A threshold call, transport keys, verification, a key cache in the browser, epochs, a daily limit, three methods, and a new way to fail: a device without the cached key cannot unlock while the key service is unavailable.

**What it does not stop.** A controller of the canister, or a majority of the vault's subnet, can ask for the key themselves. It protects against readers only.

**Recommendation, F6.** Defer the network key until after the audit. Keep the place for it in the slot format. The claim of spec section 1 holds without it: every unlock key rests on a secret of 256 bits, or on a generated passphrase of at least 64 bits. This is a judgement about cost and benefit, and yours to make. I would decide the other way if your Internet Identity lives on a different authenticator than the vault passkey, for example a hardware key.

### 3. The slot keys

- **Two keys from one input.** A wrapping key and a signing key are derived from the same input with different labels. That is what a key derivation function is for. The outputs are independent.
- **Zero bytes for an absent factor.** Unambiguous, because both halves have a fixed length and the slot kind is part of the label since correction C27.
- **Key commitment.** AES-GCM does not commit to its key. An attack that tests many candidate keys with one crafted ciphertext needs candidate keys. Here every candidate depends on the passkey secret or on 256 random bits. I found no place where it applies.
- **Nonces.** Random 96-bit nonces are safe at these counts. The busiest key wraps one data key per item version.
- **F8.** The unlock proof does not name the caller. A proof made for one owner principal is valid when another owner principal submits it. I found no gain for an attacker, but naming the caller costs nothing.
- **F9.** The canister stores proof keys that a client sends. It should refuse a key that is not a valid point of full order, and use strict verification. A session is needed to plant one, so the damage is self-inflicted.

### 4. The unlock protocol

- **F1, high. The nonce is shared by all unlock keys, and the kit principal may call unlock on any key.** A thief who holds only the kit starts a recovery. During the 48 hours they do two things. They send valid kit proofs in a stream. Each one is evaluated and moves the shared nonce, so the owner's own unlocks keep arriving stale. And they spend wrong tries on the owner's PIN keys, which locks them. The owner cannot unlock, so the owner cannot cancel. After 48 hours the thief is in. The waiting period then protects nothing, unless the owner still has a copy of the kit and the Internet Identity.
- **F3. A failed try is written after the check.** If anything on the failure path traps, the try is lost with it. The failure path writes more than the success path: a counter, a lock, a log entry. A full memory or a low balance could make exactly that path trap. Wrong guesses would then be free while right ones still work.
- **F4. A session outlives the key it was opened with.** Removing an unlock key does not end the sessions opened through it, for up to eight hours. After a re-key, a session on another device still holds the old vault key. An item it saves is wrapped with the old key and can no longer be read.
- **The rest holds.** The proof binds nonce, session principal, transport key, vault id and canister id. A recorded proof cannot be replayed with another session or another transport key. The message hook reveals nothing about a PIN.

**Recommendation.** One nonce per unlock key. The kit principal may call unlock for the kit key only, and the hook refuses it while a recovery is pending and not ready. The failed try is written first, then the proof is checked, then the try is taken back on success. Removing a key ends its sessions, and a re-key ends every session except the one that made it.

### 5. Re-key

- **What it does.** New vault key, new passkey, new PIN, new kit, old keys retired. An old vault key no longer opens the current data keys.
- **What it cannot do.** It cannot take back what was readable before. Someone who had an unlocked session could read every item. A re-key does not change that, and no deeper re-key would: a copy that has been made stays made.
- **F12.** The re-key screen must say so in one sentence, and tell the owner to change the passwords that matter. The option of a "deep re-key" in item RV2 can be closed. It would add work and no protection.

### 6. Adding a device

I tried four ways to a full session without the vault key and found none after correction C28.

- **A stolen Internet Identity and a seen token.** The handoff begins once. The thief gets the handoff blob, cannot open it, and cannot pass the proof.
- **A replayed enrolment proof.** It names the handoff, the session and the transport key.
- **A used or expired handoff.** Refused.
- **A crafted slot at the end.** It is checked for kind, fields and retired credentials, and by then the vault key has been proven.

Two things stay as they are. The code on the screen is as good as the vault key for ten minutes to someone who also holds the Internet Identity. And if the network key is deferred, the step that releases it disappears and the proof moves to the last call.

### 7. The vault id and restore

- **Binding to a vault id instead of the canister.** I found nothing that breaks. Two different vaults still have different ids. The unlock proof names the canister as well. The network key belongs to the canister.
- **A stolen backup restored by a stranger.** They get what the file already gave them.
- **A tampered file.** A changed kit slot makes the owner's kit fail at once. A changed kit principal or proof key gives the planted principal nothing, because every path still needs a proof made with the real kit or the real vault key. Dropped or changed items are caught by the MAC after the first unlock.
- **F13. An older genuine file.** A backup from last month passes every check. No design removes this. The restore screen should show the date of the file and of its newest change, and the canister should mark a restored vault as unconfirmed until the client has checked the MAC.

### 8. The recovery kit

- **The derivations hold.** One secret of 256 bits yields four independent values under four labels. The check code is 40 bits of one of them and says nothing about the others.
- **The waiting period.** It is worth having only if the owner can still get in to cancel. Finding F1 shows two ways to keep the owner out. With the fixes of question 4 and the two proofs of question 1, a kit thief can no longer touch the owner's other keys.
- **F14.** Cancelling a recovery without replacing the kit lets the thief start again at once. The interface should offer only "cancel and replace the kit" and "cancel and re-key".
- **Still open.** In stage B the kit must not be a plain controller, decision D15. And the wait helps only an owner who opens the vault within it.

### 9. The client boundary

- **F10.** The worker keeps raw keys out of the page. It does not keep a script in the page from using them. Three messages are as good as a key export: a new kit, a code for adding a device, and a re-key. A script that could send them would hold the vault for good.
- **Recommendation.** Describe the worker as what it is: one audited place for key handling. And make those three messages cost a visible step: the worker answers them only after a fresh passkey answer that opens the current slot. An injected script then has to make a passkey prompt appear.
- **F17.** A PIN typed into a text field is a string, and strings cannot be erased. Collecting the digits from key events into a byte buffer avoids the string. It is optional.

### 10. The claim

The restated claim is honest. Its list of who can change the code is incomplete, F11.

- **Missing: a release that you install.** The maintainers, and every dependency of the build, can change the code through a release. Reproducible builds show that the binary matches the source. They do not show that the source is harmless.
- **Missing: anyone who can pose as the gateway.** A mis-issued certificate or a hijacked name does what a malicious gateway does.
- **Wording.** For the Internet Identity the claim should say "has taken over", not "can read".

The claim is about single thefts. It should be followed by the pairs that are enough:

| Together | Result |
|---|---|
| Kit and Internet Identity | Opens at once |
| Kit and a backup file | Reads the backup offline |
| Kit and a controller key, in stage A | Reads the state |
| Both passkeys and the PIN | Opens |
| Vault passkey secret, and a state copy or a recorded unlock | The PIN is found offline. With the Internet Identity as well, it opens |
| Vault passkey secret and a state copy, if the network key is deferred | Reads the state |

### 11. What was not asked

- **F4** above: sessions that outlive their unlock key.
- **F15.** The canister must not serve a file that lets other origins share its Internet Identity principals. A test should prove that the well-known path for alternative origins answers "not found".
- **F18.** Slot descriptors come from a single node. Wrong salts make a right PIN count as a wrong try. With two proofs a wrong passkey salt is caught in the hook and costs nothing. A wrong stretching salt is still counted. The certified unlock answer can carry a hash of the descriptor, so the client can tell the owner that a node lied.
- **Checked and fine.** Replay of calls, forged query answers, framing, the claim link, the handoff code typed into a fake page, the export header, the check code.

## 3. Findings

| # | Severity | Where | What happens | Change |
|---|---|---|---|---|
| F1 | High | Spec 6.6 Unlock, 7.2 | A kit thief keeps the owner out during the waiting period, by moving the shared nonce and by locking the owner's PIN keys, and gets in when it ends | One nonce per unlock key. The kit principal may unlock the kit key only. The hook refuses the kit principal while a recovery is pending |
| F2 | High | Spec 6.3, 7.2 | A stolen Internet Identity keeps every PIN key locked without holding any passkey | Two proofs per unlock: passkey alone, checked in the hook, then passkey plus PIN, counted |
| F3 | Medium | Spec 6.6 Unlock step 6 | A trap on the failure path loses the failed try, so wrong guesses could become free | Write the failed try first, check, take it back on success |
| F4 | Medium | Spec 7.3, 6.6 Re-key | A session survives the removal of its key. After a re-key another device can save an item nobody can read | Removing a key ends its sessions. A re-key ends all other sessions |
| F5 | Medium | Spec 6.4 Lockout | Limits are per key, and one try a day never ends | Count the thirty-day limit per vault. After 30 wrong tries that nobody cleared, the key stays locked until a full session clears it |
| F6 | Medium | Spec 6.5 | The network key adds much complexity for one narrow case | Defer it until after the audit. Your decision |
| F7 | Low | Spec 6.1 | Argon2id on a PIN costs the owner more than an attacker | Use the cheapest parameters that are still standard. Decide with spike SP2 |
| F8 | Low | Spec 6.3 | The unlock proof does not name the caller | Add the caller principal to the signed message |
| F9 | Low | Spec 7.3 | A planted proof key of low order | Refuse it when a slot is stored |
| F10 | Medium | Spec 8.1 | Three worker messages equal a key export | A fresh passkey answer before a new kit, a device code or a re-key. Describe the worker honestly |
| F11 | Medium | Spec 1 | The claim omits two ways to change the code, and the pairs that are enough | Add both, and the table of pairs |
| F12 | Low | Spec 6.6 Re-key | The screen promises more than a re-key does | One sentence on the screen. Close the deep re-key option |
| F13 | Low | Spec 6.6 Move | An older genuine backup passes every check | Show the dates. Mark a restored vault unconfirmed until the MAC was checked |
| F14 | Low | Spec 6.6 Recover | A cancelled recovery can start again at once | Cancel only together with a kit replacement or a re-key |
| F15 | Low | Spec 7.6 | Another origin could share the principal if a certain file were served | Never serve it. Test for "not found" |
| F16 | Note | Spec 6 | Derivations, nonces, key commitment, replay, enrolment, vault id | Checked. Nothing found |
| F17 | Low | Spec 8.2 | The PIN exists as a string | Collect digits into bytes. Optional |
| F18 | Low | Spec 7.3 | A lying node makes a right PIN count as wrong | A descriptor hash in the certified unlock answer |

## 4. Triage proposal

The decision on each finding is yours. This is what I would do.

| Finding | My proposal | Effect on the plans |
|---|---|---|
| F1, F2, F3, F4, F8, F9 | Correct now | The unlock call gets a second proof and a nonce per key. Passkey slots store a second proof key. Three rules for the kit principal. No new method |
| F5 | Correct now | Two numbers in the limits |
| F6 | Your decision | If deferred: three methods fewer, no transport keys, no key cache, spike SP4 and task M2-04 shrink, about ten agent sessions saved |
| F7 | Decide with spike SP2 | A parameter |
| F10 | Correct now | Three worker messages need a fresh passkey answer |
| F11, F12, F14 | Correct now | Wording in spec section 1 and on two screens |
| F13, F15, F18 | Correct now | Small additions to restore, to the tests and to the unlock answer |
| F17 | Defer | None |
| F16 | Nothing to do | None |

## 5. Triage of 2026-11-09

The owner decided on every finding the same day. The corrections are in revision 5 of `VAULT-PLAN.md`, and listed as C37 to C47 in section 3.10 of `IMPLEMENTATION-PLAN.md`. They were designed the same day and have not been reviewed themselves.

| Finding | Decision | Where it went |
|---|---|---|
| F1 | Corrected | C38: a nonce per unlock key, the kit principal limited to the kit key |
| F2 | Corrected | C39: two proofs per unlock |
| F3 | Corrected | C40: the failed try is written first |
| F4 | Corrected | C41: sessions end with their key |
| F5 | Corrected | C42: limits per vault, and a hard stop after 30 |
| F6 | The network key is deferred until after the audit | C37, spec 6.5, spec R24 |
| F7 | Waits for spike SP2 | Not applied |
| F8, F9 | Corrected | C43 |
| F10 | Corrected | C44: a fresh passkey answer for three worker requests |
| F11, F12 | Corrected | C45: the claim and the re-key screen |
| F13, F14 | Corrected | C46: restore and recovery |
| F15, F18 | Corrected | C47 |
| F16 | Nothing to do | None |
| F17 | Deferred | Not applied |

Later the same day you took the cut line, correction C48. Restore now waits until after the audit. The restore half of F13 and of C46 therefore applies when "Move a vault" is built.

## 6. What I could not judge

- **How real authenticators behave.** PRF at creation, synced passkeys on a lost device, user verification that falls back to a device PIN. Spike SP1 answers part of it.
- **The libraries.** Whether the vetKeys verification and the Ed25519 library do what their documentation says.
- **Memory encryption on the subnet.** I treated SEV-SNP as either holding or broken.
- **How people choose PINs.** My guess counts assume a random PIN. The refusal rules narrow the gap but do not close it.
- **My own blind spots.** Two fresh reviews found 28 problems in this design after I had called it finished. This review found two more of high severity. The next reader will find more.
