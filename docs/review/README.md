# Design review brief: a personal vault on the Internet Computer

This folder is the review package of ic-vault. Updated for revision 8 of the design by task DR-01 on 2026-11-09. Nothing is built yet.

The package has two files of its own:

- **This brief.** What the project is, what to read in which order, what we ask, and how to send findings.
- **[claims.md](claims.md).** Each part of the security claim, the mechanism that upholds it, the test that will prove it, and where the claim stops.

Everything else it points to lives in this repository and is public under Apache-2.0. The first version of this brief, written for revision 4 and never sent, stays frozen as `DESIGN-REVIEW-BRIEF.md` in the repository root.

## 1. The project in ten lines

- An open-source personal vault for passwords, notes, cards and identity documents. Files come after the first release.
- Every user runs one canister of their own on the Internet Computer. It stores ciphertext and serves the web app. All encryption happens in the browser, inside a worker. Only the offline backup tool decrypts outside it, on the owner's own machine.
- Gate 1 is a sign-in with Internet Identity, which gives an owner principal.
- Gate 2 is a vault passkey with the WebAuthn PRF extension, plus a PIN. An unlock with them carries two Ed25519 proofs. The first comes from a key derived from the passkey output alone. The message hook checks it before consensus, and the method checks it again. The second comes from a key derived from passkey output and PIN. The canister counts it, with three limits on wrong tries.
- A random vault key is wrapped with AES-256-GCM once per unlock key, and once more for backup files under a key from the kit. While a device is being added, it is also wrapped for ten minutes under the code shown on screen. The first release has two kinds of unlock key: passkey plus PIN, and the recovery kit.
- The recovery kit is a 256-bit secret, written as 24 words on paper. It is an unlock key, a break-glass identity and the key for backup files. Its unlock carries one proof. Used without the Internet Identity, it opens the vault only after a waiting period, 48 hours by default.
- A second wrap of device keys with a key derived through vetKD was designed and is deferred until after the first release.
- The canister has 29 methods and calls no other canister.
- Three AI reviews have been folded in: two by fresh agent sessions, and one by the model that wrote the design. No person other than the owner has read it, and there is no budget for an audit. That is why we ask.
- The owner is one person who builds with AI coding agents.

## 2. What we ask for

- **Findings** in the format of section 8, ranked by severity.
- **An answer to each question of section 5 that you looked at**, even if the answer is "I cannot tell from this".
- **A list of what you did not look at.**

A full reading takes about two working days. A partial review is welcome too. One question answered well is worth more than eleven answered quickly.

Not asked: a code review, since there is no code yet. No review of the platform itself, of the user interface, or of the later self-upgrade stage.

## 3. What to read, in order

