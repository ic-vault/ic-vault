# Crypto spec

> Copied from section 6 of `VAULT-PLAN.md`, revision 8, by task M0-02 on 2026-11-09. From then on this file is the source of truth. It changes only through an ADR in `docs/adr/`.
>
> Section numbers are those of `VAULT-PLAN.md`. Sections 2, 6 and 11.4 to 11.6 live in `docs/threat-model.md`, `docs/crypto-spec.md` and `docs/operations.md`. Any other section number means `VAULT-PLAN.md`.

## 6. Cryptographic design

Rule zero: compose standard building blocks. Any change to this section needs an ADR and human approval.

### 6.1 Primitives

| Purpose | Primitive | Implementation |
|---|---|---|
| Authenticated encryption | AES-256-GCM, random 96-bit nonce, 128-bit tag | WebCrypto |
| Key derivation | HKDF-SHA-256 | WebCrypto |
| PIN and passphrase stretching | Argon2id v1.3, 64 MiB, 3 passes, 1 lane, 32-byte output, 16-byte salt. Parameters stored per slot | `@noble/hashes`. **VERIFY** speed at M2, switch to a Wasm build by ADR if a mid-range phone needs more than 2 seconds |
| Factor proof | Ed25519, strict verification | `@noble/curves` in the worker, `ed25519-dalek` in the canister |
| Hardware secret | WebAuthn PRF extension, 32-byte output | Browser and authenticator |
| Network key, deferred | vetKD on `bls12_381_g2` | Not in the first version, see 6.5 |
| Randomness | `crypto.getRandomValues` | Browser. The canister needs none |
| Backup integrity | HMAC-SHA-256 | WebCrypto |

Blob layout for every AES-GCM output: `nonce (12) || ciphertext || tag (16)`.

### 6.2 Keys and secrets

| Name | Size | Created | Lives | Purpose |
|---|---|---|---|---|
| Vault key, `VK` | 32 bytes random | Setup or re-key, in the worker | Worker memory while unlocked | Root of all data encryption |
| `k_wrap`, `k_meta` | AES-256 | Derived from `VK` | Worker, non-extractable | Wrap item data keys. Encrypt the labels of unlock keys |
| Data key, `DEK` | 32 bytes random | Every item save, every file | Wrapped next to the item | Encrypts one item version or one file |
| Factor secret, `F` | 32 bytes | Passkey PRF output, or kit factor | Authenticator. In memory only during unlock | Half of the slot key input |
| Stretched secret, `P` | 32 bytes | Argon2id of PIN or passphrase | In memory only during unlock | Other half of the slot key input |
| Slot key, `slot_kek` | AES-256 | Derived from `F` and `P` | Worker, during unlock | Inner wrap of `VK` |
| Slot proof key, `auth_seed` | Ed25519 | Derived from `F` and `P` | Worker, during unlock. Public half in the canister | Proves possession of the factor to the canister |
| Passkey proof key, `auth_f_seed` | Ed25519 | Derived from `F` alone, for slots with passkey and PIN | Worker, during unlock. Public half in the canister | Shows that the caller holds the passkey, before a PIN try is counted |
| Network key, `NK`, deferred | AES-256, derived from a vetKey | Not in the first version | Nowhere yet | A later outer wrap of device slots, see 6.5 |
| Session identity | ECDSA P-256 | Every unlock | UI thread memory, non-extractable | Signs all data calls |
| Kit secret | 32 bytes random | Setup, kit replacement, re-key | Paper only, as 24 words | Kit slot, break-glass identity, backup key |
| Kit check code | 5 bytes | Derived from the kit secret | Paper, and the encrypted settings | Proves the sheet can still be found without exposing the secret |
| Vault proof key | Ed25519 | Derived from `VK` | Worker while unlocked. Public half in the canister | Proves at enrolment that a device really holds the vault key |

**VERIFY** at M3: that the pinned SDK can create a non-extractable session identity.

### 6.3 Derivations and formats

