# ADR 0004: What the settings hash covers

Status: Accepted. Signed by the owner on 2026-11-10
Date: 2026-11-10
Task: the review of plan and architecture of 2026-11-10, finding D1
Decided by: the owner

## Question

A wrong PIN try is answered with `BadProof { tries_left; settings_hash }`. Which fields of a slot go into `settings_hash`?

## Facts, checked on 2026-11-10

1. The only definition is a comment in section 3.2 of the plan: "hash of the slot settings the canister holds". Spec 7.2 and correction C47 say what it is for: the app compares it with the settings it used, and notices when a single node had handed out wrong ones. `docs/crypto-spec.md` does not mention it.
2. A stored slot holds two proof keys, spec 6.4. `auth_pub` is derived from the passkey secret `F` and the stretched PIN `P` together, spec 6.3.
3. `BadProof` reaches whoever passes the message hook: an owner principal with a right `proof_f`. That is someone who holds the Internet Identity and the vault passkey, adversary A5.

If the hash covered `auth_pub`, that person could make one wrong try, keep the hash, and then test every PIN offline: derive `auth_pub` for a candidate PIN, hash, compare. The counter on chain would see one try. A 6-digit PIN falls in about a million Argon2id runs. That would break the promise of the strict profile against A5.

Nothing says the hash covers `auth_pub`. Nothing says it does not. An implementing agent should not have to guess.

## Options

| | What the hash covers | Result |
|---|---|---|
| A | Exactly the fields that `slots_public` hands out before unlock and that go into the key derivation | Does its job. Tells the caller nothing new |
| B | The whole stored slot | Simplest to write. Leaks a PIN test, see above |
| C | Drop the hash | Nothing to get wrong. The app can no longer tell a lying node from a wrong PIN, review item RV8 |

## Decision

Chosen: option A, defined in spec 6.3.

```
settings_hash = SHA-256( lp("ic-vault/v1/slot-settings") || lp(VID) || slot.id || kind
                         || lp(credential_id) || lp(prf_salt)
                         || lp(kdf_salt) || u32(memory_kib) || u32(passes) || u32(lanes)
                         || slot.salt )
```

- A field that the slot kind does not have is an empty string for `lp` and zero for a number.
- The hash never covers a proof key, the wrap byte, the blob, the label, the nonce, a counter or a time.
- The worker computes the same value from the `SlotInfo` it used, through a new message `settings.hash`, and the app compares the two.

Signed: Manuel Rüedi, the owner, on 2026-11-10, in the chat with the agent that wrote this record. The answer, in the owner's words: "Drop 0011 and 0012 for now. Rest is ok". The agent recorded it.

## Consequences

- `docs/crypto-spec.md`: the definition in 6.3 and one bullet in 6.4.
- `docs/implementation-plan.md`: the comment in 3.2, one message in 3.5, the `slot-keys.json` row in 5.2, and the proofs of M1-05 and M3-05.
- `VAULT-PLAN.md` 7.2: the bullet about the hash points to the definition.
- Tests. A fixed value in `slot-keys.json`. In the canister: the hash stays the same when `auth_pub`, `auth_f_pub` or the blob of the slot changes, and changes when any covered field changes.
