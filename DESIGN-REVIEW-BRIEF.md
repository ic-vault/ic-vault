# Design review brief: a personal vault on the Internet Computer

For the reviewer. Written 2026-11-09. You received two documents with this brief: `VAULT-PLAN.md`, the design, and `IMPLEMENTATION-PLAN.md`, the exact interface and the open questions.

Note for the owner, same day: this brief was written for revision 4 and was never sent. The author reviewed the design instead, see `DESIGN-REVIEW-FINDINGS.md`. Revision 5 has since deferred the network key and changed the unlock protocol. Revision 6 then took the cut line and left files and restore out of the first release. Since revision 7 there is no paid audit, and this brief is the core of the public call for review, task AG-01 of the implementation plan. Before it goes to anyone, section 1 and questions 1, 2, 4 and 7 of section 5 need updating.

## 1. The project in ten lines

- An open-source personal vault for passwords, notes and small files.
- Every user runs one canister of their own on the Internet Computer. It stores ciphertext and serves the web app. All encryption happens in the browser.
- Gate 1 is a sign-in with Internet Identity, which gives an owner principal.
- Gate 2 is a passkey with the WebAuthn PRF extension, plus a PIN. A key derived from both signs an unlock proof. The canister checks it and counts wrong tries.
- A random vault key is wrapped once per unlock key. Device keys get a second wrap with a key that a different subnet derives through vetKD.
- A recovery kit of 256 bits, written as 24 words on paper, is an unlock key, a break-glass identity and the key for backup files.
- Nothing is built yet. The author is one person who works with AI coding agents.
- Two automated reviews have already been folded in. They found 28 problems in half an hour each, which is why we ask a person.

## 2. What we ask for

- **Two days** of reading, thinking and writing, and a one-hour call afterwards.
- **Findings** in the format of section 8.
- **An answer to each question of section 5**, even if the answer is "I cannot tell from this".
- **A list of what you did not look at.**

Not asked: a code review, since there is no code. No review of the platform itself, of the user interface, or of the later self-upgrade stage.

## 3. What to read, in order

| Order | Document | Sections | What it gives you | Time |
|---|---|---|---|---|
| 1 | `VAULT-PLAN.md` | 1 and 2 | The security claim, the adversaries, the owner mistakes | 20 minutes |
| 2 | `VAULT-PLAN.md` | 6 | Primitives, keys, derivations, slots, the network key, every flow | 90 minutes |
| 3 | `VAULT-PLAN.md` | 7.1 to 7.5, 7.9 | State, access gates, limits, the rules the canister enforces | 40 minutes |
| 4 | `IMPLEMENTATION-PLAN.md` | 3.2 to 3.5 | The exact interface, the state layout, who may call what, the worker protocol | 45 minutes |
| 5 | `IMPLEMENTATION-PLAN.md` | 3.10, 7.3, 7.9 | The 36 corrections made so far, the open questions, what the earlier reviews found | 30 minutes |
| 6 | `VAULT-PLAN.md` | 8.1 to 8.3, 15 | Client boundaries and the risks we accept | 20 minutes |

## 4. Platform facts you can take as given

You do not need to know the Internet Computer. These facts are enough for the design questions.

- **Callers.** Every call carries a principal derived from the caller's public key. The platform checks the signature.
- **Update calls** go through consensus of the subnet. Their answers are certified by the subnet as a whole.
- **Query calls** are answered by a single node. A single node can lie in a query answer.
- **The message hook**, `canister_inspect_message`, runs on a single node before a call from outside is accepted. A call from another canister skips it.
- **A trap** discards every state change of the message that trapped.
- **vetKD** derives a key from the canister id, a context and an input. A threshold of the nodes of another subnet produces it. It arrives encrypted to a transport key that the browser supplied, so the canister never sees it.
- **The web app** is served by the canister through an HTTP gateway. The gateway verifies a certificate of the subnet on behalf of the browser.
- **A controller** of a canister can replace its code and read its state.
- **Internet Identity** gives the same user a different principal for every web origin.
- **The subnet** chosen for the vault has 7 nodes with SEV-SNP memory encryption.

## 5. Questions, most important first

Each question names the items of section 7.3 in `IMPLEMENTATION-PLAN.md` that it belongs to.

1. **The PIN.** Someone who holds the passkey secret and either a copy of the canister state or one recorded unlock request can test PINs offline. A stolen Internet Identity alone can also keep a PIN key locked. Is accepting both defensible for a first version? Would you insist on a proof in two steps, first the passkey alone and then the PIN, with a key that also depends on the network key? Items RV1, RV21.
2. **The network key.** Is it worth its complexity, given what it does and does not stop? Is our use of it sound: an AES key derived from the vetKey, used as an outer wrap in our own format? Item RV15, correction C15.
3. **The slot keys.** A wrapping key and an Ed25519 signing key are derived from the same input with different labels. An absent factor is 32 zero bytes. The PIN goes through Argon2id although it has six digits. Is anything wrong or pointless in spec 6.3 and 6.4?
4. **The unlock protocol.** Nonce, order of checks, counting of tries, the second limit, the rules of the message hook. Find a way to guess more often than the limits allow, or a cheaper way to lock the owner out. Spec 6.6 and 7.2, item RV20, correction C29.
5. **Re-key.** It changes the vault key, the passkey and the PIN, but not the keys of the items. What should it promise, and is that enough after a break-in? Item RV2, correction C27.
6. **Adding a device.** Since correction C28 a new device must prove that it holds the vault key. Is there still a way to a full session without it?
7. **The vault id.** Keys are bound to a vault id instead of the canister id, so a vault can move as ciphertext, and anyone can restore a backup file into a canister of their own. What breaks? Items RV3, RV11, correction C30.
8. **The recovery kit.** One secret yields a slot factor, an identity, an export key and a check code. A kit used without the Internet Identity waits 48 hours. Is the derivation sound, and is the waiting period worth having? Item RV4.
9. **The client boundary.** Keys live in a worker. A script in the page can still ask the worker to use them. Is the separation worth its cost? Items RV9, RV10.
10. **The claim.** It was restated today to cover readers only, correction C24. Is it now honest and complete?
11. **What did we not ask that we should have?**

## 6. Weaknesses we know and accept

You need not spend time on these, unless you think we underrate one.

- Whoever can alter the web app on its way to the browser reads everything at the next unlock. Spec R1.
- A compromised device reads the vault while it is unlocked.
- The recovery kit opens everything.
- Boundary nodes see who calls, when, and padded sizes.
- Any other canister can make the vault pay for refusing calls. Spec R22.
- Erasing secrets in JavaScript is best effort.
- There is no offline unlock.

## 7. Out of scope

- Consensus, SEV-SNP, the internals of Internet Identity and of authenticators.
- Usability and wording.
- Stage B, in which the vault upgrades itself. It gets its own review later.

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

- **No confidentiality agreement is needed.** Everything will be published under Apache-2.0.
- **Your findings.** We would like to publish them, with or without your name. Tell us which.
- **Afterwards.** Every finding becomes a correction, an accepted risk or a deferred item, recorded as a decision record. You get that list.
- **If the two days run out**, stop and tell us what is unread. An honest half is worth more than a rushed whole.
- **Contact.** To be filled in by the owner.