```
lp(x)   = u32_be(len(x)) || x            length prefix for every variable-length field
VID     = vault id bytes: the id of the canister the vault was created in
ZERO32  = 32 zero bytes
u32, u64 = big endian, like every integer in this section
kind    = one byte: 1 passkey, 2 passkey_pin, 3 passphrase, 4 kit

# Slot keys
F          = passkey PRF output | kit_factor | ZERO32
P          = Argon2id(pin_or_passphrase, slot.kdf_salt, slot.kdf_params) | ZERO32
slot_kek   = HKDF(ikm = F || P, salt = slot.salt, info = lp("ic-vault/v1/slot-kek")  || lp(VID) || slot.id || kind)
auth_seed  = HKDF(ikm = F || P, salt = slot.salt, info = lp("ic-vault/v1/slot-auth") || lp(VID) || slot.id || kind)
auth_pub   = Ed25519 public key of auth_seed            stored in the canister

auth_f_seed = HKDF(ikm = F, salt = slot.salt, info = lp("ic-vault/v1/slot-auth-f") || lp(VID) || slot.id || kind)     slots with passkey and PIN only
auth_f_pub = Ed25519 public key of auth_f_seed          stored in the canister

# Passkey PRF input
prf_salt   = 32 random bytes per slot, stored with the slot

# Vault key and children
VK         = 32 random bytes
k_wrap     = HKDF(ikm = VK, salt = VID, info = "ic-vault/v1/dek-wrap")
k_meta     = HKDF(ikm = VK, salt = VID, info = "ic-vault/v1/meta")
label_ct   = AEAD(k_meta, pad(label), aad = lp("ic-vault/v1/label") || lp(VID) || slot.id)          label of an unlock key
vault_auth = HKDF(ikm = VK, salt = VID, info = "ic-vault/v1/vault-auth")                              Ed25519 seed, public key stored in the canister

# Slot blob
inner      = AEAD(slot_kek, VK, aad = lp("ic-vault/v1/slot-inner") || lp(VID) || slot.id || u32(vk_epoch))
wrap       = one byte stored with the slot. 1 is the inner wrap alone, which is all the first version knows
blob       = inner

# Items
pad(x)     = u32_be(len(x)) || x || zero bytes up to the bucket size
DEK        = 32 random bytes, new for every saved version
wrapped    = AEAD(k_wrap, DEK,       aad = lp("ic-vault/v1/dek")  || lp(VID) || item.id || u64(version))
header_ct  = AEAD(DEK, pad(header),  aad = lp("ic-vault/v1/item") || lp(VID) || item.id || u64(version) || 0x01)
body_ct    = AEAD(DEK, pad(body),    aad = lp("ic-vault/v1/item") || lp(VID) || item.id || u64(version) || 0x02)

# Files, after the first release. 1 MiB chunks, file DEK stored inside the item body
chunk_i    = AEAD(file_DEK, data_i,  aad = lp("ic-vault/v1/blob") || lp(VID) || file.id || u32(i) || last_flag)

# Proof of factor at unlock
msg        = lp("ic-vault/v1/unlock") || lp(VID) || lp(canister id) || lp(caller principal) || slot.id
             || u64(nonce of this slot) || lp(session_principal)
proof_f    = Ed25519-Sign(auth_f_seed, msg)             slots with passkey and PIN only, checked in the message hook
proof      = Ed25519-Sign(auth_seed, msg)               counted when wrong

# Settings hash, in the answer to a wrong proof
settings_hash = SHA-256(lp("ic-vault/v1/slot-settings") || lp(VID) || slot.id || kind
             || lp(credential_id) || lp(prf_salt)
             || lp(kdf_salt) || u32(memory_kib) || u32(passes) || u32(lanes) || slot.salt)
             a field the kind does not have is empty for lp and zero as a number

# Recovery kit
kit_secret = 32 random bytes, written on paper as 24 words
kit_factor = HKDF(ikm = kit_secret, salt = VID, info = "ic-vault/v1/kit-factor")      F of the kit slot
kit_id     = HKDF(ikm = kit_secret, salt = VID, info = "ic-vault/v1/kit-identity")    Ed25519 seed, principal R
kit_export = HKDF(ikm = kit_secret, salt = VID, info = "ic-vault/v1/kit-export")      key for backup files
kit_check  = first 5 bytes of HKDF(ikm = kit_secret, salt = VID, info = "ic-vault/v1/kit-check")   8 characters on the sheet
export_hdr = AEAD(kit_export, VK, aad = lp("ic-vault/v1/export") || lp(VID))          built with the kit, stored next to the kit slot, first record of a backup file

# Device handoff
K_h        = 32 random bytes, shown as a QR code or typed code on the unlocked device
token      = HKDF(ikm = K_h, salt = VID, info = "ic-vault/v1/enroll-token")           canister stores SHA-256(token)
handoff    = AEAD(HKDF(K_h, VID, "ic-vault/v1/handoff"), VK, aad = lp("ic-vault/v1/handoff") || lp(VID) || handoff.id)
enroll_sig = Ed25519-Sign(vault_auth, lp("ic-vault/v1/enroll") || lp(VID) || lp(canister id) || handoff.id
             || lp(session_principal))

# Backup file
backup_mac = HMAC-SHA-256(HKDF(ikm = VK, salt = VID, info = "ic-vault/v1/backup-mac"), every byte of the file before the last record)
```

