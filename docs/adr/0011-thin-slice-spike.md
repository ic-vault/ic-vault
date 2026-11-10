# ADR 0011: Spike SP9, one thin path before the freeze

Status: Accepted, by the merge of the pull request that carries it
Date: 2026-11-10
Task: the review of plan and architecture of 2026-11-10, finding P4. Spike SP9
Decided by: the pull request that merges it. It changes section 2 of the plan and one dependency in section 4. The four open points below are the owner's. ADR 0012 decides them

## Question

Task M1-01 freezes section 3 of the plan. The plan says of that section that it has never been compiled. Does it compile, and does one path through it run: claim, setup, unlock, save one item, read it back?

## What was built

Throwaway code under `spikes/thin-slice/`, run on 2026-11-10 with the pinned Rust 1.99.1.

- `vault.did`: the Candid text of section 3.2, copied unchanged, with ADR 0003, 0007 and 0010 in it.
- A Rust canister with every type and all 29 methods of 3.2. Twelve methods are real, seventeen are stubs. State is on the heap.
- A client in plain Rust with the derivations of spec 6.3 that the path needs, and a test that walks the path on PocketIC.

## Results

| Check | Result |
|---|---|
| The Candid text of 3.2 parses and type-checks | Yes, once `HttpRequest` and `HttpResponse` are written out. 3.2 leaves them to a library |
| A Rust canister exports exactly that interface | Yes. `service_equal` of `candid_parser` compares the two in a test |
| The path runs on PocketIC | Yes. Claim, setup with a device slot and a kit slot, a wrong PIN, the right PIN, one item saved, synced and read back, lock |
| The message hook does what spec 7.2 says | Yes, as far as tested. A stale nonce, a wrong passkey proof, a stranger, a second claim, the owner principal on a session method and a call after `lock` never reach a method |
| A wrong passkey proof counts nothing | Yes. The nonce of the slot does not move |
| A wrong PIN is counted and answered with the settings hash of ADR 0004 | Yes. Canister and client compute the same 32 bytes |
| The count of ADR 0010 survives a success | Yes. `uncleared` shows one try after the good unlock, and `slot_clear_lock` resets it |
| The three install refusals of ADR 0003 | Yes |
| A ciphertext does not open under another version | Yes |

Three facts for ADR 0001, from the same build:

- **`getrandom`.** With `ic-cdk` 0.20.3, `candid` 0.10.39, `sha2` 0.11.1 and `ed25519-dalek` 3.0.0 without its default features, no `getrandom` is in the Wasm dependency tree. This answers the open check for this set of crates. M0-05 repeats it for the real workspace.
- **`ed25519-dalek` 3.0.0** has `verify_strict`, `VerifyingKey::from_bytes` and `is_weak`, which is what spec 6.1 and correction C43 need.
- **Size.** The Wasm is 892 KB without any web assets and without `ic-wasm`.

## Gaps the spike found in section 3

None of them stopped the path. Each needs one sentence before the freeze. They are listed here and not decided here, because each is a choice. ADR 0012 decides them and carries the edits.

| # | Gap | Proposal |
|---|---|---|
| 1 | The label of an unlock key is `AEAD(k_meta, pad(label), ...)`, spec 6.3. The padding buckets are defined for headers and bodies only. The label has none. The spike used a fixed blob | One bucket of 64 bytes for labels, in spec 6.3 and in `limits.json` |
| 2 | `tries_left` in `BadProof` is not defined. Is it what is left of the five in a row? What does it say once the limit of ten in thirty days applies, or after a lock? The spike returned five minus the row | Define it as the tries left before the next lock of this key, whichever limit comes first, and zero when the key is now locked |
| 3 | A sixth session. The limit is five, and nothing says what happens then. The spike refuses with `LimitReached` | End the session with the oldest activity instead. A refusal locks the owner out for up to 30 minutes after five tabs |
| 4 | `claim` with a wrong code reaches the method and costs cycles. The hook can compare the hash at no cost, as it does for the token of `enroll_begin` | The hook checks the claim code |

Two observations that need no decision:

- Spec 6.3 uses two styles for the `info` of HKDF. Slot keys use `lp(label) || lp(VID) || ...` with the slot salt. The children of the vault key and of the kit use the bare label with the vault id as salt. Both are sound. The vectors of 5.2 pin them, and the Rust reference of M2-08 has to follow each exactly.
- `renew`, `sync`, `lock`, `backup_manifest` and the queries return plain values, so a refusal inside the method can only be a trap. That fits the rule "refusals without a reason" in 3.2. The client must treat a trap on these as "no session".

## What the spike does not show

- Nothing about the browser: no worker, no SDK, no passkey, no Internet Identity. Spikes SP1, SP6 and SP8 cover those.
- No stable memory, no upgrade, no certified assets.
- The waiting period, the kit principal, enrolment, re-key and the logs are stubs.
- The client is Rust written by the same agent as the canister. A shared misreading of the spec would pass. The vectors and the second implementation of M2-08 are the real check.
- PocketIC 16.1.0 was downloaded by the test from GitHub without a pinned hash. Its release date was not checked against the 7-day rule. The product tasks pin it.

## Decision

- Spike SP9 gets a row in section 2 and is done.
- M1-01 depends on SP9. The four gaps above are answered by ADR 0012, in the same pull request.
- `spikes/thin-slice/` stays in the repository until M1-05 has its own tests, then it is deleted.

## Development dependencies of the spike

All exact, all at least 7 days old on 2026-11-10. None ships in the product through this spike.

| Crate | Version | Released |
|---|---|---|
| `ic-cdk` | 0.20.3 | 2026-09-15 |
| `candid` | 0.10.39 | 2026-10-21 |
| `ed25519-dalek` | 3.0.0 | 2026-07-06 |
| `sha2` | 0.11.1 | 2026-10-12 |
| `serde` | 1.0.229 | 2026-07-18 |
| `serde_bytes` | 0.11.19 | 2025-09-15 |
| `candid_parser` | 0.4.1 | 2026-08-14 |
| `pocket-ic` | 16.1.0 | 2026-10-08 |
| `hkdf` | 0.13.0 | 2026-03-30 |
| `aes-gcm` | 0.11.1 | 2026-08-21 |
| `argon2` | 0.6.0 | 2026-08-27 |

The transitive crates in `Cargo.lock` were not checked one by one against the 7-day rule.

## Consequences

- `docs/implementation-plan.md`: a row SP9 in section 2, and SP9 in the "Depends" column of M1-01.
- The spike was changed once more to follow ADR 0012. The results above are from the first run and still hold, except that a wrong claim code and a sixth session now behave as ADR 0012 says.
- `spikes/thin-slice/` with its own Cargo workspace and lock file. The root of the repository gets no `Cargo.toml` from this. That stays with M0-05.
