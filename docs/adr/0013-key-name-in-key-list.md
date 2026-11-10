# ADR 0013: How the name of an unlock key is set and read

Status: Accepted. Decided by the owner on 2026-11-10
Date: 2026-11-10
Task: the open point of ADR 0012, found by its fresh review. The review of plan and architecture of 2026-11-10, finding P4
Decided by: the owner. It changes sections 3.2, 3.5 and 3.7 of `docs/implementation-plan.md` before task M1-01. It changes neither `docs/crypto-spec.md` nor section 7 of `VAULT-PLAN.md`

## Question

Every unlock key stores an encrypted name, its label. Section 3 has no way to set that name and no way to read it back. How does the app do both?

## Facts, checked on 2026-11-10

1. Spec 6.3 defines `label_ct`, and ADR 0012 fixes its length at 92 bytes. `Slot` in section 3.2 has the field `label`.
2. No worker message of section 3.5 takes the text of a name. `setup.build`, `slot.build` and `rekey.build` return a `Slot`, or arguments that hold one, without any input for it. No message opens a label.
3. No method of section 3.2 gives the label of a device key back. `SlotInfo`, the entry of `slots_public`, has none. Only `BackupManifest` carries the kit slot with its label.
4. The owner needs the name. Threat model U19: a device is sold or lost, and the "list of unlock keys with label and last use" is what the owner decides from. The proof of M5-02 says "Label and last use shown".
5. `slots_public` is a query, open to the owner principal before an unlock. Section 3.2 calls it a hint, because one node answers it. The same list already carries `created` and `last_used`.
6. The label is encrypted with `k_meta`, a child of the vault key, and bound to its key by the associated data `lp("ic-vault/v1/label") || lp(VID) || slot.id`. It always has 92 bytes. Whoever holds the Internet Identity and not the vault key learns nothing from it, not even its length.
7. The settings hash of ADR 0004 never covers the label. Spec 6.4 says so already.
8. Section 3.5: "Three kinds of plaintext cross to the UI thread". A name is typed by a person and shown to a person, so it is a fourth.
9. Section 3.7: "All plaintext is UTF-8 JSON with a format number". A name of at most 60 bytes has no room for that.

## Options

| | Rule | Good | Bad |
|---|---|---|---|
| A | `SlotInfo` gets the field `label`. The app reads it with the list it already reads | One field. No new method, so no new row in the matrix of 3.4 | The ciphertext is also handed out before an unlock, to the owner principal |
| B | A new method for full sessions returns the labels | Only an unlocked session gets the ciphertext | One more method to gate and to test, for 92 bytes that say nothing |
| C | No names in the first release | Least to build | Two phones cannot be told apart when one has to be removed, U19 |

## Decision

Chosen: A.

**In the interface**

- `SlotInfo` gets `label : blob`, the 92 bytes as stored. `slots_public` returns it for every key.
- No method renames a key. A label is written with its key and never changes. So a node cannot show an older name of the same key, and the label is as bound to its key as a body is to its item.
- `unlock.prove` and `settings.hash` take a `SlotInfo` and ignore its label.

**In the worker**

- `setup.build`, `slot.build` and `rekey.build` take the name of the device key as one more input, as plain text.
- The worker refuses a name of more than 60 bytes of UTF-8. An empty name is allowed.
- The label of a kit slot is always the empty text. The app has its own words for the kit.
- A new message `labels.open` takes entries of `SlotInfo` and returns the name of each, or for each label that does not open a mark that says so. It works only while the worker holds the vault key.
- The names of unlock keys are a fourth kind of plaintext that crosses to the UI thread.

**In the plaintext formats**

- A name is plain UTF-8 text without JSON and without a format number. The length prefix of `pad` is its only frame.

**In the web app**

- The screens that create a device key ask for its name: setup, adding a device, and a re-key. A change of PIN builds a new key and passes the old name on.
- The list of unlock keys shows the names after an unlock. A label that does not open is shown as unreadable. It is not hidden, and it is not a reason to remove the key.
- Names are shown as text, never as markup, rule 8.

The owner decided this on 2026-11-10, in the chat with the agent that wrote this record, on a card with the three options above. His choice, as the card recorded it: Chose "Add the name to the key list" on the decision card asking "Choose how the app saves and shows the name of an unlock key". The agent recorded it.

## What goes beyond the card

The card asked only where the name is read from. These details were added by the agent. The pull request names them.

- No rename in the first release.
- The kit slot has the empty name, and a device key may have one.
- The name is plain text and not JSON.

## What the spike shows of this

`spikes/thin-slice/` follows this record. `SlotInfo` has the label, and the test `thin_slice_end_to_end` reads the label of the device key from `slots_public`, opens it with `k_meta` and gets the name back. A label taken from another key does not open. The spike has no worker, so the messages of 3.5 are not shown.

## Consequences

- `docs/implementation-plan.md`:
  - 3.2: `SlotInfo` gets `label`. The paragraph "Which answers can be trusted" names the label.
  - 3.5: one more input for `setup.build`, `slot.build` and `rekey.build`, the new message `labels.open`, and four kinds of plaintext.
  - 3.7: the name of an unlock key as plain text.
  - Proofs of M1-04, M2-03, M3-04, M5-01 and M5-02.
  - M1-01 depends on SP9 and no longer on an open point.
- ADR 0012 was merged with this point listed as open. It is an accepted record and is not rewritten. This record is the answer it asks for.
- The Candid interface changes by one field of one record. This is before the freeze of M1-01, so no deployed canister is touched.