- **Padding buckets.** Headers pad to a multiple of 256 bytes. Bodies pad to the next of 1, 2, 4, 8, 16, 32, 64, 128, 256 KiB. The label of an unlock key pads to 64 bytes, so it holds at most 60 bytes of text and its ciphertext always has 92 bytes. ADR 0012.
- **Plaintext encoding.** UTF-8 JSON. The header holds type, title, subtitle, tags and favourite flag. The body holds everything else, including password history, and later file keys. The label of an unlock key is the exception: plain UTF-8 text without JSON, ADR 0013.
- **Identifiers.** Slot, item, file and handoff ids are 16 random bytes generated in the browser.
- **Kit encoding.** The kit secret is 32 random bytes. The 24 words are the standard BIP39 encoding of those bytes with its 8-bit checksum, English list. Only the encoding is used. The BIP39 seed step with PBKDF2 is not. Each word is validated while typing, and the checksum at the end. The word list is vendored and a test pins its hash, so there is no new dependency. **DECIDE** (D9)
- **Check code.** Eight characters in Crockford base32. The app compares it with a copy in the encrypted settings. It reveals nothing useful about the kit secret.
- **Vault id.** Every key and every ciphertext is bound to the vault id, not to the canister that currently holds the vault. At creation the two are equal. The vault id is stored in the canister, never changes and is printed on the kit sheet. A vault can therefore move to another canister as ciphertext, see "Move a vault" in 6.6. Two things use the current canister id instead: the origin check in 8.2 and the unlock proof, which names both ids so it cannot be replayed to a copy. **DECIDE** (D16)
- **Files, after the first release, are immutable.** A changed file gets a new id, so a chunk needs no version.

### 6.4 Key slots

A slot is one way to open the vault. All slots wrap the same vault key.

| Kind | `F` | `P` | Proofs at unlock | Lockout after wrong tries |
|---|---|---|---|---|
| `passkey` | PRF output | none | One, checked in the message hook | no, 256-bit secret |
| `passkey_pin` | PRF output | PIN of 6 digits or more, or a passphrase | Two: the passkey alone in the hook, then passkey and PIN, counted | yes |
| `passphrase` | none | generated passphrase of 5 words or more | One, counted | yes |
| `kit` | kit factor | none | One, checked in the message hook | no, 256-bit secret |

The first release accepts `passkey_pin` and `kit` only. The canister refuses a slot of any other kind.

