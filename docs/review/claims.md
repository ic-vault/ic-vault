# The claim and what upholds it

Part of the [review package](README.md). This file maps the security claim of [spec section 1](../../VAULT-PLAN.md#1-goals-and-the-security-claim) to the mechanisms of the design, and to the tests that will prove them.

Nothing is built yet. "Proven by" therefore names the task whose proof column will show it, from [section 4 of the plan](../implementation-plan.md#4-work-packages). Task AG-01 replaces those names with the code and the tests before the release gate.

Adversaries A1 to A14 are rows of the [threat model](../threat-model.md#2-threat-model). Spec sections 6.x are in the [crypto spec](../crypto-spec.md#6-cryptographic-design). Items RV, C, R and D are explained at the top of section 5 of the [brief](README.md#5-questions-most-important-first).

## The claim

Whoever can read or copy what one of the following holds, but cannot change the code your browser runs, cannot read vault contents:

1. the nodes that host the canister
2. the network path, boundary nodes and gateways
3. the user's Internet Identity, even when someone has taken it over completely
4. the project maintainers and their release channel
5. a stolen, locked device
6. a complete copy of the canister state

## What every row rests on

- **Encryption happens only in the browser.** The worker holds the vault key. The canister stores and returns ciphertext and never sees a key. [Crypto spec 6.2](../crypto-spec.md#62-keys-and-secrets), [spec 7.1](../../VAULT-PLAN.md#71-state).
  Proven by: the canary of M3-08, a marker stored as a password that never appears in any request or in browser storage.
- **The vault key is stored only under slot keys.** A device slot key needs the passkey's PRF output, which the authenticator releases only after user verification, and the PIN stretched by Argon2id. The kit slot needs the 256-bit kit secret. [Crypto spec 6.3](../crypto-spec.md#63-derivations-and-formats), [6.4](../crypto-spec.md#64-key-slots).
  Proven by: the vectors of M2-03, and the second implementation in Rust of M2-08, which must produce every vector byte for byte.
- **Guesses through the canister are counted.** A PIN try counts only after a proof made with the passkey alone. Three limits on wrong tries are kept on chain. [Crypto spec 6.4](../crypto-spec.md#64-key-slots), [spec 7.2](../../VAULT-PLAN.md#72-access-gates).
  Proven by: M1-05, where a wrong passkey proof is refused in the message hook and never counted, and a wrong PIN proof is counted.

## Party by party

### 1. The nodes that host the canister

Adversary A1.

- **Holds.** The canister state:
  - item ciphertexts and their wrapped data keys
  - the wrapped vault key of every slot
  - salts and public proof keys
  - counters and logs

  SEV-SNP encrypts memory and disk, but the claim does not rely on it.
- **Stopped by.** Nothing in the state decrypts without a slot key, see "What every row rests on".
- **Proven by.** M2-03, M2-08 and the canary of M3-08.
- **Where the claim stops.**
  - Together with the secret of a vault passkey, the state yields the PIN offline. R24 and RV1, question 1 of the brief.
  - A majority of the subnet's nodes can serve altered code, which the claim excludes. R1.
  - Seven nodes guard the try counter and the waiting period. RV7.

### 2. The network path, boundary nodes and gateways

Adversary A2.

- **Holds.** Everything in transit: signed requests including unlock proofs, ciphertext responses, the page itself, and who calls when.
- **Stopped by.**
  - Requests and responses carry ciphertext only.
  - Answers to update calls are certified by the subnet and checked in the browser against the root key of the Internet Computer.
  - An unlock proof names the caller and uses the current nonce of its key, so a recorded proof cannot be replayed. C38, C43, [spec 7.2](../../VAULT-PLAN.md#72-access-gates).
  - Item and label sizes are padded to buckets, [crypto spec 6.3](../crypto-spec.md#63-derivations-and-formats).
- **Proven by.** The canary of M3-08. The stale-nonce and replayed-call tests of M1-05.
- **Where the claim stops.**
  - One recorded unlock request together with the passkey secret yields the PIN offline, RV1. Adding the Internet Identity then opens the vault, see the pairs below.
  - Who calls, when and the padded sizes stay visible. R11.
  - Whoever can alter the page on its way, the gateway or someone posing as it, is excluded from the claim. R1 and RV6.

### 3. The user's Internet Identity, even when taken over

Adversary A4.

- **Holds.** The owner principal.
- **Stopped by.**
  - The owner principal is never a session. It may read the public slot descriptors, try to unlock, and begin adding a device with a handoff token. It can read or write no data. [Plan 3.4](../implementation-plan.md#34-who-may-call-what), [spec 7.2](../../VAULT-PLAN.md#72-access-gates).
  - An unlock of a passkey key needs a proof made with the passkey before a PIN try counts. So the owner principal alone can neither guess the PIN nor use up its tries. C39, RV21.
  - Adding a device needs a live handoff token that only a full session can create, and the new device must prove the vault key. C28.
- **Proven by.**
  - The authorization matrix test of M1-03, run as ingress and through a proxy canister that skips the message hook.
  - The hook tests of M1-05.
  - The handoff tests of M1-13.
- **Where the claim stops.**
  - The Internet Identity together with the kit opens the vault at once. The waiting period covers the kit alone. RV4, tested in M1-06.
  - Together with the passkey secret and one recorded unlock, it opens the vault after an offline PIN search. RV1.

### 4. The project maintainers and their release channel

Adversary A7.

- **Holds.** The source, the release tags and the build.
- **Stopped by.**
  - They control no vault. Every user runs their own canister, and in the first stage only the owner's identities are its controllers. [Spec 9.1](../../VAULT-PLAN.md#91-who-controls-a-vault-canister).
  - A release reaches a vault only when its owner installs it.
  - Builds are reproducible, and anyone can compare a canister's module hash with a local build of the tagged source. [Spec 9.2](../../VAULT-PLAN.md#92-reproducible-builds), [9.3](../../VAULT-PLAN.md#93-verification-by-anyone).
- **Proven by.**
  - M4-02: two clean builds give one hash, locally and in CI.
  - M4-07: `verify.sh` and `doctor.sh` report a healthy staging vault.
- **Where the claim stops.**
  - A release that the owner installs runs with full access at the next unlock. Its authors and every dependency of its build are among the five parties the claim excludes. R1.
  - A reproducible build shows that a binary matches its source, not that the source is harmless. The code is written with agents, R9, and nobody independent has read it, R26.

### 5. A stolen, locked device

Adversary A6.

- **Holds.** The authenticator with the vault passkey and the Internet Identity passkey, and the Internet Identity delegation in IndexedDB.
- **Stopped by.**
  - Both passkeys need the device's biometric or PIN, because user verification is required at every use. [Crypto spec 6.4](../crypto-spec.md#64-key-slots).
  - The vault PIN is still needed, with its limits.
  - Neither the vault key nor any plaintext is ever written to browser storage. [Crypto spec 6.7](../crypto-spec.md#67-what-is-deliberately-left-out), [spec 8.2](../../VAULT-PLAN.md#82-hardening).
  - The vault locks after 5 idle minutes, on sleep and when the tab closes, and the worker with its keys is gone after a lock. [Crypto spec 6.6](../crypto-spec.md#66-flows).
- **Proven by.**
  - The canary of M3-08 checks browser storage.
  - M3-05 covers the lock on idle and on sleep, and that the worker is gone afterwards.
  - M2-06 checks that no key is reachable after a lock.
  - The M3 scenario "five wrong PINs lock the key".
- **Where the claim stops.** A device that is unlocked, or runs malware, is outside the claim. R3 and A10.

### 6. A complete copy of the canister state

Adversary A9.

- **Holds.** The same bytes as row 1, without the running canister.
- **Stopped by.** The same slot keys as row 1. Without the canister there is no try counter. Everything then rests on the 256-bit passkey output and the 256-bit kit secret, which the PIN cannot replace.
- **Proven by.** The same tests as row 1.
- **Where the claim stops.**
  - Together with the passkey secret, the PIN is found offline. R24, question 1.
  - Together with the kit, the state is read.
  - A backup file is a state copy of the items. It needs the kit secret, and its MAC shows tampering. C30, tested in M6-02.

## Where the claim stops: pairs of thefts

The claim is about one theft at a time. These pairs are enough, as spec section 1 says:

| Together | Result | What limits it |
|---|---|---|
| Kit and Internet Identity | Opens at once | The kit is paper only, stored apart from devices and from backups. R5, R13, RV4 |
| Kit and a backup file | Reads the backup offline | Kit and backups are kept in different places, owner mistake U6 |
| Kit and a controller key, in the first stage | Reads the state | The controller key is password protected. Stage B removes the outside controller. R2 |
| Both passkeys and the PIN | Opens | The PIN rules refuse weak PINs, owner mistake U7. A hardware security key avoids a shared passkey provider, A5 |
| The secret of a vault passkey and a copy of the canister state | The PIN is found offline and the state is read | Nothing in the first release. The deferred network key would stop it. R24, question 2 |
| The secret of a vault passkey, one recorded unlock and the Internet Identity | The PIN is found offline, then the vault opens | Nothing in the first release. RV1, question 1 |

## What the claim does not cover

- **Altered code in the browser.** Five parties can serve it: the HTTP gateway, anyone who poses as it through a mis-issued certificate or a hijacked name, a majority of the subnet's nodes, whoever controls the canister in the first stage, and the authors of a release that the owner installs. Altered code sees the passkey output, the PIN and the kit at the next unlock. R1, R2 and RV6. Trust stage 3 needs a client that does not arrive through the gateway.
- **A compromised device while the vault is unlocked.** R3.
- **Mistakes of the owner.** The owner-mistake table of the [threat model](../threat-model.md#2-threat-model) lists U1 to U28 with their safeguards, and [spec 7.9](../../VAULT-PLAN.md#79-rules-that-protect-the-owner-from-mistakes) lists the rules the canister enforces. The tests are named after the mistakes, R17.

## Goals that are not about reading

- **Nobody else can change or upgrade a vault**, goal 2. Only the owner's identities control it, [spec 9.1](../../VAULT-PLAN.md#91-who-controls-a-vault-canister). The controller list is public, and `verify.sh` reads it, M4-07.
- **One understandable mistake must not lose the vault or open it to someone else**, goal 5. See the owner-mistake table and spec 7.9 above.