| Order | Document | Sections | What it gives you | Time |
|---|---|---|---|---|
| 1 | [VAULT-PLAN.md](../../VAULT-PLAN.md#1-goals-and-the-security-claim) and [docs/threat-model.md](../threat-model.md#2-threat-model) | 1, and the threat model | The security claim, the scope of the first release, the trust stages, the adversaries, the owner mistakes | 25 minutes |
| 2 | [claims.md](claims.md) | all | Each part of the claim, its mechanism, its test, its limits | 15 minutes |
| 3 | [docs/crypto-spec.md](../crypto-spec.md#6-cryptographic-design) | 6.1 to 6.7 | Primitives, keys, derivations, slots, the deferred network key, every flow | 90 minutes |
| 4 | [VAULT-PLAN.md](../../VAULT-PLAN.md#71-state) | 7.1 to 7.5, and [7.9](../../VAULT-PLAN.md#79-rules-that-protect-the-owner-from-mistakes) | State, access gates, limits, the rules the canister enforces | 40 minutes |
| 5 | [docs/implementation-plan.md](../implementation-plan.md#32-interface) | 3.2 to 3.5 | The exact interface, the state layout, who may call what, the worker protocol | 45 minutes |
| 6 | [docs/implementation-plan.md](../implementation-plan.md#310-corrections-to-the-spec) | 3.10, [7.3](../implementation-plan.md#73-design-questions-for-a-specialist), [7.9](../implementation-plan.md#79-what-the-independent-reviews-found) and [7.10](../implementation-plan.md#710-the-authors-own-review) | The corrections made so far, the open questions, what the earlier reviews found | 40 minutes |
| 7 | [VAULT-PLAN.md](../../VAULT-PLAN.md#81-structure-and-boundaries) | 8.1 to 8.3, [9.1 to 9.3](../../VAULT-PLAN.md#91-who-controls-a-vault-canister) and [15](../../VAULT-PLAN.md#15-residual-risks) | Client boundaries, who controls a vault, reproducible builds, and the risks we accept | 30 minutes |

- Section numbers follow `VAULT-PLAN.md` throughout. Its sections 2, 6 and 11.4 to 11.6 live in `docs/`, where they are the source of truth.
- The model's own review of the design is [DESIGN-REVIEW-FINDINGS.md](../../DESIGN-REVIEW-FINDINGS.md), with findings F1 to F18. It reviewed revision 4, so it still treats the network key, restore and a paid audit as current. Section 7.10 of the plan says what came of it.

## 4. Platform facts you can take as given

You do not need to know the Internet Computer. These facts are enough for the design questions.

- **Callers.** Every call carries a principal derived from the caller's public key. The platform checks the signature.
- **Update calls** go through consensus of the subnet. Their answers are certified by the subnet as a whole.
- **Query calls** are answered by a single node. A single node can lie in a query answer.
- **The message hook**, `canister_inspect_message`, runs on a single node before a call from outside is accepted. A call from another canister skips it.
- **A trap** discards every state change of the message that trapped.
- **vetKD**, needed only for question 2, derives a key from the canister id, a context and an input. A threshold of the nodes of another subnet produces it. It arrives encrypted to a transport key that the browser supplied, so the canister never sees it.
- **The web app** is served by the canister through an HTTP gateway. The gateway verifies a certificate of the subnet on behalf of the browser.
- **A controller** of a canister can replace its code and read its state.
- **Internet Identity** gives the same user a different principal for every web origin. It treats the gateway domains `icp.net`, `icp0.io` and `ic0.app` as one origin.
- **The address.** Every vault lives at `https://<canister-id>.icp.net`, decision D8 in [ADR 0002](../adr/0002-gateway-domain.md), on condition that spike SP1 shows passkeys working there. The public suffix list makes that host a suffix of its own. The vault passkey uses the full host as its relying party id.
- **The subnet** chosen for the vault has 7 nodes with SEV-SNP memory encryption.

## 5. Questions, most important first

Each question names the items it belongs to. RV items are in [section 7.3 of the plan](../implementation-plan.md#73-design-questions-for-a-specialist), corrections C in [section 3.10 of the plan](../implementation-plan.md#310-corrections-to-the-spec), risks R in [section 15 of the spec](../../VAULT-PLAN.md#15-residual-risks), decisions D in [section 16 of the spec](../../VAULT-PLAN.md#16-open-decisions).

1. **The PIN.** A passkey key needs two proofs per unlock, C39. The first, from the passkey output alone, is checked in the message hook and again in the method, and is never counted. The second, from passkey output and PIN, is counted, with three limits. A stolen Internet Identity alone can therefore no longer spend PIN tries, RV21. What remains: someone who holds the passkey secret and either a copy of the canister state or one recorded unlock request can test PINs offline, RV1 and R24. Is that acceptable for a first release that may only hold secrets you can rotate? What would you change that does not need the network key?
   Where: [crypto spec 6.3](../crypto-spec.md#63-derivations-and-formats), [6.4](../crypto-spec.md#64-key-slots), [spec 7.2](../../VAULT-PLAN.md#72-access-gates).
2. **The deferred network key.** The design had a second wrap of device keys, with an AES key derived from a vetKey, RV15. Revision 5 deferred it, C37. It would stop only the pair of a state copy and a passkey secret, and only for a thief without the Internet Identity. It was also the most complex layer. Was deferring it right? When it returns, is our planned use sound: an outer wrap in our own format with a key derived from the vetKey, C15?
   Where: [crypto spec 6.5](../crypto-spec.md#65-network-key-deferred).
3. **The slot keys.** From the passkey output and the stretched PIN, the design derives a wrapping key and a signing key with different labels. A second signing key comes from the passkey output alone. An absent factor is 32 zero bytes. The PIN goes through Argon2id although it has only six digits or more. The slot kind is part of the key labels, C27. Is anything wrong or pointless?
   Where: [crypto spec 6.3](../crypto-spec.md#63-derivations-and-formats), [6.4](../crypto-spec.md#64-key-slots).
4. **The unlock protocol.** Each key has its own nonce, C38. The hook checks only proofs that rest on 256 bits. The method writes the failed try first, then checks, then takes the try back on success, C40. Three limits count wrong tries, C29 and C42. The proof names the caller, C43. The kit principal may try the kit key only. Find a way to guess more often than the limits allow, or a cheaper way to lock the owner out.
   Where: [crypto spec 6.4](../crypto-spec.md#64-key-slots), [6.6](../crypto-spec.md#66-flows), [spec 7.2](../../VAULT-PLAN.md#72-access-gates), [plan 3.4](../implementation-plan.md#34-who-may-call-what).
5. **Re-key.** It changes the vault key, creates a new passkey, asks for a new PIN and retires the old credential ids, C27. It ends every other session, C41. It does not change the keys of items that did not change, and the re-key screen says so, RV2 and C45. What should a re-key promise, and is that enough after a break-in?
   Where: [crypto spec 6.6](../crypto-spec.md#66-flows).
6. **Adding a device.** A new device must prove that it holds the vault key before its session becomes full, C28. Showing the code for adding a device needs a fresh passkey answer first, C44. The code itself is the secret behind a key that wraps the vault key for ten minutes. Whoever sees it and also holds the Internet Identity, or a state copy from those minutes, gets the vault key. The model's review accepted this. Is that right? Is there still a way to a full session without the vault key?
   Where: [crypto spec 6.3](../crypto-spec.md#63-derivations-and-formats), [6.6](../crypto-spec.md#66-flows), [plan 3.4](../implementation-plan.md#34-who-may-call-what), [the model's review](../../DESIGN-REVIEW-FINDINGS.md#6-adding-a-device).
7. **The vault id.** Keys are bound to a vault id instead of the canister id, so that a vault can later move as ciphertext, D16 and RV3. Since the cut line the first release has neither restore nor move, C48 and RV11, but the binding is part of the formats that freeze now. A backup file ends with a MAC under a key derived from the vault key, C30. What breaks, now or once restore exists?
   Where: [crypto spec 6.3](../crypto-spec.md#63-derivations-and-formats).
8. **The recovery kit.** One secret yields a slot factor, an identity, an export key and a check code. Used without the Internet Identity, the kit waits 48 hours by default, D12, and its principal may try the kit key only, C38. Is the derivation sound, and is the waiting period worth having, RV4?
   Where: [crypto spec 6.3](../crypto-spec.md#63-derivations-and-formats), [6.6](../crypto-spec.md#66-flows), [spec 7.2](../../VAULT-PLAN.md#72-access-gates).
9. **The client boundary.** Keys live in a worker. A script in the page can still ask the worker to use them, RV9. The session key that signs data calls lives on the UI thread, so such a script can also make calls. Three requests that equal a key export need a fresh passkey answer, C44, except in a session opened with the kit. Erasing secrets in JavaScript is best effort, RV10. Is the separation worth its cost?
   Where: [spec 8.1](../../VAULT-PLAN.md#81-structure-and-boundaries), [8.2](../../VAULT-PLAN.md#82-hardening), [plan 3.5](../implementation-plan.md#35-worker-protocol).
10. **The claim.** It covers readers only, C24. It names the five parties that can change the code, and it lists the pairs of thefts that are enough, C45. [claims.md](claims.md) maps each part to its mechanism and adds one pair that spec section 1 does not list yet. Is the claim now honest and complete?
    Where: [spec section 1](../../VAULT-PLAN.md#1-goals-and-the-security-claim), [claims.md](claims.md).
11. **What did we not ask that we should have?**

## 6. Weaknesses we know and accept

You need not spend time on these, unless you think we underrate one.

- Whoever can alter the web app on its way to the browser reads everything at the next unlock. R1 and RV6.
- A compromised device reads the vault while it is unlocked. R3.
- The recovery kit opens everything. R5 and R13.
- The passkey secret plus a state copy, or plus one recorded unlock request, finds the PIN offline. R24 and RV1, question 1.
- The code for adding a device, seen within its ten minutes, gives the vault key to someone who also holds the Internet Identity or a state copy from those minutes. Question 6.
- Boundary nodes see who calls, when, and padded sizes. R11.
- Any other canister can make the vault pay for refusing calls. R22 and RV16.
- Some answers before unlock come from a single node. RV8.
- A controller in stage A can turn the vault back with a snapshot. R23 and RV17.
- Erasing secrets in JavaScript is best effort. RV10.
- There is no offline unlock.
- Nobody independent has reviewed the design. R26.

## 7. Out of scope

- Consensus, SEV-SNP, the internals of Internet Identity and of authenticators.
- Usability and wording.
- Stage B, in which the vault upgrades itself. It gets its own review later.
- Everything after the cut line of spec section 1: files, import, unlock keys without a PIN or without a passkey, restore and moving a vault, the change of address, the launcher. The network key only as far as question 2 asks.

## 8. Format of a finding

```text
Title
Severity      critical, high, medium, low or note
Where         document and section
What happens  who the attacker is, what they hold, what they do, what they get
How sure      checked against the text, likely, or a hunch
Change        what you would do, or what should be examined further
```

For each question of section 5: your answer, how sure you are, and what would change your mind.

## 9. Practicalities

- **How to send findings.** Report them privately through GitHub, as [SECURITY.md](../../SECURITY.md) describes. That is the one channel the project has decided, D21. Please do not open a public issue about a flaw.
- **No confidentiality agreement is needed.** Everything is published under Apache-2.0.
- **Your name.** Once a finding is handled, we would like to publish it, with or without your name. Tell us which.
- **Afterwards.** Every finding is listed, and each is fixed or accepted in writing. You get that list.
- **If your time runs out**, stop and tell us what is unread. An honest half is worth more than a rushed whole.
- **Payment.** There is none, decision D10. If money for a paid review turns up, section 6.4 of the plan says how it would be booked.