- **Protection profiles.** The first release has one: strict, which is `passkey_pin`. Standard, which is `passkey`, and fallback, which is `passphrase`, wait until after the first release and keep their numbers. This holds only if spike SP1 passes on every device you use. Decision D3.
- **Wrapping.** Every slot holds the vault key under its slot key and records which wrapping it uses. The second wrap of 6.5 is deferred.
- **Stored per slot in the canister:** id, kind, passkey credential id and PRF salt, Argon2id salt and parameters, slot salt, both proof keys, its own nonce, the wrap byte, blob, encrypted label, timestamps including last use, failed-try counter, locked-until time. The kit slot also carries the kit number and the export header.
- **Returned before unlock:** only id, kind, credential id, salts, parameters, nonce and the encrypted label, plus what the unlock screen must show early: kit number and date, a pending recovery, the cycles runway. Never a proof key, never the blob. The label is 92 bytes under a key that comes from the vault key, so it says nothing before an unlock, not even how long the name is. ADR 0013.
- **Answer to a wrong proof.** The tries that are left, and the settings hash of 6.3. The tries that are left are the number of wrong tries this slot still takes until it is locked, the one that locks it included. It is the smallest of what the three limits below leave, each counted with the try that is being answered: 5 minus the wrong tries in a row, 11 minus the wrong tries of the vault within thirty days, and 30 minus the count that nobody cleared. It is never below zero. Zero means that the slot is locked now, and that holds for the one try after a lock has run out too. ADR 0012. The hash covers the fields above except the nonce, so the client can tell that a node handed out wrong ones. It never covers a proof key, the wrap byte, the blob, the label, the nonce, a counter or a time. A hash over a proof key would let whoever holds the passkey test PINs offline after one counted try. ADR 0004.
- **One slot per passkey, not per device.** A synced passkey gives the same PRF output on every device that holds it, so those devices share one slot and need no enrolment of their own. The interface calls a slot an unlock key.
- **Creating a passkey.** Each new passkey gets a fresh random user id. When a device or a second unlock key is added, it also gets the credential ids of all existing slots as its exclude list. An authenticator that already holds a vault passkey then refuses a second one instead of overwriting the first. A re-key uses no exclude list, because the authenticator in your hand holds the passkey that is being replaced and would refuse the new one. The fresh user id keeps the old passkey intact until the re-key has succeeded. ADR 0008. The passkey is named "Vault unlock key" with the creation date and the words "do not delete". The relying party id is the full host of the vault and never a parent domain, and user verification is required at creation and at every use. A re-key retires every credential id. A retired passkey is never accepted again, and the interface names the old entries so that you can delete them.
- **PIN rules.** Entered twice. Refused: fewer than 6 digits, one repeated digit, ascending or descending runs, repeated pairs or triples, anything that reads as a date, and a built-in list of the most common PINs. The screen says not to reuse the PIN of the device or of a bank card.
- **Minimum stretching.** The client refuses Argon2id parameters below the minimum built into its release, whatever the canister says.
- **Passphrases**, after the first release, are always generated by the app, five words or more from a word list. A self-chosen passphrase is not accepted, because a passphrase key has no hardware secret behind it.
- **The kit slot exists exactly once.** It cannot be removed, only replaced by `kit_replace` or `rekey`.
- **Lockout.** Five wrong tries in a row lock the slot for 1 minute. After a lock expires there is one try, and a wrong one locks the slot again for twice as long, up to 24 hours. A successful unlock ends the row. A second limit ignores successes and counts all slots of the vault together: after ten wrong tries within thirty days, every further wrong try locks its slot for 24 hours. Without it, someone who holds both passkeys could slip four guesses in after each of your unlocks. A third limit ends the guessing: after 30 wrong tries that nobody cleared, the slot stays locked until a full session clears it. Every counted wrong try adds one to that count of its slot, and a successful unlock does not lower it. `slot_clear_lock` from a full session sets the count of one slot to zero and ends its lock. It leaves the limit of ten in thirty days alone. The unlock result carries the count of every slot. The app clears a slot by itself only when its count equals the wrong tries this page made on that slot and has not yet cleared. Otherwise a blocking alert names the slots with tries from elsewhere, at every unlock until it is answered: "They were mine" clears those slots, and "Not mine" clears nothing and leads to a re-key. ADR 0010.
- **VERIFY** at M3: some authenticators return the PRF output only at sign-in, not at creation. Setup then needs a second passkey prompt.
- **VERIFY** at M3: the capability name for PRF in `PublicKeyCredential.getClientCapabilities()`, which the device check of setup uses. A yes from the browser does not prove that the authenticator supports PRF, so the practice unlock stays the real test.

