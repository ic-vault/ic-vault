# ADR 0008: No exclude list when a re-key creates the new passkey

Status: Proposed. Needs the owner's signature, because it changes `docs/crypto-spec.md`
Date: 2026-11-10
Task: the review of plan and architecture of 2026-11-10, finding D6
Decided by: the owner

## Question

Spec 6.4 gives every new passkey "the credential ids of all existing slots as its exclude list". A re-key creates a new passkey on the device that holds the old one. Does the rule apply there?

## Facts, checked on 2026-11-10

1. Spec 6.4: "An authenticator that already holds a vault passkey then refuses a second one instead of overwriting the first." That is how an exclude list works in WebAuthn, and it is the purpose here, owner mistake U10.
2. Spec 6.6, "Re-key": step 1 asks for "a passkey prompt with the current key". Step 2: "You create a new vault passkey." The `rekey` call that retires the old credential ids comes in step 4. So at step 2 the old slot still exists and its credential id would be on the list.
3. The authenticator that just answered step 1 holds that credential. It refuses step 2.
4. Spec 6.4 also gives each passkey "a fresh random user id". An authenticator overwrites a passkey only when relying party and user id are both the same. A fresh user id already prevents the overwrite.

Not run. It follows from the WebAuthn rules, and task M5-05 has to show it with the virtual authenticator.

## Decision

Proposed: the exclude list is used when a device is added, and not at a re-key.

- **Add a device, and a second unlock key:** the new passkey gets the credential ids of all existing slots as its exclude list, as before.
- **Re-key:** the exclude list is empty. The fresh user id keeps the old passkey intact until `rekey` has succeeded. Afterwards the interface names the old entry so that you can delete it, as spec 6.4 already says.
- **Change a PIN** creates no passkey, so nothing changes there.

Signed: not yet.

## Consequences, once signed

- `docs/crypto-spec.md` 6.4: the bullet "Creating a passkey".
- `docs/implementation-plan.md`: the proofs of M3-02 and M5-05.
- Tests. With the virtual authenticator: a re-key completes on the authenticator that holds the old passkey, and the old passkey still works if the re-key is abandoned before the call.