### 6.5 Network key, deferred

The first version has no network key. You decided this on 2026-11-09, after the design review, finding F6.

- **What it would be.** A key that a different subnet derives through vetKD for this canister. It would wrap every device slot a second time, so that a copy of the canister state plus the secret of a vault passkey is still not enough.
- **Why it waits.** It protects against that one pair of thefts and nothing else. If both passkeys sit with the same provider it protects nothing. It is also the most complex layer: a threshold call, transport keys, a key cache in the browser, epochs and three more methods. A first version that a reviewer can read in full is worth more.
- **What is given up.** A copy of the canister state together with the secret of a vault passkey reads the vault, because the PIN can then be found offline. See R24.
- **What is gained.** The canister calls no other canister, so every method is one atomic message. No device depends on a key service to unlock.
- **What stays ready.** Every slot records which wrapping it uses. The first version knows one, the inner wrap alone. A later release adds the outer wrap and re-wraps a slot at its next unlock.

### 6.6 Flows

**Setup, once per vault**

1. The deploy script creates the canister with the hash of a random claim code and prints a claim link with the code in the URL fragment. The code is 32 bytes from the random generator of the operating system, ADR 0012. The link is shown once and written to no file.
2. You open the link and sign in with Internet Identity. The client calls `claim(code)`. The caller becomes the owner and the claim code is erased. The screen asks you to bookmark the vault address. A device check then shows what this device can do: a passkey with PRF, persistent storage, a way to enter the kit. It names the protection profile that will be used, before anything is created.
3. The worker generates the vault key and the kit secret and builds the kit slot in memory.
4. The interface shows the kit sheet described in section 8.5. You print it or copy it by hand.
5. You enter all 24 words from the paper, typed or scanned. The worker discards its own copy of the kit secret, rebuilds it from your input and opens the kit slot with it. Setup does not continue until that works.
6. The interface creates the vault passkey and asks for a PIN according to the rules in 6.4.
7. The worker builds the device slot and the export header, and derives the kit principal `R`, the vault proof key and the check code.
8. `setup_finish` stores both slots, the export header and the vault proof key, adds `R` as an owner, and registers the first session. Setup cannot run a second time.
9. The app locks itself and asks for one practice unlock. Only after it succeeds does the vault accept items. The safety check then asks for a second unlock key.

The kit comes first because it is the step people skip. If setup is abandoned before step 8, nothing was stored. It starts again at step 3 with fresh keys, and the screen says that a sheet from the earlier attempt is void.

**Unlock**

1. Sign in with Internet Identity if there is no valid delegation.
2. `slots_public()` returns the slot descriptors with the nonce of each slot, the kit number, any pending recovery and the cycles runway. A pending recovery or a short runway is shown before anything else.
3. The passkey prompt returns `F`. The interface asks for the PIN if the slot needs one.
4. The worker derives `slot_kek` and the proof keys, and signs. A passkey slot with a PIN gets two proofs: one made with the passkey alone, one made with passkey and PIN. The interface generates a session identity.
5. `unlock` is called as the Internet Identity principal with the slot id, the nonce of that slot, the session principal and the proofs.
6. The message hook checks what costs nothing to check: the caller is an owner, the nonce is current, the slot is not locked, and the proof made with the passkey alone is right. A call that fails here is not counted and costs the vault nothing. Only someone who holds the passkey gets further.
7. The method repeats those checks. Then it writes a failed try, checks the proof that contains the PIN, and takes the failed try back if the proof is right. Because the try is written first, nothing that goes wrong on the way can make a wrong guess free. The nonce of the slot increases on every attempt that reaches this point.
8. On success the canister registers the session and returns the slot blob. The same answer carries the time of the previous unlock and every alert that was not acknowledged yet. They come from the whole subnet, not from one node.
9. The worker opens the slot blob with `slot_kek`. It derives `k_wrap`, `k_meta` and the vault proof key, and overwrites `F`, `P` and the slot keys.
10. The interface switches to the session identity, calls `sync`, and decrypts the headers.
11. Anything unusual since the last unlock blocks the screen until you acknowledge it, see section 8.5.

On a typical day that is one Internet Identity prompt in the morning, then one vault passkey prompt and the PIN for each unlock.

**Lock**

Idle for 5 minutes, device sleep, tab closed, or the lock button. Typing counts as activity. Before an idle lock the app stores an open edit as a separate encrypted draft and offers it again after the next unlock. Then the worker is terminated, the session identity is dropped, and `lock()` ends the session in the canister.

**Add a device**

A device that already holds the vault passkey through a synced keychain needs no enrolment. You sign in and unlock. The client tries this first. The steps below are for a device without the passkey.

1. On an unlocked device, choose "Add device". After a passkey prompt the worker creates `K_h` and uploads the handoff with a 10-minute expiry. The screen shows `K_h` as a QR code and as a typed code.
2. On the new device, sign in with the same Internet Identity and scan or type the code.
3. `enroll_begin(token, session principal)` checks the token, marks the handoff as used, returns it, and registers a session that may only finish the enrolment.
4. The new device decrypts the vault key, creates its own passkey and builds its slot. `enroll_finish` takes the slot together with a signature made with a key derived from the vault key. The canister checks both, stores the slot, deletes the handoff and promotes the session.

Without that signature, someone who held the Internet Identity and had seen the token could have gained a full session without ever knowing the vault key.

**Recover**

- **Forgotten PIN.** Open the vault with the kit or with another unlock key, then use "Change PIN" on a device that holds the passkey.
- **Lost device, Internet Identity still works.** Sign in, choose "Use recovery kit" and enter the 24 words. Two secrets are present, so the kit slot opens at once. Enrol the new device, then remove the lost one.
- **Lost Internet Identity.** Choose "Recover without Internet Identity" and enter the 24 words. The client signs in as the kit principal `R`. The first valid call starts a waiting period of 48 hours and is logged. **DECIDE** (D12) During that time every enrolled device shows a blocking warning at unlock with two ways out: cancel and replace the kit, or cancel and re-key. Cancelling alone is not offered, because the thief could start again at once. After it, `R` opens the kit slot and you link a new Internet Identity with `owner_set`.
- **Why the wait.** The kit alone is the one path where a single stolen, photographed or phished secret opens the vault. The wait gives an owner who still has a working device the time to cancel and re-key. An owner who has lost everything else waits two days, or reads a backup file offline at once.
- **Lost kit, device still works.** Replace the kit as described below.
- **Old kit in hand.** The recovery screen shows the number and date of the current kit, so an outdated sheet is recognised before any typing.
- **Lost everything.** The data is gone. This is by design and the setup screen says so.

**Remove a device**

The interface asks one question: could anyone else have unlocked the vault on that device? "No" removes the unlock key with `slot_remove(id)`. "Yes" or "not sure" removes it and starts a re-key. The confirmation names the key and its last use. A synced passkey is one unlock key for several devices, and the list says so. A lost device that shares a synced passkey keeps receiving that keychain, new passkeys included. The interface therefore tells you to remove the device from your Apple, Google or Microsoft account first.

**Replace the kit**

The interface first asks why. "I lost it" replaces the kit. "Someone may have seen it" starts a re-key instead, because a kit that was seen still opens every backup file made so far.

1. After a passkey prompt the worker creates a new kit secret. The interface shows the new sheet with the next kit number.
2. You enter the new words in full, and the worker opens the new kit slot with them.
3. `kit_replace` swaps the kit slot, the kit principal, the kit number and the export header in one call. Until then the old kit stays valid.
4. The screen confirms the new kit number. Only then destroy the old sheet.

**Kit check**

Thirty days after a kit is made, and then once a year, the safety check asks for the 8-character check code from the kit sheet. It proves that the sheet can be found and is the current one. It never asks for the words. If the sheet is gone, the same screen starts a kit replacement.

**Change a PIN**

In an unlocked session: a passkey prompt, then the new PIN twice. The worker builds a new slot under the same passkey, and `slot_add` replaces the old slot in the same call.

**Re-key**

A re-key locks out whoever holds an old key, from now on. It cannot take back what they could read before. The screen says so, and tells you to change the passwords that matter.

1. The interface lists the unlock keys that will stop working and the devices that must enrol again. A passkey prompt with the current key comes first.
2. You create a new vault passkey and choose a new PIN. The worker creates a new vault key, re-wraps every data key including trash and drafts, and builds a new slot and a new kit. The old passkey and the old PIN are not reused, because whoever made the re-key necessary may know them.
3. You print the new kit and enter its words in full.
4. `rekey` replaces everything in one atomic call, the vault proof key included. It fails if any item changed in between. The vault key epoch increases by one, and every other session ends.
5. The screen confirms the new kit number. Destroy the old sheet and delete backup files made before the re-key, because the old kit still opens them. All other unlock keys are gone, every old credential id is retired, and those devices enrol again with new passkeys.

**Backup, from M6**

A backup file is every stored record as ciphertext: items, settings, the kit slot, the vault id, the kit number and the vault key epoch. It also holds the export header: the vault key wrapped with the kit's export key. Device slots are left out, so that a backup file holds nothing a PIN could be tested against. The file ends with a MAC under a key derived from the vault key. The header is built when the kit is created and stored in the canister, so making a backup never needs the kit. The file opens offline with the kit alone, even if the canister, the network key or the origin no longer exist. Keep it apart from the kit. The safety check reminds you after 90 days without a backup. "Verify a backup" checks a chosen file record by record against the open vault.

**Move a vault, after the first release**

Deferred with the cut line. Until this flow exists, leaving a canister means reading a backup with the offline tool and entering the data again. The vault id is already in every format, so adding the flow later changes no format.

A vault moves when its subnet is retired, loses its confidential setup or is no longer trusted, or when the canister itself is lost. Nothing is decrypted on the way.

1. Make a backup file in the old vault if it still answers. Otherwise take the latest one.
2. Create a new canister with `deploy.sh` and open its claim link. Choose "Restore a vault" instead of setup. The screen shows the date of the file and of its newest change, because an older genuine backup passes every check.
3. The client uploads the records through two restore methods that come with this flow. The canister accepts both only from the owner and only before setup is complete. It takes over the vault id, the kit slot, the kit principal, the vault proof key and the vault key epoch from the file. It accepts exactly one kit slot, no device slot and no other owner. It records the restore in the alert log.
4. Unlock with Internet Identity and the kit. The client then checks the MAC of the file against the vault it has just opened, and tells you to discard the canister if the file was altered. Only after that check does it acknowledge the restore alert. Enrol your devices again, because passkeys belong to the old address.
5. Replace the kit, so that the new sheet shows the new address. If you moved because the old place is no longer trusted, re-key instead.
6. Only now retire the old canister: stop it with `retire.sh`, and delete it by hand after 30 days.

Anyone who holds a backup file can restore it into a canister of their own. Without the kit that gives them nothing they did not already have.

**Change the address**

This flow is an outline and waits until after the first release. It has a known gap: after step 3 the old address can no longer unlock, so every device has to be enrolled before that session ends. A complete design needs a second owner for a transition period. The flow is needed only if the gateway domain of the vault is ever retired. The canister and its data stay where they are. Passkeys and the Internet Identity principal belong to the old address and have to be replaced.

1. An upgrade names the new host as a second accepted host. Both addresses work during the change.
2. Open the vault at the new address and sign in. It answers "not the owner" and shows a code for this identity.
3. In the unlocked vault at the old address choose "Change address" and scan the code. `owner_set` makes the new identity the owner.
4. Still at the old address, use "Add device" for each device and enrol it at the new address.
5. Replace the kit, so that the sheet shows the new address. A later upgrade drops the old host.

If the old address is already gone, the kit is the way in: recovery without Internet Identity at the new address, with its waiting period.

### 6.7 What is deliberately left out

- No plaintext and no vault key in IndexedDB, localStorage or any cache.
- No offline unlock.
- No key export function anywhere in the interface.
- No server-side search, preview or thumbnailing.
- No "remember this device" that skips the passkey.
- No plaintext export from the web app. The command-line tool of M6 is the only way to write plaintext to disk.
- No reminders by email, push or any outside channel. The vault speaks only when it is open.
