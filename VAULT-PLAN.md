# Personal Vault on the Internet Computer: Full Plan

Working name: **Vault**. Repository: `ic-vault`. Protocol label: `ic-vault/v1`.
Status: plan only, nothing is built. Written 2026-11-09.
Revision 2, same day: owner mistakes assessed and safeguards added. See "Owner mistakes" in section 2, the flows in 6.6, and sections 7.9, 8.5 and 9.6.
Revision 3, same day: setup and upkeep checked against the current docs and assessed. See sections 11.4 to 11.6, "Move a vault" and "Change the address" in 6.6, and decisions D8 and D16 to D18.
Revision 4, same day: gaps found while writing `IMPLEMENTATION-PLAN.md`, and by two independent reviews, are corrected. Section 3.10 of that document lists them.
Revision 5, same day: the findings of the design review in `DESIGN-REVIEW-FINDINGS.md` are applied, and the network key is deferred. Section 3.10 of `IMPLEMENTATION-PLAN.md` lists these changes as C37 to C47.
Revision 6, same day: the cut line. The first release leaves out files, import, two kinds of unlock key, moving a vault, the change of address, the quota setting and the launcher. See "Scope of the first release" in section 1.
Revision 7, same day: no paid audit, because there is no budget for one. A release gate takes the place of the audit, and trust grows in three stages. See "Trust stages" in section 1, "The release gate" in section 12, and risk R26.
Revision 8, same day: the choices before the first push are made. See decisions D1, D2 and D21.

## 0. How to use this document

- **Audience.** You, and the AI agents that will build it. Agents treat sections 6, 7, 8, 9.6, 11.5, 11.6 and 12 as the specification. Exact types, encodings and the order of tasks are in `IMPLEMENTATION-PLAN.md`.

  > Since task M0-02, read `docs/implementation-plan.md` wherever this document names `IMPLEMENTATION-PLAN.md`. The root copy is frozen.
- **DECIDE** marks a choice that is yours. Each has a default so work can start without you.
- **VERIFY** marks a fact an agent must confirm against current documentation before relying on it.
- At bootstrap, copy section 2 into `docs/threat-model.md`, section 6 into `docs/crypto-spec.md` and sections 11.4 to 11.6 into `docs/operations.md`. From then on the repository copies are the source of truth and change only through an ADR.
- This is security software written by agents. Section 15 lists what stays risky. Do not put real secrets into it before the release gate in section 14, and then only by the trust stages of section 1.

## 1. Goals and the security claim

**Goals**

1. A personal vault for passwords, notes, cards and identity documents. Small files follow after the first release.
2. Open source and self-sovereign. Every user runs their own vault canister. Nobody else can read it, change it or upgrade it.
3. Reading raw data requires defeating several independent locks that sit with different parties.
4. Small enough for agents to build and for one reviewer to read in full: one canister, one web app, standard primitives only, no home-made cryptography.
5. Forgiving. One understandable mistake by the owner must not lose the vault or open it to someone else.

**The claim the design must uphold**

Whoever can read or copy what one of the following holds, but cannot change the code your browser runs, cannot read vault contents:

- the nodes that host the canister
- the network path, boundary nodes and gateways
- the user's Internet Identity, even when someone has taken it over completely
- the project maintainers and their release channel
- a stolen, locked device
- a complete copy of the canister state

The claim is about reading. It does not cover a party that can change the web app on its way to your browser, because altered code sees the passkey secret, the PIN and the kit at the next unlock. Five parties can do that: the HTTP gateway, anyone who can pose as it through a mis-issued certificate or a hijacked name, a majority of the nodes of the vault's subnet, whoever controls the canister in stage A, and the authors of a release that you install, together with every dependency of its build. Reproducible builds show that a binary matches its source. They do not show that the source is harmless. This is the largest risk the design accepts, see R1 and decision D19.

The claim is about one theft at a time. These pairs are enough:

| Together | Result |
|---|---|
| Kit and Internet Identity | Opens at once |
| Kit and a backup file | Reads the backup offline |
| Kit and a controller key, in stage A | Reads the state |
| Both passkeys and the PIN | Opens |
| The secret of a vault passkey and a copy of the canister state | The PIN is found offline and the state is read. The deferred network key would stop this, see 6.5 |
| The secret of a vault passkey, one recorded unlock and the Internet Identity | The PIN is found offline, then the vault opens |

**What still defeats it**

- Altered code served to your browser, as said above.
- A compromised device while the vault is unlocked.
- Theft of the recovery kit. A thief without the Internet Identity must also sit out a waiting period, see section 6.6.
- An attacker who holds both an owner sign-in and a vault factor.
- Some mistakes of the owner. They are more likely than any attacker, so section 2 lists them and the design treats them like an adversary.

**Non-goals for v1**

- Protection on a device that already runs malware.
- Hiding that a vault exists, or hiding traffic timing.
- Sharing between users, family plans, teams. Planned later with vetKeys identity-based encryption.
- Browser autofill extension and native mobile apps.
- A hosted multi-user service.
- Offline unlock.
- Any AI or assistant feature. It would require plaintext to leave the client.

**Scope of the first release**

You took the cut line on 2026-11-09. A reviewer then has less to read, and nothing in the right column weakens the claim.

| In the first release | Waits until after the first release |
|---|---|
| Logins, notes, cards and identity documents | Files |
| One kind of unlock key: passkey plus PIN | Unlock keys without a PIN, and without a passkey |
| The recovery kit, its check and its replacement | Import from other password managers |
| Adding a device by code | A QR decoder library. Scanning works where the browser can do it alone |
| Recovery, with the waiting period | Moving a vault to another canister |
| Re-key, trash, alerts, the safety check | The flow for a change of address |
| Backup files and the offline tool | Changing the quota |
| Staging, production, watchdog and upkeep | The launcher for people without a command line |
| | The network key, deferred earlier, see 6.5 |

One condition: spike SP1 has to pass on every device you use. A device without PRF cannot unlock a vault that has no passphrase key. If SP1 fails anywhere, the passphrase key comes back before the freeze.

**Trust stages**

Nobody independent has reviewed this design, and there is no budget for an audit. So trust grows in stages. You decided this on 2026-11-09, decisions D10 and D19.

| Stage | What goes in | Opens when |
|---|---|---|
| 1 | Test data | A staging vault exists |
| 2 | Secrets you can rotate, such as passwords. Your current password manager stays the master copy | The release gate of section 12 has passed |
| 3 | What you cannot rotate, such as identity documents and seed phrases | An independent person has reviewed the design, and a client exists that does not arrive through the gateway |

Stage 3 may never open. The vault is then a place for secrets you can replace.

## 2. Threat model

> Since task M0-02 the source of truth for this section is `docs/threat-model.md`. The text here is the copy of revision 8 and is not updated.

**Assets, in order of value**

1. Item contents: passwords, notes, and later files.
2. Keys: the vault key and anything that unwraps it.
3. Metadata: titles, item count, sizes, access times.
4. Availability: the vault must not be lost or locked forever.

**Adversaries**

| # | Adversary | What they hold | What still stops them |
|---|---|---|---|
| A1 | Node provider on the vault subnet | The physical machines | SEV-SNP encrypts memory and disk. If that fails they hold ciphertext, wrapped slots and verifier keys. They still need a vault factor. A majority of the subnet's nodes acting together could also serve altered code, see R1. |
| A2 | Boundary node or HTTP gateway operator | The TLS endpoints | Only ciphertext and metadata pass through. Responses to update calls are verified in the browser against the IC root key. They can tamper with page delivery: see R1. |
| A3 | Colluding nodes of a key subnet | Nothing, as long as the network key is deferred | Applies again when the network key returns, see 6.5. |
| A4 | Internet Identity takeover: phished recovery phrase, linked Google account, governance-level compromise | The owner principal | They see slot descriptors only. Unlock needs a signature from a vault factor. They cannot spend tries on a passkey slot, because a counted try needs a proof made with the passkey. A passphrase slot has no such proof and can be kept locked. The kit slot never locks. |
| A5 | Passkey provider takeover, for example a synced keychain | The Internet Identity passkey and the vault passkey | Strict profile: a PIN with five tries in a row and ten in thirty days across all slots, counted on chain. Standard profile: not protected. Use a hardware security key or the strict profile. |
| A6 | Thief with a locked device | Hardware | Both passkeys need a biometric or the device PIN. |
| A7 | Project maintainers, malicious or coerced | The release channel | They cannot touch deployed vaults. A bad release has to be installed by the user. Builds are reproducible. |
| A8 | Thief of the deployer key, stage A only | The canister controller | They can replace the code: see R2. Stored data stays encrypted. A planted interface could capture factors at the next unlock. |
| A9 | Thief of a state copy or a backup file | Bytes | Backup files need the kit secret. State copies are case A1. |
| A10 | Malware or script injection on an unlocked device | Everything on screen | Out of scope. Section 8 lowers the likelihood. |
| A11 | Phisher with a look-alike site | A fake page | Passkeys are bound to the real origin. Kit and passphrase can be phished, so the interface asks for the kit words only at kit creation and on the recovery screens, and a kit used alone has to wait. |
| A12 | Future quantum attacker with recorded traffic | A break of public-key cryptography | All data and slot encryption is symmetric with 256-bit keys. Public-key parts guard access, not data. |
| A13 | Bad luck: lost devices, lost identity, canister out of cycles | Nothing | Recovery kit, backup files, long freezing threshold, runway warning. |
| A14 | The owner, by mistake | Every key and every command | The safeguards in the table below. |

**Owner mistakes**

Nobody attacks in this table. The owner does something understandable and loses data or exposes it. For a personal vault that is more likely than any adversary above.

- **High** means plausible for a careful person, and the damage is total or permanent.
- **Medium** means plausible but limited or recoverable, or total but unlikely.
- **Low** means unlikely, or minor.

"Before" rates revision 1 of this plan. "After" rates it with the safeguards in the last column. Both are estimates, to be corrected by the hallway test in section 12.

| # | Mistake | Cost | Before | After | Safeguard |
|---|---|---|---|---|---|
| | **Recovery kit** | | | | |
| U1 | Kit kept as a screenshot, photo, PDF or note instead of paper | The master key sits in a cloud account | High | Medium | Paper-only wording on the sheet and in setup. Kit-only recovery waits 48 hours and can be cancelled. Advice only beyond that, see R13. |
| U2 | Kit lost, or not found on the day it is needed | Total loss once the last device fails | High | Low | Full entry at setup proves it exists. Kit check after 30 days, then yearly. Two copies in two places. |
| U3 | Kit copied or typed with an error | The kit fails on the day it is needed | High | Low | 24 words with a checksum, each word validated while typing. Setup opens the kit slot with the entered words before any data exists. |
| U4 | Old kit kept, new kit thrown away | Recovery fails | Medium | Low | Kit number and date on the sheet and on the recovery screen. The old kit stays valid until the new one has been entered in full. |
| U5 | Kit words typed into a fake page | Total compromise | Medium | Medium | The words are requested only at kit creation and on the recovery screens. The yearly check uses a separate check code. The vault address is on the sheet. Waiting period when the Internet Identity is missing. |
| U6 | Kit stored next to a device or a backup file | One theft gets everything | Medium | Medium | Advice on the sheet and in the safety check. Nothing can enforce it. |
| | **Setup and unlock factors** | | | | |
| U7 | Weak PIN, or the PIN of the phone or bank card reused | Whoever holds both passkeys guesses it | Medium | Low | Entered twice. Repeats, runs, dates and common PINs are refused. Warning against reuse. |
| U8 | PIN forgotten | The unlock key locks | Medium | Low | Remaining tries and wait time are shown. The kit or another unlock key opens the vault, then "Change PIN". |
| U9 | Standard profile with passkeys synced to one cloud account | That account holds both gates | Medium | Low | The first release has the strict profile only. Later the app reads the backup flag of the passkey and warns. |
| U10 | Vault passkey deleted in a clean-up, or overwritten by enrolling twice | That way of unlocking is gone | Medium | Low | Clear passkey name with date. Fresh user id and exclude list at creation. The safety check asks for a second way in. |
| U11 | Signed in with a different Internet Identity | "Not the owner", which looks like a lost vault | Medium | Low | The error screen explains it and offers to switch identity. Hint line on the kit sheet. |
| U12 | One device and no other unlock key | One loss away from depending on the kit | Medium | Low | The safety check stays amber until a second unlock key exists or the owner confirms that the passkey is synced. |
| U13 | Real secrets stored in a test vault or a build that did not pass the release gate | Secrets guarded by a test key or by unchecked code | Medium | Low | Fixed banner driven by the environment name and a build flag. |
| | **Daily use** | | | | |
| U14 | Item deleted or overwritten by mistake | Permanent loss of that item | High | Low | Trash for 30 days. Field history inside the item. A sync conflict is saved as a copy. |
| U15 | Vault left open, or edits lost when it locks itself | Exposure, lost work | Medium | Low | Auto-lock on idle, sleep and tab close. An open edit is saved as an encrypted draft first. |
| U16 | Password pasted into the wrong place, or kept by a clipboard history | Exposure of that password | Medium | Medium | Clipboard cleared after 30 seconds where the browser allows it. See R14. |
| U17 | Warnings ignored | A break-in goes unnoticed | Medium | Low | A new unlock key, an owner change, foreign failed tries or a pending recovery block the screen until acknowledged. Routine information stays quiet. |
| U18 | Wrong address typed, or a look-alike vault opened | Phishing | Medium | Low | Bookmark step at setup. Address on the kit sheet. Passkeys work only on the real origin. |
| | **Devices** | | | | |
| U19 | Device sold, lost or given away while still enrolled | Depends on whether someone could unlock it | Medium | Low | List of unlock keys with label and last use. One plain question decides between remove and re-key. |
| U20 | Wrong unlock key removed, or a needless re-key | Own devices locked out, a new kit to handle | Low | Low | The confirmation names the key and its last use. Re-key lists what it disconnects. The kit slot cannot be removed. |
| | **Upkeep** | | | | |
| U21 | No backup, an old backup, or one never tested | Loss if the canister or the subnet fails | Medium | Low | Reminder after 90 days. "Verify a backup". Making a backup never needs the kit. |
| U22 | Plaintext import file left on disk | Exposure of everything imported | Medium | Low | Import waits until after the first release. Then the app offers to delete the file where the browser allows it, and reminds otherwise. |
| U23 | Cycles run out unnoticed | The vault freezes and is later deleted | Medium | Low | Generous first funding. Runway on the unlock screen, notice at 365 days, urgent at 90. Top-up needs no unlock. |
| | **Operating the canister in stage A** | | | | |
| U24 | Canister deleted or reinstalled, or the wrong environment targeted | All data gone | High | Low | Scripts without deletion or reinstall mode. Typed confirmation. Separate identities. Snapshot and backup before every upgrade. |
| U25 | Wrong subnet, wrong environment name or no freezing threshold at creation | A weak setup that cannot be changed | Medium | Low | Checks run before the claim link is printed. `doctor.sh`. |
| U26 | Controller identity lost, for example when WSL is reset | No more upgrades | Medium | Low | Second controller on paper. Migration through backup and restore. |
| U27 | Unverified build installed | Broken or malicious code | Medium | Low | The upgrade script installs only the hash of a signed release that staging already runs. |
| U28 | Things that cannot be replaced stored before stage 3 | A flaw that nobody has looked for exposes what cannot be rotated | Medium | Medium | The statement at setup has to be acknowledged, and the safety check repeats it. Nothing can enforce it. |

**What the assessment shows**

- The recovery kit carries most of the risk. It is a paper secret that is never used, so it is easy to lose, easy to copy wrongly and tempting to photograph. Three of the five High ratings are kit mistakes, and three kit mistakes stay Medium after the safeguards.
- The other two High ratings are actions that cannot be undone: deleting an item, and deleting or reinstalling the canister in stage A.
- More cryptography fixes none of this. The fixes are safe defaults, proof at creation, scheduled checks, undo and waiting periods. They are specified in sections 6.4, 6.6, 7.9, 8.5 and 9.6 and tested in section 12.
- Five risks stay Medium because nothing but the care of the owner prevents them: U1, U5, U6, U16 and U28.

## 3. Decisions

| Topic | Decision | Why |
|---|---|---|
| Tenancy | One canister per user | The user is the only controller. No shared target. The developer cannot upgrade anyone. |
| Packaging | One Rust canister serves the API and the embedded web app | One Wasm hash covers every line of code the user runs. |
| Subnet | Confidential subnet `re2t4-faa75-v3vhk-kdmdr-uyrkl-aik2l-ixd6u-p3fyr-zlfkc-6c5af-zae` | Hides ciphertext and metadata from node providers. Chosen at creation, cannot change later. **DECIDE** |
| Gate 1: sign-in | Internet Identity | Proven, origin-bound, no password database. |
| Gate 2: vault factor | A dedicated vault passkey using the WebAuthn PRF extension, plus a PIN. Other kinds of unlock key come after the first release | Independent of Internet Identity. Its secret never leaves the authenticator unprotected. |
| Factor check | The canister verifies an Ed25519 signature made with a key derived from the factor | A try counter and lockout enforced by consensus. No custom WebAuthn verification on chain. |
| Sessions | Unlock registers a fresh session identity. All data calls are made as that identity | Uses the IC's own request signatures. No custom signing envelopes. |
| Encryption | Client side only. AES-256-GCM, HKDF-SHA-256, Argon2id, WebCrypto keys marked non-extractable, inside a worker | Standard, reviewable, quantum-tolerant at rest. |
| Key structure | Random vault key, one key slot per factor, one data key per item version | Familiar disk-encryption pattern. Adding or removing a device never re-encrypts data. |
| Network key | Deferred until after the first release. Every slot records its wrapping, so it can be added later | It protects one narrow case and is the most complex layer. Your decision of 2026-11-09, see 6.5. |
| Recovery | One paper kit holding a 256-bit secret as 24 words | It is a key slot, a break-glass identity and the backup key in one. Used without the Internet Identity it opens the vault only after a waiting period. |
| Owner mistakes | Safe defaults, proof at creation, scheduled checks, undo and waiting periods for every irreversible step | The owner is the most likely cause of loss. See section 2. |
| Portability | The vault id is separate from the canister id | A vault can move to another canister or subnet as ciphertext. Nothing is decrypted on the way. **DECIDE** |
| Setup | Rehearsed on a staging vault that is configured like production, then repeated for the real one | The first run of a procedure should never be the one that counts. See section 11.5. |
| Upkeep | A daily watchdog that holds no key, a monthly agent task, a yearly drill | The vault cannot call for help itself. See section 11.6. |
| Storage | `ic-stable-structures` only | Survives upgrades without migration code. |
| Frontend | Vite, React, TypeScript, minimal runtime dependencies | Agents write it well. Small supply chain. |
| Tooling | `icp-cli` at the exact version pinned at M0, PocketIC, pnpm, Cargo, WSL on Windows | Current official toolchain. It moves fast, so the pin is deliberate. |
| Agent skills | Pin the official ICP skills into the repo | They carry the current API names and the known pitfalls. |
| Code hosting | Public GitHub repository from the first commit, owned by a GitHub organization named after the project. Optional Entire mirror | Cursor and Claude cloud agents can only clone from GitHub. The public call for review needs a public repository. The kit sheet points at this address for years, and an organization can change hands without changing it. |
| License | Apache-2.0 | Patent grant, matches the ecosystem. **DECIDE** |
| Telemetry | None. No third-party requests of any kind | A vault must not phone home. |

## 4. How the IC strengths are used

| IC capability | Use in the vault |
|---|---|
| Confidential subnet with SEV-SNP | Canister memory and disk are encrypted against node providers. |
| vetKeys | Not in the first version. Later: the network key for device slots, sharing by identity-based encryption, timelocked inheritance. |
| Internet Identity | Gate 1. Passkey sign-in with a principal that exists only for this vault's origin. |
| Logic enforced by consensus | Try counter, lockout, session expiry, upload limits, the waiting period for kit-only recovery. A node operator cannot reset a counter or shorten the wait. |
| Tamper-proof state | An audit log of unlocks and device changes that nobody can edit, including the user. |
| Certified assets, public module hash, public controller list | Anyone can check which code a vault runs and who can change it. |
| Canister-served frontend | No separate web host, no CDN, no DNS of your own to hijack. |
| User-controlled canister | The vault outlives the project. The user can leave with their data at any time. |
| Timers | Not used in the first version. Expiry is worked off when the owner unlocks. Later: inheritance after inactivity. |
| Cycles model | The user funds their own vault. No account, no subscription, no billing data. |
| Snapshots | A restore point before every upgrade. |

## 5. Architecture

```
 Browser, canonical origin https://<canister-id>.icp.net
 +--------------------------------------------------------------+
 | UI thread   screens, Internet Identity client, passkey       |
 |             prompts, two IC agents (owner and session)       |
 |      | ciphertext only               ^ plaintext on demand   |
 | Crypto worker   vault key, slot keys, encrypt and decrypt    |
 +------+-------------------------------------------------------+
        | HTTPS. Page load is certified. API calls are signed
        | by the caller and responses are verified in the browser.
 -------+--------- API boundary nodes and HTTP gateway ----------
        v
 Vault canister, one per user, confidential subnet
   http        certified web app and security headers
   access      owner check, factor signature, session table
   storage     slots, items: ciphertext only. Audit log.
   vetkd       deferred until after the first release, see 6.5

 Internet Identity at https://id.ai gives the owner principal.
```

Three things live in the browser only and never reach the canister unwrapped: the vault key, the factor secrets, and all plaintext.

## 6. Cryptographic design

> Since task M0-02 the source of truth for this section is `docs/crypto-spec.md`. The text here is the copy of revision 8 and is not updated.

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

- **Padding buckets.** Headers pad to a multiple of 256 bytes. Bodies pad to the next of 1, 2, 4, 8, 16, 32, 64, 128, 256 KiB.
- **Plaintext encoding.** UTF-8 JSON. The header holds type, title, subtitle, tags and favourite flag. The body holds everything else, including password history, and later file keys.
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
- **Returned before unlock:** only id, kind, credential id, salts, parameters and nonce, plus what the unlock screen must show early: kit number and date, a pending recovery, the cycles runway. Never a proof key, never the blob.
- **One slot per passkey, not per device.** A synced passkey gives the same PRF output on every device that holds it, so those devices share one slot and need no enrolment of their own. The interface calls a slot an unlock key.
- **Creating a passkey.** Each new passkey gets a fresh random user id, and the credential ids of all existing slots as its exclude list. An authenticator that already holds a vault passkey then refuses a second one instead of overwriting the first. The passkey is named "Vault unlock key" with the creation date and the words "do not delete". The relying party id is the full host of the vault and never a parent domain, and user verification is required at creation and at every use. A re-key retires every credential id. A retired passkey is never accepted again, and the interface names the old entries so that you can delete them.
- **PIN rules.** Entered twice. Refused: fewer than 6 digits, one repeated digit, ascending or descending runs, repeated pairs or triples, anything that reads as a date, and a built-in list of the most common PINs. The screen says not to reuse the PIN of the device or of a bank card.
- **Minimum stretching.** The client refuses Argon2id parameters below the minimum built into its release, whatever the canister says.
- **Passphrases**, after the first release, are always generated by the app, five words or more from a word list. A self-chosen passphrase is not accepted, because a passphrase key has no hardware secret behind it.
- **The kit slot exists exactly once.** It cannot be removed, only replaced by `kit_replace` or `rekey`.
- **Lockout.** Five wrong tries in a row lock the slot for 1 minute. After a lock expires there is one try, and a wrong one locks the slot again for twice as long, up to 24 hours. A successful unlock ends the row. A second limit ignores successes and counts all slots of the vault together: after ten wrong tries within thirty days, every further wrong try locks its slot for 24 hours. Without it, someone who holds both passkeys could slip four guesses in after each of your unlocks. A third limit ends the guessing: after 30 wrong tries that nobody cleared, the slot stays locked until a full session clears it. A full session can clear a lock, and the alert about tries that were not yours says when to do so.
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

1. The deploy script creates the canister with the hash of a random claim code and prints a claim link with the code in the URL fragment. The link is shown once and written to no file.
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

## 7. Vault canister

### 7.1 State

All state lives in stable memory through `ic-stable-structures`, one memory per map.

| Memory | Content |
|---|---|
| 0 | Config cell: owners, setup flag, claim hash, vault key epoch, limits, sequence number, kit number, recovery delay, pending recovery, vault id, environment name, accepted hosts, vault proof key, retired credential ids |
| 1 | Slots, and the export header |
| 2 | Item records: id, version, sequence, updated time, deleted flag and time, wrapped data key, header ciphertext |
| 3 | Item bodies |
| 4 | Reserved for file chunks, after the first release |
| 5 | Activity log, the last 10,000 events |
| 6 | Handoffs and enrolment tokens |
| 7 | Change index: change number to item id, for `sync` |
| 8 | Expiry index: what has to be removed, and when. It is worked off at unlock and at sync |
| 9 | Alert log: what an owner must see, kept until acknowledged |

Sessions live on the heap. An upgrade ends all sessions, which is the safe default.

### 7.2 Access gates

Every method starts with its gate. The message hook, `canister_inspect_message`, repeats the checks that cost nothing, so that a refused call costs the vault no cycles. It is never the only check, because a call from another canister skips it.

What the hook does for `unlock`:

- **It refuses a stale nonce and a locked slot.** Each slot has its own nonce.
- **It checks every proof that rests on 256 bits.** For a passkey slot that is the proof made with the passkey alone. For the kit slot it is the kit proof. A wrong one is refused and not counted. **VERIFY** at M1 that the check fits the instruction limit of the hook.
- **It never checks a proof that contains a PIN or a passphrase.** Every such guess has to reach consensus and be counted. Checking it early would hand out unlimited uncounted guesses.
- **It limits the kit principal.** The kit principal may try the kit slot and no other. While a recovery is pending and not yet ready, the hook refuses the kit principal altogether.

What the hook does elsewhere:

- For `enroll_begin` it compares the token with the pending handoff. The token has 256 bits, so this leaks nothing.
- It refuses every ingress call to a query method. Called as an update, `status` or `http_request` would cost the vault cycles.
- A call that the hook turns away comes back without an error value. The client reads `slots_public` again and decides from that.

What the method does for `unlock`, after repeating the hook:

- **It writes the failed try first**, then checks the proof, then takes the try back on success. If anything traps in between, the whole call is undone and the caller learns nothing.
- **It names the caller.** The signed message contains the caller principal, so a proof made for one owner cannot be submitted by another.
- **It answers a wrong proof with a hash of the slot settings.** The client compares it with what it used, and can tell the owner when a node had handed out wrong settings. Spec 6.3 defines the hash. It covers only what `slots_public` hands out and never a proof key, ADR 0004.

| Calls | Caller must be | Extra proof |
|---|---|---|
| `http_request`, `status` | anyone | none. Static certified assets, plus version, environment name, vault id, kit number, a coarse health value and whether a recovery is pending |
| `claim` | any non-anonymous principal, once | the claim code |
| `setup_finish` | owner, before setup is complete | none |
| `slots_public` | owner | none |
| `unlock` | owner. The kit principal for the kit slot only | For a passkey slot with a PIN two Ed25519 signatures: one from the passkey alone, one from passkey and PIN. For the kit principal also an expired waiting period |
| `enroll_begin` | Internet Identity owner | one-time token from the handoff |
| `enroll_finish` | enrolment session | a signature made with a key derived from the vault key |
| every read and write of items, slots, settings, logs | live session principal | the IC request signature |

Owners are the Internet Identity principal and the kit principal `R`. The anonymous principal is rejected everywhere except the first row.

**The waiting period.** When the caller is the kit principal, a valid `unlock` does not open the kit slot at once. The first valid call records a pending recovery that becomes ready after the recovery delay, logs it and returns the ready time. A valid call after that time opens the slot. Any full session can end a pending recovery with `recovery_cancel`. The Internet Identity principal opens the kit slot without waiting, because it brings a second secret. A pending recovery belongs to one kit number. It is used up by the first unlock after it became ready, lapses 7 days after that time, and is cleared by `recovery_cancel`, `kit_replace` and `rekey`.

**A passphrase slot** has no proof that rests on 256 bits. Anyone who holds an owner principal can spend its tries. That is one more reason why the first release has no passphrase slots.

### 7.3 Interface sketch

```candid
service : (InitArgs) -> {
  // open
  http_request  : (HttpRequest) -> (HttpResponse) query;
  status        : () -> (Status) query;

  // owner principal
  claim         : (blob) -> (Result);
  setup_finish  : (SetupArgs) -> (Result);
  slots_public  : () -> (SlotsPublic) query;
  unlock        : (UnlockArgs) -> (UnlockResult);
  enroll_begin  : (EnrollArgs) -> (EnrollResult);

  // session principal
  enroll_finish : (Slot, blob) -> (Result);
  renew         : () -> (nat64);
  lock          : () -> ();
  sync          : (nat64) -> (SyncPage);
  item_put      : (ItemPut) -> (ItemPutResult);
  item_delete   : (blob, nat64) -> (Result);
  item_restore  : (blob, nat64) -> (Result);
  item_purge    : (blob) -> (Result);
  item_body     : (blob) -> (opt Body) query;
  slot_add      : (Slot, opt blob) -> (Result);
  slot_remove   : (blob) -> (Result);
  slot_clear_lock : (blob) -> (Result);
  handoff_put   : (Handoff) -> (Result);
  kit_replace   : (KitReplace) -> (Result);
  recovery_cancel : () -> (Result);
  backup_manifest : () -> (BackupManifest);
  owner_set     : (principal) -> (Result);
  rekey         : (RekeyArgs) -> (Result);
  alerts_ack    : (nat64) -> (Result);
  limits_set    : (nat64) -> (Result);
  audit         : (nat64, nat32) -> (vec AuditEvent) query;
  trust         : () -> (TrustInfo) query;
}
```

- **Writes use optimistic concurrency.** `item_put` carries the expected version. A mismatch returns the current version and the client resolves it.
- **`sync` is an update call.** Its response is certified by the subnet, so the list of items and versions cannot be forged or rolled back by one node.
- **Bodies are queries.** They are fast, and each is bound to its item id and version by the AEAD associated data. A stale or swapped answer fails to decrypt.
- **Deleting moves an item to the trash.** `item_restore` brings it back within 30 days. After that the ciphertext is erased at the next unlock, and `item_purge` erases it at once. A tombstone stays until day 90 so other devices learn about the deletion. Trash counts toward the quota.
- **`slot_add` can replace.** Given the id of an existing slot it swaps the two in one call. That is how a PIN is changed. `slot_remove` rejects the kit slot.
- **`owner_set` changes the Internet Identity principal only.** It rejects the anonymous principal, the kit principal and the id of the canister itself. The kit principal changes only through `kit_replace` and `rekey`.
- **`status` is all the outside world can ask.** It returns the release version, the environment name, the vault id, the number and date of the current kit, and a health value: `ok`, `low` below 365 days until freezing, `urgent` below 90. It also says whether a recovery with the kit alone is pending, without a time, so that the watchdog can warn you inside the waiting period, ADR 0007. No balance and no sizes. The watchdog of section 11.6 reads it.
- **Restore waits until after the first release.** Two methods will then load a backup file into a fresh canister as ciphertext, in place of setup.
- **Decisions hang on update calls only.** A query is answered by one node. Alerts and the previous unlock come with the unlock result, the item list comes from `sync`, and a backup takes its kit slot and export header from `backup_manifest`. `slots_public`, `audit` and `trust` are hints: a wrong answer makes an unlock fail or shows stale numbers, and nothing else.
- **Settings are a reserved item.** They are versioned and synced like any other item, so there is no separate settings call.
- **`alerts_ack` and `limits_set`.** The first stores which alerts were acknowledged. The second changes the waiting period. Changing the quota waits until after the first release.
- **Checks on slots.** `slot_add` and `enroll_finish` refuse the kit kind, refuse the kit slot as the one to replace, refuse a retired credential id, and refuse a slot whose fields do not fit its kind. They also refuse a proof key that is not a valid point of full order.
- **Sessions end with their key.** `slot_remove` ends every session that was opened through that slot. `rekey` ends every session except its own.
- The exact types are in section 3.2 of `IMPLEMENTATION-PLAN.md`.

### 7.4 Limits

| Limit | Value |
|---|---|
| Header, padded plaintext | At most 3,840 bytes. The ciphertext is 28 bytes longer |
| Body, padded plaintext | At most 256 KiB. The ciphertext is 28 bytes longer |
| File size, after the first release | 50 MiB, in 1 MiB chunks |
| Vault quota | 1 GiB, fixed in the first release |
| Slots | 16 |
| Items, including trash and drafts | 10,000, so that a re-key fits into one call |
| Sync page | 500 records or 1.5 MiB |
| Concurrent sessions | 5 |
| Session lifetime | 30 minutes, renewed while active, 8 hours at most |
| Wrong tries on a PIN or passphrase slot | 5 in a row, then locked for 1 minute, doubling up to 24 hours. Ten within thirty days across all slots, then 24 hours for each further one. After 30 that nobody cleared, locked until a full session clears it |
| Handoff lifetime | 10 minutes, single use |
| Trash retention | 30 days, then erased |
| Waiting period for the kit principal | 48 hours, adjustable by the owner from 24 hours to 30 days |

### 7.5 Audit log

There are two logs. The alert log holds what an owner must see: slot added, slot removed, kit slot opened, kit replaced, recovery started, recovery cancelled, owner changed, re-key, enrolment finished, upgrade applied. It keeps each entry until it is acknowledged, so noise cannot push anything out of it. The activity log holds the last 10,000 other events: claimed, setup finished, unlock succeeded, unlock failed, slot locked, handoff created, quota or rate limit hit. Each event has a time, the caller principal and the slot id where one applies. The interface shows the previous successful unlock on every unlock.

### 7.6 Serving the web app

- The frontend build is embedded in the Wasm and certified with `ic-asset-certification` 4 and `ic-http-certification` 4. The router is rebuilt and the certified data is set in both `init` and `post_upgrade`.
- `http_request` never upgrades to an update call. Asset serving costs the vault no cycles.
- The canister serves nothing under `/.well-known/ii-alternative-origins`. That file would let other origins share the Internet Identity principal of this vault. A test proves that the path answers not found.
- The release Wasm is identical for every vault. At install the canister builds a certified `config.json` with its own id, the vault id and the accepted hosts, so the app knows where it is supposed to run. The web app uses hash routes, so one certified page serves every screen. The host suffix and the environment name are install arguments: `icp.net` on mainnet, the local network host in development. On a local network the canister also takes the root key of that network and serves it in `config.json`. The app uses it only on a loopback host, ADR 0003. A second accepted host for a change of address waits until after the first release.
- Response headers, all certified:

| Header | Value |
|---|---|
| Content-Security-Policy | `default-src 'none'; script-src 'self' 'wasm-unsafe-eval'; style-src 'self'; img-src 'self' data: blob:; font-src 'self'; connect-src 'self' https://icp-api.io; worker-src 'self'; frame-ancestors 'none'; base-uri 'none'; form-action 'none'; require-trusted-types-for 'script'; trusted-types vault-worker` |
| Strict-Transport-Security | `max-age=63072000; includeSubDomains` |
| X-Content-Type-Options | `nosniff` |
| Referrer-Policy | `no-referrer` |
| Cross-Origin-Opener-Policy | `same-origin-allow-popups`, because the Internet Identity popup needs its opener |
| Cross-Origin-Resource-Policy | `same-origin` |
| Permissions-Policy | For the vault itself only: camera for QR scanning, creating and using passkeys, writing to the clipboard. Everything else off |

- The current docs name `https://icp-api.io` as the API host. **VERIFY** at M0 that React, the SDK and the worker run under this policy.
- Starting the worker is a Trusted Types sink. One named policy, `vault-worker`, returns the fixed worker URL and nothing else. The final headers are in place from the M0 hello page on, so a header that breaks the app is found at once.
- Requests on a raw domain get a redirect to the canonical origin. This helps honest clients only, because the gateway never verifies a raw domain. On any other gateway domain the same certified page is served, and the app itself refuses to run there, see 8.2. A response that depends on the host cannot be certified with plain asset certification.

### 7.7 Cycles safety

- Freezing threshold of 180 days, declared in the deployment config. The platform default is 30 days. A frozen vault keeps its data and stops accepting calls until topped up. Once the cycles are gone the code and the data are deleted.
- `trust()` returns the balance and the estimated runway until freezing. `slots_public` returns the runway too, so the unlock screen warns before unlocking: a notice below 365 days, urgent below 90.
- Topping up needs no unlock and no controller rights. The Trust screen and the kit sheet both say how.
- The settings guide says a frozen canister does not execute messages, and reports on the developer forum show query calls being rejected as well. The page itself then stops loading when the vault freezes, and the kit sheet and `docs/runbooks/top-up.md` are the only guidance left. Both must cover this case. **VERIFY** at M4 on the local network by raising the freezing threshold above the balance.
- Every input has a size limit. Every collection has a count limit. No loop runs over unbounded data.
- The first version calls no other canister. No method waits for an answer from elsewhere, so every method is one atomic message and costs about the same small amount.
- A stolen Internet Identity can cause only a bounded number of counted wrong tries. Everything else it sends is rejected before consensus.
- Another canister can call the vault without passing the message hook, and the vault pays for refusing each call. Anyone who knows the address can therefore burn its cycles, at a lower cost to themselves. Every method refuses in its first statement, the funding is generous and the watchdog reports a falling balance. See R22.
- The burn rate is measured from balance samples taken at unlock, at most once a day.

### 7.8 Upgrade safety

- No logic in `pre_upgrade`. Everything durable is already in stable memory.
- The config record carries a schema version. Migrations run in `post_upgrade` and are tested from every released version.
- `post_upgrade` refuses to start on data whose schema version is newer than the code. There are no downgrades. The way back is the snapshot.
- No migration loops over all items inside `post_upgrade`. A change of record format is applied when a record is next written.
- The upgrade test runs against a vault filled to its limits and records the instruction count, so growth cannot make an upgrade fail one day.
- Mainnet installs always name their mode: install at creation, upgrade ever after. The automatic mode of `icp deploy` is never used there.
- The first version has no `await`. When one is added, the caller is read before it.

### 7.9 Rules that protect the owner from mistakes

The canister enforces these. The interface cannot skip them, and neither can a modified client.

- There is always exactly one kit slot. It can be replaced, never removed.
- `kit_replace` and `rekey` are single atomic calls. A dropped connection leaves either the old state or the new one, and the kit number in `slots_public` tells which.
- A deleted item stays restorable for 30 days, unless the owner deletes it forever in a separate step. An overwritten item is not covered: its earlier values live in the field history inside the item.
- The kit principal waits out the recovery delay, and every enrolled device can cancel.
- The kit principal can try the kit slot and nothing else. A try on a passkey slot is counted only for a caller who has shown the passkey.
- `claim` runs once. Then `setup_finish` runs, once.
- `owner_set` can never remove the kit principal, so a wrong new owner is repaired with the kit.
- After setup, nothing the owner principal can do without a vault factor changes or deletes data.
- A full session is only ever held by someone who can decrypt the vault key. An unlock proves a factor, and an enrolment proves the vault key itself.

## 8. Web client

### 8.1 Structure and boundaries

- `crypto/` runs in a dedicated worker. It is the only place that handles vault key material, and the only place that imports noble. `platform/` uses WebCrypto only through the SDK, for the Internet Identity delegation and the session identity.
- `platform/` wraps Internet Identity, passkeys and the two IC agents. Its canister functions accept only values of the branded type `Ciphertext`.
- `vault/` holds the session state machine: locked, unlocking, unlocked, locking.
- `ui/` renders. It receives the decrypted headers for the list and the body of the one item on screen, and nothing else.
- A lint rule enforces these import boundaries. A branded `Plaintext` type cannot be passed to `platform/` without a compile error.
- Factor secrets pass through the UI thread only at the moment of entry. They are transferred to the worker and the source buffer is zeroed.
- The worker is the one place for key handling that a reviewer has to read. It does not stop a script in the page from asking it to use the keys.
- Three requests to the worker are as good as a key export: a new kit, a code for adding a device, and a re-key. The worker answers them only after a fresh passkey answer for the current slot: it compares the passkey proof key it derives from the answer with the one it kept from the unlock, ADR 0005. A session that was opened with the kit is exempt, because the kit was entered minutes before.

### 8.2 Hardening

- No inline scripts, no `eval`, no `innerHTML`. Trusted Types enforced.
- No requests to any origin except the vault itself, the IC API host and the Internet Identity popup. No fonts, analytics, error reporters or CDNs.
- No service worker in v1. It adds a persistent code cache that a single bad page load could poison.
- On start the app compares `location.host` with the canonical host built from its canister id and refuses to continue on any other origin.
- The browser stores only what the Internet Identity client keeps for the sign-in: its credential storage in IndexedDB and its record of who is signed in. Spike SP6 lists the exact entries, ADR 0006.
- Every sign-in passes `maxTimeToLive` of 8 hours, decision D7. The pinned client has no idle logout. The vault has its own auto-lock.
- No bundle calls `fetchRootKey`, and a bundle test checks this. The only root key other than the built-in one comes from the certified `config.json` of a local network. The app accepts it only on a loopback host with the environment `local`, and stops when it finds one anywhere else, ADR 0003.
- The setup and recovery screens ask for a browser profile without extensions. An extension that may read every page can read the kit. The self-hosting guide recommends such a profile for daily use too.

### 8.3 Interface rules that are security rules

- The kit words are requested in two places only: when a kit is created and on the recovery screens. Both say that no other screen will ever ask for them. The kit check asks for the check code, never for the words. A completed word is masked.
- Secrets are masked by default. Copy clears the clipboard after 30 seconds where the browser allows it.
- Every unlock shows the time of the previous unlock and the number of failed tries since. Unusual events block the screen until acknowledged, see 8.5.
- Passkeys are created and used with user verification required, and with the full host of the vault as relying party id.
- Setup cannot be finished without entering the kit in full and one practice unlock.

### 8.4 Dependency policy

Runtime allowlist: `react`, `react-dom`, `@icp-sdk/core`, `@icp-sdk/auth`, `@noble/hashes`, `@noble/curves`, one small QR generator. Anything else needs an ADR.

- Exact versions, committed lockfile, `pnpm install --frozen-lockfile --ignore-scripts`.
- `pnpm audit` and `cargo deny` run in the gate.
- `minimumReleaseAge` is set to 7 days in `pnpm-workspace.yaml`, so a version published yesterday cannot enter the lockfile. An exception for a security fix goes through `minimumReleaseAgeExclude` and an ADR.
- Cargo has no such setting. The monthly update task applies the same 7-day rule by hand.
- Rust runtime crates: `ic-cdk` 0.20, `ic-stable-structures`, `ic-asset-certification` 4, `ic-http-certification` 4, `candid` 0.10, `ed25519-dalek`, `sha2`, `serde`.
- **VERIFY** at M0 that every pin above is still current, and that any crate pulling `getrandom` has the Wasm workaround that the official skills describe.

### 8.5 Safeguards against owner mistakes

Five principles, applied to every screen and every script:

1. **The safe choice is the default.** Strict profile, paper kit.
2. **Recovery material is proven when it is made and checked on a schedule.** Not on the day it is needed.
3. **Nothing irreversible happens in one step.** A destructive action can be undone for 30 days, or is prepared and confirmed before one atomic call, or waits out a delay.
4. **Plain questions instead of expert words.** The interface asks what happened and chooses between remove, replace and re-key itself.
5. **No silent failure.** Every error screen says what happened, whether the data is safe, and what to do next.

**The kit sheet** carries everything recovery needs, so nothing has to be remembered:

- the vault address as text and as QR code, and the vault id
- kit number, creation date and time
- the 24 numbered words, and the same as a QR code
- the 8-character check code
- a blank line "Internet Identity used:" to fill in by hand
- short instructions: paper only, no photo and no file. Enter the words only at the address above or into the offline recovery tool. Keep two copies in two places, none next to a device or a backup file. Where to find the recovery tool and the top-up guide. What to do when the page does not load.

A print stylesheet prints the sheet and nothing else. The setup text recommends copying the words by hand when the printer is shared or networked, and warns against "Save as PDF".

**The safety check** is one screen, and one coloured mark on the home screen. It is green when all four rows hold.

| Check | Green when | Otherwise |
|---|---|---|
| Recovery kit | Check code confirmed within the last 12 months. The first check is due 30 days after the kit was made | Ask for the check code, or replace the kit. Thirty days overdue, it becomes a blocking alert |
| Second way in | Two unlock keys besides the kit, or the owner confirmed that the passkey is synced to another device | Offer "Add device" or a hardware security key |
| Backup, from M6 | A backup within the last 90 days | Offer "Make a backup" |
| Cycles | More than 365 days until freezing | Show the top-up command with the canister id filled in |

The check code and the dates of the last kit check and the last backup live in the encrypted settings. The vault cannot reach an owner who does not open it. Reminders therefore appear after unlock. The two that cannot wait, a pending recovery and a short cycles runway, appear on the unlock screen.

**Alerts that block.** After unlock the screen stops until each of these is acknowledged: an unlock key added or removed, the owner principal changed, the kit slot opened, failed tries that this device did not make, a recovery started or cancelled. The alert for a pending recovery offers "Cancel and re-key".

**Unlock keys.** The list shows label, kind, creation date and last use for each key. A key unused for 180 days is flagged for review.

**Warnings at creation.** The app reads the backup flag of a new passkey. With the standard profile and a synced passkey it warns that one cloud account then holds both gates, and offers the strict profile.

**Error screens** for situations that look like a lost vault and are not:

| Situation | The screen says |
|---|---|
| Signed in with an Internet Identity that is not the owner | Wrong identity, the vault is fine. Switch identity or use the kit |
| No vault passkey on this device | The data is safe. Add this device from another one, or use the kit |
| Slot locked after wrong tries | Time left, that one try follows, the other ways in |
| Browser without PRF support | Use another browser or device. A passphrase key comes after the first release |
| Private window, or site data cleared | It works. Only the Internet Identity sign-in is asked again |

**Items**

- Delete moves an item to the trash. "Delete forever" is a separate action with its own confirmation.
- Each edit keeps the previous value of a changed field inside the item, last 10 values.
- A version conflict saves the local edit as a second item marked as a conflict copy. Nothing is merged or dropped silently.
- After the first release: an import is tagged as one batch and can be removed as one batch. After an import the app offers to delete the source file where the browser allows it, and reminds otherwise. **VERIFY** at M6 which browsers can delete a picked file.

**Banners.** A vault that was not installed with the environment name production shows a fixed banner on every screen: "Test vault, do not store real secrets". So does every build without the release flag, which only releases that passed the release gate carry. Every release also carries the label "Not independently reviewed. Keep here only what you can replace". It has to be acknowledged at setup and stands in the safety check. It goes only when a named person who is independent of the project has reviewed the design and published the result.

**Nothing else is stored.** Since the network key is deferred, the browser keeps no key of the vault at all.

## 9. Trust and supply chain

### 9.1 Who controls a vault canister

| Stage | Controllers | Upgrade path | When |
|---|---|---|---|
| A, self-hosted preview | Your production identity, protected by a password, and a backup controller kept on paper | You run the upgrade script | M4 to M6 |
| B, sovereign | The vault canister itself, with a break-glass path for the kit holder | You approve a published release inside the vault. How break-glass works is settled in the spike, see 9.5 | From M7 |

In neither stage do the project maintainers control anything.

### 9.2 Reproducible builds

- One script builds the frontend, embeds it, compiles, shrinks and compresses the Wasm, and prints its SHA-256.
- It runs inside a container image pinned by digest, with pinned Rust, Node and pnpm versions and locked dependencies.
- The same script is used locally, in the release process and by anyone who wants to check a release.
- Each release is a signed tag with the hash in the release notes.
- CI runs `pnpm check` on every pull request and builds each release tag on a second machine. A release counts only if the hash from CI equals the local one. CI holds no credentials.
- Each release archives its build inputs as release assets: the exported container image, the vendored crates and the pnpm store. A scheduled job rebuilds the latest release every month, so a release that can no longer be rebuilt is noticed early.
- The tag signing key sits on a hardware token or behind a passphrase. `docs/runbooks/release.md` covers its rotation.

### 9.3 Verification by anyone

- The module hash and the controller list of any canister are public. `scripts/verify.sh <canister-id>` reads both and compares the hash with a local build of the tagged source.
- `icp canister status --public` reads both from the public state tree without controller rights. `verify.sh` and the watchdog use it.
- The Trust screen shows canister id, canonical origin, subnet, module hash, controllers, cycles runway, slots and the audit log. It reads module hash and controllers from the public state tree itself, because a canister cannot see its own module hash in a query. It is a convenience. The independent check is the script, because a tampered interface could lie about itself.

### 9.4 Upgrade runbook, stage A

`scripts/upgrade.sh production` does all of this. The steps are listed so you know what it does.

1. Ask for a backup file made today, from M6 on, and run `doctor.sh`.
2. Build the release and compare its hash with the published one.
3. Check that the staging vault already runs this hash, and ask you to confirm that you unlocked it there.
4. Stop the canister and take a snapshot. The script keeps it for 30 days and then removes it. A canister holds at most 10 snapshots and pays storage for each.
5. Install the upgrade. Upgrade mode only.
6. Start the canister, run `verify.sh` and `doctor.sh`. Then you unlock once.
7. On failure, restore the snapshot.

### 9.5 Stage B design, to be proven by a spike first

- A release registry canister lists version, Wasm hash, source tag and publication time. The project publishes to it. Each vault pins the registry id at install.
- Inside an unlocked session you approve an upgrade to a listed version. The vault checks the hash against the registry, enforces a cooldown of 7 days after publication, fetches the Wasm, and installs it on itself.
- A stolen session can therefore install only official releases. The project can publish but cannot install.
- **Spike before committing:** prove on a local network that a canister listed as its own controller can install code on itself with a one-way call. If that proves fragile, use a tiny per-vault upgrader canister instead, as other IC projects do.
- After the first release, and only if decision D11 says yes: a launcher page and factory canister create vaults for people without a command line. The factory hands control to the vault itself in the same call and keeps nothing.
- **Break-glass and the waiting period.** A controller can replace the code and read all stored state. A kit principal that is a plain controller would let a kit thief skip the waiting period of section 6.6. The spike therefore also settles break-glass. Default: the kit holder is no controller, and code inside the vault or a small upgrader canister grants control only after the same waiting period. Fallback: the kit principal is a direct controller, and the waiting period is documented as protection for stage A only. **DECIDE** (D15)
- **Two-step switch.** Stage B starts with the backup controller still in place. It is removed only after the production vault has upgraded itself once. Until then a broken self-upgrade can be repaired from outside.
- **Approval without the web app.** An upgrade must also be approvable through the command-line tool and the kit. A web app that has grown too old to reach the network could otherwise never approve its own replacement.

### 9.6 Operator guard rails for stage A

In stage A you hold the controller, and a controller can destroy the vault with one command. These rules keep the destructive commands out of reach.

- **Scripts only.** Production is touched through `deploy.sh`, `upgrade.sh`, `topup.sh`, `retire.sh`, `verify.sh` and `doctor.sh`. `watch.sh` only reads. No runbook contains a raw `icp` command against production, with one exception: the move runbook deletes a retired canister by hand after 30 days.
- **No script can delete or reinstall.** Neither a canister deletion nor reinstall mode appears anywhere in `scripts/`. A test searches for both.
- **Typed confirmation.** Every script takes the environment as its first argument and prints the canister id, the subnet and the current module hash. For production it asks you to type the last five characters of the canister id.
- **Separate identities.** `vault-staging` uses keyring storage. `vault-prod` uses password storage, and the password is asked on every use, so no script and no agent can use it unattended. The staging identity is not a controller of the production vault, so a staging command cannot reach it. **VERIFY** at M4 whether a keyring exists under WSL. If not, staging uses password storage too.
- **Backup controller.** `deploy.sh` adds a second controller identity. `icp identity new` shows its seed phrase once. The phrase goes on paper and can be restored with `icp identity import --read-seed-phrase`. A lost laptop or a reset WSL then does not end upgrades. Keep that paper apart from the kit, because a controller key together with the kit skips the waiting period. A hardware key through the `link hsm` option of `icp identity` is the stronger choice. **VERIFY** at M4.
- **Checks before the claim link.** After creating the canister, `deploy.sh` asserts the subnet, the environment name, the freezing threshold, the controller list and the module hash. Only if all hold does it print the claim link. A canister on the wrong subnet is abandoned unclaimed, which costs the creation fee and nothing else.
- **`doctor.sh <environment>`** repeats these checks at any time and adds the cycles runway. It prints one line: healthy, or what is wrong.
- **Funding.** `docs/runbooks/top-up.md` describes one path from ICP to cycles, tried with a small amount first. `topup.sh` reads the canister id from the project, never from the keyboard.
- **Agents** prepare releases and never hold the production password. This repeats section 13 on purpose.
- **Explicit steps.** The scripts never call `icp deploy` on mainnet. That command builds, creates, applies settings and installs in one go and picks the install mode itself. The scripts call create, settings update and install separately, with install mode at creation and upgrade mode ever after.
- **Private deployment directory.** Everything about your own vaults lives in a private directory outside the public repository: the deployment config with the settings, the canister ids that `icp` writes under `.icp/data/`, and the log of every script run. The public repository ignores `.icp/` entirely. Agents work in the public repository and never see the private one. `ops-template/` is its starting point. **VERIFY** at M4 how `icp.yaml` declares a released Wasm by file and hash.
- **Settings are declared, not typed.** Freezing threshold, controllers and the other canister settings stand in the config of the private directory. `doctor.sh` compares the live canister with that file after every script run.

## 10. Repository layout

```
ic-vault/
  AGENTS.md  CLAUDE.md  README.md  SECURITY.md  LICENSE
  icp.yaml  Cargo.toml  rust-toolchain.toml  deny.toml
  package.json  pnpm-workspace.yaml  pnpm-lock.yaml
  canisters/vault/
    vault.did
    src/  lib.rs  state.rs  access.rs  claim.rs  unlock.rs  sessions.rs
          slots.rs  items.rs  sync.rs  audit.rs
          limits.rs  http.rs  upgrade.rs
    tests/  authz.rs  unlock.rs  recovery.rs  items.rs  rekey.rs  http.rs  upgrade.rs
  apps/web/
    index.html  vite.config.ts
    src/
      crypto/    worker.ts  kdf.ts  slots.ts  items.ts  kit.ts
                 wordlist.ts  proof.ts  padding.ts  types.ts
      platform/  identity.ts  passkey.ts  agents.ts  vault-actor.ts
      vault/     machine.ts  sync.ts  store.ts
      ui/        setup/  unlock/  items/  devices/  recovery/  safety/  trust/
      main.tsx
    e2e/
  packages/
    declarations/     generated TypeScript bindings
    vectors/          crypto test vectors as JSON
    shared/           limits.json and the constants generated from it
  tools/cli/          Rust recovery and backup tool, from M6
  docker/Dockerfile.build
  ops-template/       starting point for your private deployment directory
  .github/workflows/  check on pull requests, release rebuild, watchdog template
  scripts/  setup.sh  dev.sh  check.sh  build-release.sh  deploy.sh  verify.sh  upgrade.sh  doctor.sh  topup.sh  retire.sh  watch.sh
  docs/  threat-model.md  crypto-spec.md  operations.md  implementation-plan.md  adr/  runbooks/ (setup-day, recovery, upgrade, top-up, move, release, hotfix)
```

## 11. Tooling, environments, costs, setup and upkeep

### 11.1 Tooling

- **Work inside WSL Ubuntu.** On native Windows the local network needs Docker Desktop. Cloud agents run Linux anyway.
- **CLI.** `icp-cli` 1.6 with `icp.yaml`. The official docs now lead with it and provide a migration guide from dfx. This replaces the dfx choice in the earlier plan. The docs index already carries a newer version label, so M0 pins the release that is current then, and the monthly task tracks it.
- **Bindings.** `@icp-sdk/bindgen` generates TypeScript from `vault.did`.
- **Local network.** `icp network start -d`, with Internet Identity enabled by `ii: true` on the local network in `icp.yaml`.
- **Skills.** `npx skills add dfinity/icskills` pins the official skills for Claude Code and Cursor. The relevant ones are canister-security, internet-identity, stable-memory, certified-variables, cycles-management and icp-cli.
- **Tests.** PocketIC for the canister, Vitest for the worker, Playwright with a virtual authenticator for end to end. **VERIFY** that the virtual authenticator supports PRF in the pinned Chromium.

### 11.2 Environments

| Environment | Network | Subnet | Purpose |
|---|---|---|---|
| local | local managed network | n/a | Development and all automated tests |
| staging | mainnet | confidential | A vault with fake data, configured exactly like production. Every setup and every upgrade is rehearsed here first |
| production | mainnet | confidential | Your real vault |

Staging and production differ in one install argument, the environment name. It drives the test banner.

There are no secrets in the repository and none in the canister. The only credentials are your controller identities, see section 9.6.

### 11.3 Costs per vault

Published prices are for a 13-node subnet and scale with node count. The confidential subnet has 7 nodes, so these figures are scaled estimates. One trillion cycles costs about 1.37 USD. **VERIFY** against the dashboard at M4.

| Item | Cycles | About |
|---|---|---|
| Create the canister | 0.27 trillion | 0.37 USD once |
| Storage | 2.2 trillion per GiB per year | 3 USD per GiB per year |
| A vault with 100 MiB | 0.2 trillion per year | 0.30 USD per year |
| Network key derivation | none | The network key is deferred |
| Ordinary update call | a few million | negligible |
| Upload, per MiB | about 1.1 billion | 0.0015 USD |
| Queries | free | free |

Fund a new vault with 3 to 5 trillion cycles. That covers years of normal use and the 180-day freezing reserve. The safety check warns when fewer than 365 days remain before freezing. `icp canister create` attaches 2 trillion cycles unless told otherwise, so `deploy.sh` names the amount. A snapshot is billed like the storage it copies.

### 11.4 Setup and upkeep risks

> Since task M0-02 the source of truth for sections 11.4 to 11.6 is `docs/operations.md`. The text here is the copy of revision 8 and is not updated.

Section 2 covers attackers and owner mistakes. This section covers what is left: bringing a vault to life, and keeping it alive for years. Nothing is deployed yet, so the check ran against this plan and the current documentation. Ratings use the scale of section 2. "Before" rates revision 2.

**What the check against the current docs found**

| Topic | Revision 2 said | The docs say | Change |
|---|---|---|---|
| Vault address | `<id>.icp0.io`, to be verified at M4 | Every canister is reachable at `<id>.icp.net`. The pages read name no other domain | Default is `icp.net`. Confirmed at M0, before any passkey exists |
| Mainnet install | `icp deploy -e ic` | Deploy builds, creates, applies settings and installs in one step and picks the mode itself. A reinstall mode exists, and no confirmation is documented | Scripts call each step with a named mode |
| Canister ids | Not mentioned | Stored under `.icp/data/`, which projects usually commit | A private deployment directory. The public repository ignores `.icp/` |
| Freezing threshold | 180 days | The default is 30 days | Declared in the deployment config and checked by `doctor.sh` |
| Public state | Command unknown | `icp canister status --public` shows module hash and controllers to anyone | `verify.sh` and the watchdog use it |
| Staging key | `test_key_1` | `test_key_1` is for development, `key_1` for production | Moot while the network key is deferred |
| Snapshots | One before each upgrade | At most 10 per canister, the canister must be stopped, each is billed as storage | Kept 30 days, then removed |
| API host | To be verified at M4 | `https://icp-api.io` | Written into the page policy |
| Tool version | `icp-cli` 1.6 | The docs index already carries a newer version label | Exact pin at M0, tracked monthly |

Two questions the docs did not answer. The staging rehearsal settles both:

- whether `icp` installs a Wasm above 2 MiB without extra steps
- whether a keyring exists under WSL

**Setup risks**

| # | Risk | Cost | Before | After | Safeguard |
|---|---|---|---|---|---|
| S1 | The vault address rests on a gateway domain that changes, or that was chosen wrongly | Passkeys and the Internet Identity principal are bound to it. Every device has to enrol again | High | Medium | Default follows the docs and is confirmed at M0. The canister redirects every other host. The kit does not depend on the address, and a flow for changing it comes after the first release. Stays a platform risk, R7. |
| S2 | The subnet proves to be the wrong choice and cannot be changed | The vault has to move | Medium | Medium | The vault id is separate from the canister id, so "Move a vault" can be added after the first release without changing a format. Until then: a backup, the offline tool, and entering the data again. |
| S3 | The production key is first used by the real vault, or its name is retired later | Setup fails, or no new device can enrol | Medium | Low | Does not apply to the first version, which uses no threshold key. It applies again when the network key returns. |
| S4 | A deploy applies more than intended: an install mode it picked itself, or settings that changed in the config | Wrong settings or controllers go live unnoticed. At worst a reinstall wipes the vault | Medium | Low | Named steps and modes. The config sits in the private directory. `doctor.sh` compares the live canister with it after every run. |
| S5 | Canister ids and deployment state land in the public repository | The address of your vault is published. Agents and CI see production targets | Medium | Low | Private deployment directory. `.icp/` ignored. CI without credentials. |
| S6 | The first page load is trusted blindly, in a browser full of extensions | The code that creates the vault key and the kit could be altered or watched | Medium | Medium | `verify.sh` right before the claim link is opened. A browser profile without extensions. The gateway stays in the path, R1 and R18. |
| S7 | Funding fails on the day: exchange delays, tokens sent to a wrong account, too few cycles | Days of delay, or lost money | Medium | Low | Funding a week ahead with the documented commands and a small test amount first. `deploy.sh` checks the balance before it creates anything. |
| S8 | The procedure runs for the first time on the real vault, with tools that have moved on | A half-created vault and improvised commands | Medium | Low | Full rehearsal on staging within 7 days before, with the same release, scripts and tool versions. |
| S9 | The Internet Identity has one passkey and no recovery | One lost phone means the kit and a 48-hour wait | Medium | Low | The setup-day checklist asks for a second passkey or a recovery phrase before the claim. |
| S10 | The device cannot do what setup needs, and this shows halfway | A printed kit from an aborted attempt, or a weaker profile chosen in a hurry | Medium | Low | Device check before anything is created. |

**Upkeep risks**

| # | Risk | Cost | Before | After | Safeguard |
|---|---|---|---|---|---|
| K1 | Nobody is told when something goes wrong: cycles low, code or controllers changed, vault unreachable | The problem is found at the next unlock, or too late | Medium | Low | Daily watchdog outside the vault. It reads public data, holds no key and alerts through the scheduler it runs on. |
| K2 | An upgrade breaks the vault or its data | Downtime, or loss of what was written since the snapshot | Medium | Low | Staging first. Named upgrade mode. Snapshot kept 30 days, backup file before. No downgrades. Upgrade test against a full vault. |
| K3 | A dependency or a tool is poisoned, or a published flaw goes unnoticed | Malicious or vulnerable code in a release | Medium | Low | Exact pins. Seven-day release age. Monthly update task with a fixed list of sources. Second build in CI. |
| K4 | An old release can no longer be rebuilt | A deployed hash cannot be verified any more | Medium | Low | Build inputs archived with each release. Monthly rebuild of the latest release. |
| K5 | The platform changes underneath: gateway domain, API version, Internet Identity, subnet, PRF support in browsers | The embedded web app stops working, or a setup choice becomes invalid | Medium | Medium | Watch list checked monthly. Sign-in and unlock on staging every month. A rebuild and upgrade every year. Kit, backup file and offline tool need none of it. |
| K6 | The vault has to move to another canister or subnet | Without preparation, a migration under pressure | Medium | Medium | Until "Move a vault" exists, after the first release: a current backup, the offline tool, and entering the data again. The yearly drill decrypts a backup offline. |
| K7 | The switch to self-controlled upgrades goes wrong | A vault that can never be upgraded again | Medium | Low | Two-step switch. The backup controller stays until one self-upgrade worked. Upgrades can be approved with the command-line tool and the kit. |
| K8 | Upkeep depends on one person and on chat sessions that are gone | Runbooks rot, routines stop | Medium | Low | Every routine is a script or a prompt in the repository. Each run is logged. Runbooks are executed on a calendar. |
| K9 | A flaw needs a fix within days | A slow, improvised release | Medium | Low | `SECURITY.md` points to GitHub's private vulnerability reporting. A hot-fix runbook that keeps the staging rehearsal. In stage B the cooldown still applies, R19. |
| K10 | The tag signing key is lost or stolen | No releases, or a false one | Medium | Low | Key on a hardware token. Rotation runbook. The second build in CI and, in stage B, the cooldown limit the damage. |

**What the assessment shows**

- Setup is where the choices are made that cannot be taken back. The table in 11.5 names each one and its way out. The address domain rated High because revision 2 defaulted to the older domain and checked it only at the milestone that creates the first vault.
- Upkeep fails through silence. The vault cannot call for help, so a watchdog outside it and a calendar do.
- Five risks stay Medium. Three belong to the platform: the address domain, the gateway in front of the first page load, and change underneath. Two wait for "Move a vault", which the cut line puts after the first release.
- The minimum that protects the data is small: enough cycles, a current backup file and the kit. Everything else in this section buys early warning and convenience.

### 11.5 Setup procedure

A vault is set up twice: once on staging as a rehearsal, then for real. Both runs use the same release, the same scripts and the same tool versions.

**Choices that are fixed at setup**

| Fixed | Why | Way out |
|---|---|---|
| Subnet | Chosen when the canister is created | A backup, the offline tool and a new vault. After the first release: "Move a vault" |
| Canister id | Given at creation, part of the address | The same |
| Gateway domain | Passkeys and the Internet Identity principal are bound to the full address | The kit under the new address, with its waiting period. After the first release: "Change the address" |
| Vault id | Bound into every ciphertext | None needed. It moves with the vault |
| First controllers | Set at creation | Changed by script in stage A |

**Steps**

| Step | When | What | Checkpoint |
|---|---|---|---|
| 1. Fund | A week before | `docs/runbooks/top-up.md`: account id, small test transfer, full transfer, mint cycles | `icp cycles balance` covers both vaults |
| 2. Prepare identities | A week before | A second passkey or a recovery phrase on your Internet Identity. Create `vault-staging`, `vault-prod` and the backup controller | You can sign in to Internet Identity from two devices |
| 3. Rehearse | Within 7 days before | Steps 4 to 7 against staging with fake data, then one upgrade and one kit recovery | Every checkpoint below passed on staging. Problems are fixed in the scripts, not worked around |
| 4. Create | Setup day | `deploy.sh production`: balance check, create on the subnet with named cycles and controllers, apply settings, install in install mode | The script stops at the first failed check |
| 5. Verify | Setup day | `doctor.sh` and `verify.sh`: subnet, environment name, freezing threshold, controllers, module hash against the release | Both report healthy. Only then is the claim link shown |
| 6. First run | Setup day, right after | The setup-day checklist, then the flow of section 6.6 | Kit entered in full. Practice unlock succeeded |
| 7. Close | Same day | Second unlock key. Bookmark. Kit copies in their two places. Watchdog switched on. First line in the log | The safety check is green except for the 30-day kit check |

**Setup-day checklist**, also kept as `docs/runbooks/setup-day.md`:

- Thirty quiet minutes, and nobody else can see the screen.
- A printer you trust, or pen and paper.
- A browser profile without extensions. A private window will do.
- The second device at hand.
- The two places for the kit copies are decided.
- The staging rehearsal is less than 7 days old.

### 11.6 Upkeep routine

**The watchdog** is `scripts/watch.sh`, run once a day by a scheduler of your choice. It needs the canister id and nothing else.

- It reads the module hash and the controllers with `icp canister status --public`, calls `status`, and loads the start page.
- It fails when the vault does not answer, when the module hash is not on the list of released hashes, when the controllers differ from the recorded ones, or when the health value is not `ok`.
- A failure reaches you through the scheduler: a failed workflow run that is mailed to you, or a local task that shows a notification. `ops-template/` ships a scheduled workflow for a private repository. **DECIDE** (D17)
- It holds no key and can change nothing. Whoever runs it learns the address of the vault and nothing more.
- After a failure the rule is: do not unlock before `doctor.sh` and `verify.sh` have run.

**Calendar**

| When | What | Who | Time |
|---|---|---|---|
| Daily | Watchdog | Scheduler | None |
| Monthly | Update task from Appendix E: advisories, pins, watch list. Sign in and unlock on staging. Confirm that the watchdog still runs | Agent prepares, you review | 20 minutes |
| Every 3 months, from M6 | Backup and "Verify a backup". `doctor.sh`. Review the list of unlock keys | You | 15 minutes |
| Yearly | Kit check. Backup drill: decrypt a backup with the offline tool. Rebuild and upgrade even without changes. Run every runbook once and date it. Look again for a funded or volunteer review | You | 2 hours |
| On each release | Upgrade runbook, staging first | You | 30 minutes |

**Watch list**, checked in the monthly task:

| What can change | Where it shows | What to do |
|---|---|---|
| Gateway domain | Custom domains guide, developer forum | The kit under the new address. A flow for it comes after the first release |
| HTTP API versions | SDK release notes | Upgrade within the announced window |
| Internet Identity | Its release notes, and the monthly sign-in on staging | Fix and release before production breaks |
| Subnet: members, confidential setup, public access | The subnet page on the dashboard | A backup, the offline tool and a new vault. "Move a vault" comes after the first release |
| Cycle prices | Cycle costs reference | Adjust the funding |
| PRF in browsers and authenticators | An unlock on each of your devices after a major browser update | Another device, or the kit. A passphrase key comes after the first release |
| `icp-cli` and SDK versions | Release notes | Bump the pins in one pull request |

**Urgent fixes.** `SECURITY.md` gives a private way to report a flaw: GitHub's private vulnerability reporting, which needs no mailbox. `docs/runbooks/hotfix.md` is the short path: fix, review, release, staging, production, on the same day if needed. It may skip the 7-day release age through the documented exclusion. It never skips staging.

**The log.** Every script appends one line to `log.md` in the private directory: date, script, environment, release, result. The yearly routine reads it.

**What upkeep is for.** A funded vault that nobody tends keeps working until its cycles run out or a platform change breaks the embedded web app. Even then the data stays readable with the kit, a backup file and the offline tool.

## 12. Testing and security gates

`pnpm check` is the only gate and must pass before every commit. It runs, in order: format and lint, type check, Rust tests, PocketIC tests, worker tests, bundle checks, dependency audits.

**Canister**

- An authorization matrix test: every method called as anonymous, stranger, owner without session, enrolment session and full session. The expected result for each cell is a table in the test file. A new method without a row fails the build. The matrix runs twice: as ingress, and through a proxy canister, because a call from another canister skips the message hook.
- Unlock: valid signature, bad signature, stale nonce, replayed request, locked slot, lockout timing, reset of the failed tries.
- Unlock, guess limiting: the failed-try count survives the failed call, and `canister_inspect_message` never reveals whether a PIN was right.
- Unlock, second limit: four wrong tries after each success reach the limit of ten in thirty days. Tries on two slots count together. After 30 that nobody cleared, the slot stays locked until a full session clears it.
- Unlock, per slot: each slot has its own nonce. The kit principal is refused on every other slot, and in the hook while a recovery is pending.
- Unlock, two proofs: a call without the right passkey proof is refused in the hook and never counted.
- Unlock, order: a trap forced after the check leaves nothing behind, and a wrong PIN is counted even when the log cannot be written.
- Sessions: removing a slot ends its sessions, and a re-key ends all others.
- Enrolment: without a proof of the vault key there is no session. A handoff begins once.
- The message hook refuses ingress calls to query methods, and an unlock with a stale nonce.
- Alert log: a burst of failed unlocks pushes nothing out of it.
- Slots: the kit kind, the kit slot as a replace target, a retired credential id and fields that do not fit the kind are all refused.
- Recovery: a ready recovery is used once, lapses after 7 days, and is cleared by a kit replacement and by a re-key.
- Items: version conflicts, size limits, quota, tombstones, sync paging.
- Re-key: atomicity when an item changes mid-way.
- Upgrade: install each released version, load data, upgrade to head, verify everything is intact and all sessions are gone.
- HTTP: responses verify against the certified data, headers are present, raw hosts are redirected, and the path for alternative origins answers not found.

**Worker**

- Known-answer vectors for every construction in section 6.3. The same vectors are later consumed by the Rust CLI.
- Property tests: round trip, and any single flipped bit in ciphertext, nonce, tag or associated data must fail.
- Wrong vault id, wrong item id, wrong version must fail.
- Argon2id parameters below the minimum are refused. A relying party id other than the full host is refused.
- Coverage of `crypto/` at 95 percent or more.

**End to end**

- Setup, unlock, add item, lock, unlock again, add device, recover with kit, re-key.
- **Canary test.** A known marker string is stored as a password. The test records every request to the canister and every write to browser storage and fails if the marker appears anywhere.
- Bundle test: no `eval`, no `fetchRootKey`, no absolute URLs outside the allowlist.
- End-to-end tests load the page that the canister serves, with the final headers, never the development server.
- A backup file with one changed byte is caught by "Verify a backup" and by the offline tool.

**Owner mistakes**

- A test for a safeguard carries the mistake number in its name, for example `u14_trash_restore`. A mistake without a test is either marked advice only in the table or a gap.
- Setup: cannot finish with a wrong word, a missing word or a skipped practice unlock. An abandoned setup leaves no slot behind.
- Kit: the BIP39 test vectors pass. A replacement interrupted before `kit_replace` leaves the old kit working. After it the old kit fails and the screen shows the new number. The kit slot cannot be removed.
- Waiting period, with the PocketIC clock: the kit principal gets "pending", a device session cancels, a new attempt waits again, and after the delay the slot opens. Internet Identity plus kit opens at once.
- PIN rules: every refused pattern is refused, and wrong tries show the remaining count.
- Passkeys: a second enrolment on the same authenticator is stopped by the exclude list.
- Trash: delete, restore, erase after 30 days, tombstone until day 90. A version conflict produces a copy.
- Alerts: an unlock key added in one session blocks the next unlock on another device until acknowledged.
- Scripts: `scripts/` contains neither a canister deletion nor reinstall mode. On the local network `deploy.sh` refuses to print a claim link when a setting is wrong.
- Banner: a vault not installed as production shows the banner on every route.
- **Hallway test** at M5: one person who has never seen the project sets up a vault, loses the device in a role play, and recovers with nothing but the kit sheet. You watch and do not help. Everything they stumble over becomes a task, and the ratings in the owner-mistake table are corrected.

**Setup and upkeep**

- `deploy.sh` on the local network stops when the balance is too low, when a setting differs from the config, and when the module hash is not the release hash.
- `doctor.sh` reports each single deviation: subnet, environment name, freezing threshold, controller list.
- `watch.sh` fails on an unknown module hash, a changed controller, an unreachable canister and a health value other than `ok`. One test proves each.
- `status` returns no balance and no size.
- Upgrade: from every released version to head, with a vault filled to its limits. Newer data with older code is refused. The instruction count is recorded.
- After the first release, Move: backup, restore into a second canister, unlock there with the kit. Every item decrypts and the device slots are gone. A second restore into the same canister is refused.
- After the first release, address change: with two accepted hosts, `owner_set` and one enrolment bring a device to the new host without the kit.
- CI: a pull request cannot merge without `pnpm check`, and the release hash from CI equals the local one.
- The rehearsal is the acceptance test of the scripts: section 11.5 on a fresh staging vault from start to finish, at the end of M5.

**Reviews**

- Changes under `crypto/`, `platform/` or `scripts/`, or in `access.rs`, `unlock.rs`, `sessions.rs`, `slots.rs`, `claim.rs`, `http.rs` or `upgrade.rs`, need a second review by a fresh agent session that sees the spec and the diff, not the author's reasoning, followed by your approval.
- Before the interface is frozen: a design review of sections 6 and 7. It was done on 2026-11-09 by Claude, the author of this plan, instead of an outside specialist. Its eighteen findings are in `DESIGN-REVIEW-FINDINGS.md`, and revision 5 applies them.
- Before M4 and before M7: a full security review pass with the strongest available model against `docs/threat-model.md`.
- Before real secrets: the release gate below.

**The release gate**

There is no budget for a paid audit, decided on 2026-11-09. Three free steps take its place. They are weaker than an audit, and the label says so.

- **A review by a second model.** A model from another vendor reads `docs/crypto-spec.md`, the canister access logic and the worker against `docs/threat-model.md`. It did not write the design. Every finding is fixed, or accepted by you in writing.
- **A public call for review.** The crypto spec and the review brief are posted on the Internet Computer developer forum and in one cryptography community. The gate waits 30 days. Silence is not approval, and the call stays open afterwards.
- **A label.** The README, the setup screen and the safety check say "Not independently reviewed". The README keeps the word experimental as long as the label stands.

The gate opens stage 2 of section 1 and nothing more. Once a year the upkeep checks whether a funded review has become possible.

## 13. Agent workflow

- One task, one branch `task/<name>`, one pull request. Each task names its milestone and its done-when line.
- Spec first. If a task needs a change in section 6 or 7, the agent writes an ADR and stops for approval before writing code.
- Agents load the pinned ICP skills before writing canister or SDK code. Their API names are newer than model training data.
- Agents never weaken, skip or delete a failing test to reach green. They report it.
- Agents never add a dependency, a network request, a log line containing arguments, or a storage write outside what rule 15 of `AGENTS.md` allows.
- Mainnet deploys and upgrades are run by you, not by an agent. Agents prepare the release and the runbook output.
- A new destructive or irreversible action, in the interface or in a script, first gets a row in the owner-mistake table and a safeguard.
- Upkeep is agent work too. The monthly task has a fixed prompt in Appendix E and ends in a report and at most one pull request. It needs no mainnet access.
- `AGENTS.md` is the single source for agent rules. `CLAUDE.md` contains only `@AGENTS.md`.

## 14. Milestones

**M0 Bootstrap**
- Repository, toolchain pins, `icp.yaml`, skills, `AGENTS.md`, spec documents copied from this plan.
- Decision D8 on the gateway domain. CI that runs `pnpm check` on every pull request. Seven-day release age for packages. `.icp/` ignored.
- The hello page carries the final headers and runs once on a throwaway mainnet canister: subnet, Internet Identity sign-in, and a passkey with PRF on the real kind of address.
- A hello canister that serves a certified hello page.
- Done when: `icp network start -d` and `icp deploy` work in WSL, the page loads from the local canister, `pnpm check` passes locally and in CI, and D8 is decided.

**Design review**
- The design is reviewed against `DESIGN-REVIEW-BRIEF.md` before any format is frozen. Done on 2026-11-09 by Claude instead of a specialist. The findings are in `DESIGN-REVIEW-FINDINGS.md`. You decided on each the same day: every proposed correction is applied, and the network key is deferred.
- Done on 2026-11-09. This document is at revision 5.

**M1 Canister core**
- State, claim, owners, slots, unlock with signature check, counter and lockout, sessions, items with trash, sync, limits, audit log, the waiting period and the other rules of section 7.9. Tests drive it with fixed test keys. No frontend.
- Done when: the authorization matrix and all canister tests pass, and the upgrade test passes from an empty and a filled state.

**M2 Crypto core**
- The worker implements section 6 with vectors and property tests. No network code.
- Argon2id benchmark on a mid-range phone, decision recorded.
- Done when: all vectors pass, coverage target met, the import boundary lint is active.

**M3 Vertical slice**
- Internet Identity sign-in, claim, setup exactly as in section 6.6 with full kit entry, PIN rules and practice unlock, unlock, logins and notes, auto-lock with drafts, test-vault banner.
- Done when: the end-to-end flow and the canary test pass on the local network, and setup cannot be completed with a wrong kit word.

**M4 Hardened packaging and the staging vault**
- Single Wasm with certified assets and headers, reproducible build, deploy script for the confidential subnet, verify script, Trust screen, freezing threshold, self-hosting guide.
- The guard rails of section 9.6: script rules, two identities and a backup controller, `doctor.sh`, top-up runbook.
- Setup as in section 11.5, as far as it goes before M5: private deployment directory, funding, a staging vault, setup-day checklist. No production vault yet.
- Upkeep as in section 11.6: health value in `status`, watchdog, log, first monthly task, monthly rebuild of the release in CI.
- Done when: two clean builds on different machines give the same hash, `verify.sh` passes against the staging vault, `doctor.sh` reports healthy, a deliberately wrong deployment is refused on the local network, the staging vault was created by the scripts and upgraded once, the watchdog has caught a deliberately wrong hash, and the vault works end to end with test data.

**M5 Devices and recovery**
- Add device by QR or typed code, remove device, kit recovery in both cases, kit replacement, new Internet Identity, re-key, change PIN, audit screen, lockout messages.
- Everything else in section 8.5: safety check, kit check, blocking alerts, error screens, trash screen.
- Done when: every flow in section 6.6 has an end-to-end test, the hallway test of section 12 has been run and its findings fixed, the full rehearsal of section 11.5 has passed on a fresh staging vault, and a written recovery drill has been performed by you once.

**M6 Backups and the recovery tool**
- Backup export locked with the kit. A Rust command-line tool that decrypts a backup offline.
- Backup reminder and "Verify a backup".
- Done when: a backup made in the browser is decrypted by the tool with only the kit, and a changed byte in a backup is caught.

**Release gate**
- The three steps of section 12: a review by a model of another vendor, a public call that waits 30 days, and the label. Findings fixed, or accepted in writing.
- The production vault is created from the release that passed the gate, following section 11.5. You recover it once with the real kit on a second device. Only then stage 2 of section 1 opens.

**M7 Sovereign upgrades**
- Self-upgrade spike, release registry, in-vault upgrade with consent and cooldown, controllers reduced in two steps as settled in the spike including break-glass.
- Done when: a vault upgrades itself between two test releases on the local network and on staging, and refuses an unlisted hash.

**M8 Later**
- An independent review, as soon as a reviewer or the money for one turns up. It is the other condition of stage 3.
- A client that does not arrive through the gateway: a browser extension, or a verifying gateway on your own machine. First in line, decision D19.
- The network key of section 6.5, deferred on 2026-11-09.
- What the cut line deferred: files, import, unlock keys without a PIN and without a passkey, "Move a vault", the change of address, the quota setting, a QR decoder, and the launcher.
- Sharing an item with another vault through vetKeys identity-based encryption.
- Inheritance: a timer releases a sealed share to a named heir after a period without unlocks.
- One-time codes, browser extension with autofill, which also removes the gateway from the delivery path.

## 15. Residual risks

| # | Risk | Why it remains | Mitigation |
|---|---|---|---|
| R1 | Others deliver the web app | Page loads are verified by the HTTP gateway, not by the browser, and the subnet certifies what the canister serves. A malicious gateway, or a majority of the subnet's nodes, could serve altered code to one user. Altered code reads everything at the next unlock | Use only the canonical origin. The M6 command-line tool depends on neither. A client that does not arrive through the gateway comes first after the release and is a condition of stage 3, decision D19. |
| R2 | Controller keys in stage A | A controller can replace the code, including the interface | A password-protected production identity, and a backup controller on paper kept apart from the kit. Stage B removes both. |
| R3 | Compromised endpoint | Plaintext must exist somewhere to be read | Strict page policy, minimal dependencies, short auto-lock. |
| R4 | Synced passkeys with the standard profile | One cloud account then holds both passkeys | Strict profile by default, or a hardware security key. |
| R5 | Kit theft | The kit is the master key by design | Paper only, stored apart from devices and from backups. Waiting period when it is used without the Internet Identity. |
| R6 | The confidential subnet is new and small | 7 nodes tolerate 2 faults. SEV-SNP has published side-channel research | Data is encrypted before it arrives. Regular backups. Revisit the subnet choice at M4. A vault can move to another subnet as ciphertext. |
| R7 | Origin changes | Passkeys and the Internet Identity principal are bound to the vault's origin. Gateway domains have changed before | Pin one canonical origin, by default the one the current docs give. "Change the address" in 6.6. The kit works regardless of origin. |
| R8 | Running out of cycles | A canister that stays frozen too long is deleted | 180-day threshold, runway warning on the unlock screen, top-up without unlock, backups. |
| R9 | Agent-written security code | Subtle mistakes pass tests | Spec with vectors, a second implementation, fresh-context reviews, and the release gate of section 12. There is no audit, see R26. |
| R10 | Platform governance | The protocol and Internet Identity are upgraded by network governance | No single platform component can read data alone. Backups need only the kit. |
| R11 | Metadata | Boundary nodes see the caller, call timing and padded sizes | Accepted for v1. |
| R12 | PRF availability | Not every browser and authenticator supports it | Feature detection. The passphrase slot comes after the first release, so until then a device without PRF cannot unlock. Spike SP1 checks your devices first. |
| R13 | Kit handling by the owner | A photo, a file or a fake page gets the whole kit. Together with the Internet Identity, a backup file or a controller key there is no waiting period | Paper-only wording, check code for routine checks, address on the sheet, waiting period for kit-only use. |
| R14 | Clipboard and plaintext files | A copied password or an import file is outside the protection of the vault | Timed clearing and no plaintext export in the web app. Import and its delete offer come after the first release. |
| R15 | The vault cannot reach its owner | No email, no push. A pending recovery or a shrinking balance is seen only at the next visit | Adjustable waiting period, generous funding, warnings before unlock. |
| R16 | Raw commands in stage A | A controller can still delete the canister by hand | Scripts only, password-protected production identity, backups. Stage B removes the deployer. |
| R17 | The safeguards are code too | Trash, waiting period and checks add surface that can be wrong | Tests named after each mistake, and all of it inside the review of the release gate. |
| R18 | Browser extensions | An extension that may read every page sees plaintext and, at setup or recovery, the kit | A browser profile without extensions for the vault. The setup and recovery screens say so. Nothing can enforce it. |
| R19 | The cooldown delays urgent fixes | In stage B a release can be installed only 7 days after publication | Accepted. Until then the owner can leave the vault locked. |
| R20 | The watchdog needs an outside scheduler | It learns the address of the vault and whether a recovery is pending, and it can stop running unnoticed | It holds no key. The monthly task confirms that it still runs. |
| R21 | A moved vault exists twice for a while | The old canister still opens with the same kit until it is retired | A move ends with stopping the old canister. Re-key when the move was about trust. |
| R22 | Any canister can make the vault pay | A call from another canister skips the message hook, and the vault pays for refusing it. Draining a vault costs the caller less than it costs the owner | Refusal in the first statement, generous funding, the watchdog, top-up without unlock. A frozen vault keeps its data for 180 days. |
| R23 | A controller can turn the vault back | Restoring a snapshot brings back removed unlock keys, an old PIN and a replaced kit | Only a controller can do it, and a controller can replace the code anyway. Stage B removes the outside controller. Devices could remember the newest state they have seen. That is not built in the first version. |
| R24 | No network key in the first version | A copy of the canister state together with the secret of a vault passkey reads the vault, because the PIN can then be found offline | Your decision of 2026-11-09, see 6.5. Both thefts are needed. The slot format keeps room for the network key, and it is on the list for after the first release. |
| R25 | The first release cannot move | Until "Move a vault" exists, leaving a canister or a subnet means reading a backup with the offline tool and entering the data again | The vault id is in every format, so the flow needs no format change. Backups and the offline tool are part of the first version. |
| R26 | No independent review | No specialist and no auditor reads the design or the code before real secrets go in. A flaw in the key handling could expose everything without a sign | The release gate of section 12, the label, and the trust stages of section 1. Your current password manager stays the master copy. |

## 16. Open decisions

| # | Decision | Default |
|---|---|---|
| D1 | Product and repository name | Decided on 2026-11-09: `ic-vault` |
| D2 | License | Decided on 2026-11-09: Apache-2.0 |
| D3 | Protection profile | Strict, passkey plus PIN. It is the only profile in the first release |
| D4 | Subnet | Confidential, 7 nodes |
| D5 | Network key | Decided on 2026-11-09: deferred until after the first release. Then per device, cached |
| D6 | Auto-lock time | 5 minutes idle |
| D7 | Internet Identity sign-in lifetime, passed as `maxTimeToLive`, ADR 0006 | 8 hours |
| D8 | Canonical gateway domain | Decided on 2026-11-09: `icp.net`, ADR 0002, on condition that spike SP1 passes there on every device you use. For an existing vault it changes only by enrolling every device again |
| D9 | Kit format | 24 words in BIP39 English encoding, plus QR code and an 8-character check code |
| D10 | External audit | Decided on 2026-11-09: none, because there is no budget. The release gate of section 12 takes its place |
| D11 | Whether to offer a hosted launcher at all | Decide after the first release |
| D12 | Waiting period for recovery with the kit alone | 48 hours |
| D13 | Trash retention | 30 days |
| D14 | Kit check rhythm | 30 days after a kit is made, then yearly |
| D15 | Break-glass control in stage B | Granted by code after the waiting period. Settle in the M7 spike |
| D16 | Vault id separate from the canister id | Yes. A vault then moves as ciphertext. The alternative is listed in Appendix D |
| D17 | Where the watchdog runs | A scheduled workflow in a private repository. A scheduled task on your own machine is the alternative |
| D18 | Key for the staging vault | Moot while the network key is deferred. Then `key_1`, the same as production |
| D19 | First real secrets with the web app alone, or only with a client that does not arrive through the gateway | Decided on 2026-11-09: in stages, see "Trust stages" in section 1. Secrets you can rotate go in with the web app alone. Everything else waits for an independent review and for such a client |
| D20 | Scope of the first release | The cut line, decided on 2026-11-09. See section 1 |
| D21 | Repository | Decided on 2026-11-09: public from the first commit, owned by a GitHub organization, flaws reported through GitHub private vulnerability reporting |

## Appendix A. Draft AGENTS.md

```markdown
# AGENTS.md

Vault is a single-user encrypted vault on the Internet Computer. It is security software.
Read docs/threat-model.md and docs/crypto-spec.md before changing code.

## Commands
pnpm setup          install toolchains and dependencies
pnpm dev            local network, canister, web app
pnpm check          the only gate, must pass before every commit
pnpm e2e            end-to-end tests on the local network
pnpm build:release  reproducible release build, prints the Wasm hash

## Hard rules
1. Vault keys exist only in apps/web/src/crypto. Nothing else imports noble, and platform/ uses
   WebCrypto only through the SDK, for identities.
2. The canister stores and returns ciphertext. It never decrypts and never logs arguments.
3. docs/crypto-spec.md is law. Changing a constant, label, format or flow needs an ADR and human approval.
4. Every canister method starts with its gate. A new method needs a row in the authorization matrix test first.
5. No new runtime dependency without an ADR. Exact versions only.
6. Never weaken, skip or delete a failing test. Report it.
7. No requests to any origin other than the IC. No analytics, fonts or CDNs.
8. Never call fetchRootKey. Never use innerHTML, eval or inline scripts.
9. Load the pinned ICP skills before writing canister or SDK code.
10. One task, one branch task/<name>. Mainnet deploys are done by the human.
11. No irreversible step in one action. A destructive feature first needs a row in the
    owner-mistake table of docs/threat-model.md and its safeguard.
12. scripts/ never contains a canister deletion or reinstall mode.
13. Nothing about a deployed vault enters this repository: no canister ids, no .icp/ directory, no logs.

## Review
Changes in apps/web/src/crypto, apps/web/src/platform or scripts/, or in access.rs, unlock.rs,
sessions.rs, slots.rs, claim.rs, http.rs or upgrade.rs, need a fresh-context review
session and human approval before merge. Section 6 of docs/implementation-plan.md has the levels.
```

## Appendix B. Kickoff prompt for M0

```text
You are bootstrapping the repository ic-vault from VAULT-PLAN.md.
Do milestone M0 only.

1. Read VAULT-PLAN.md fully. Copy section 2 to docs/threat-model.md, section 6 to docs/crypto-spec.md and sections 11.4 to 11.6 to docs/operations.md. Copy IMPLEMENTATION-PLAN.md to docs/implementation-plan.md.
2. Run: npx skills add dfinity/icskills. Load the icp-cli, canister-security and stable-memory skills.
3. Create the layout from section 10 with empty modules, AGENTS.md from Appendix A, and CLAUDE.md containing @AGENTS.md.
4. Pin toolchains: Rust, Node, pnpm, icp-cli. Verify every version named in section 8.4 against current docs and record the pins in docs/adr/0001-toolchain.md.
5. Write icp.yaml with one Rust canister named vault and a local network with Internet Identity enabled.
6. Implement a hello canister that serves one certified HTML page with the headers from section 7.6.
7. Write scripts/setup.sh, dev.sh and check.sh, and wire them to pnpm.
8. Add .icp/ to .gitignore, set minimumReleaseAge to 7 days in pnpm-workspace.yaml, and add a CI workflow that runs pnpm check on every pull request.
9. Ask the human to confirm decision D8, the gateway domain, and record the answer in docs/adr/0002-gateway-domain.md.
10. Done when: icp network start -d and icp deploy succeed, the page loads from the local canister, pnpm check passes locally and in CI.

Stop after M0. List every VERIFY item you resolved and every one you could not.
```

## Appendix C. Command cheat sheet

```bash
# Tools and local development
npm install -g @icp-sdk/icp-cli @icp-sdk/ic-wasm
npx skills add dfinity/icskills
icp network start -d
icp deploy

# Identities and funding
icp identity new vault-staging --storage keyring
icp identity new vault-prod --storage password
icp identity account-id
icp token balance -n ic
icp cycles mint --icp 5 -n ic
icp cycles balance -n ic

# Read-only check, safe against any canister
icp canister status <canister-id> -n ic --public

# What the scripts run on mainnet, shown for understanding. Never type these against production
icp canister create vault -e staging --cycles 4T --subnet re2t4-faa75-v3vhk-kdmdr-uyrkl-aik2l-ixd6u-p3fyr-zlfkc-6c5af-zae
icp canister install vault -e staging --mode install
icp canister install vault -e staging --mode upgrade
icp canister snapshot create vault -e staging
icp canister top-up vault --amount 1T -e staging

# Mainnet changes go through the scripts of section 9.6
scripts/deploy.sh production
scripts/doctor.sh production
scripts/upgrade.sh production
scripts/topup.sh production 1T
scripts/watch.sh production
```

## Appendix D. Alternatives considered

| Alternative | Why not |
|---|---|
| Official EncryptedMaps library as the whole backend | Fastest path and the documented pattern, but it gives one lock only: whoever obtains the vetKey and the ciphertext reads everything. No versions, files, lockout or audit log. Worth revisiting for sharing. |
| One shared canister for all users | Simple onboarding, but its controller could upgrade the code and hand out network keys. It is also one large target. |
| No Internet Identity, the vault passkey is the identity | One prompt less, but it removes an independent gate and needs custom WebAuthn signature handling. |
| Public-key recipients per device | Allows re-keying without the other devices online, but puts public-key cryptography in front of the vault key and loses quantum tolerance at rest. |
| Standard asset canister plus a separate backend canister | Familiar, but two hashes to verify and two canisters to control. |
| Master password only | No hardware factor, and offline guessing becomes possible for anyone who gets the wrapped key. |
| Immutable canister with no controllers | Strongest against upgrades, but a single bug would be permanent. |
| Network key derived on every unlock | Slightly stronger revocation, but each unlock costs about 0.036 USD and several seconds. Kept as option D5. |
| Kit as 52 base32 characters | Shorter, but no word-by-word validation and hard to copy by hand. Replaced by 24 words in revision 2. |
| Kit split into shares, 2 of 3 | Covers loss and theft at once, but adds a secret-sharing primitive and three papers to manage. A later option. |
| No waiting period for the kit | Simplest recovery, but one photographed sheet opens the vault at once. |
| Reminders by email or push | Would reach an owner who never opens the vault, but needs an outside service and a stored address. |
| Hard block on removing the last device key | Safe, but it stops an owner from cutting off a stolen only device at once. A confirmation is used instead. |
| Keys bound to the canister id | One concept less, but a vault could then move only by decrypting and re-encrypting everything with a tool that acts as a client. Replaced by the vault id in revision 3. |
| A custom domain for the vault | An address under your own control, but DNS and a registrar account become new ways to hijack the page. |
| A canister that watches other vaults | No outside scheduler, but it needs cycles and upkeep of its own and cannot send mail. |
| Automatic dependency updates | Less monthly work, but a poisoned release would flow straight into a build. |
| A different key for staging | Keeps test and production keys apart, but the real vault would be the first user of the production key. |
| The network key in the first version | A second lock against a state copy plus a passkey secret, but the most complex layer for one narrow case. Deferred in revision 5. |
| One shared nonce for all unlock keys | Simpler, but a kit thief could keep the owner's unlocks failing during the waiting period. Replaced by one nonce per key in revision 5. |
| Full scope in the first release | More features sooner, but about a fifth more for a reviewer to read. The cut line was taken in revision 6. |
| A paid audit before real secrets | The right order, but there is no budget for it. A release gate and trust stages take its place in revision 7. |

## Appendix E. Prompt for the monthly upkeep task

```text
You are doing the monthly upkeep task for ic-vault. You have no mainnet credentials and need none.

1. Read docs/operations.md, especially the watch list, and AGENTS.md.
2. Look for published advisories that affect the pinned versions: pnpm audit, cargo deny, and the release notes
   of icp-cli, ic-cdk, @icp-sdk/core, @icp-sdk/auth, @noble/hashes, @noble/curves and react.
3. Check every row of the watch list against its source. Quote what changed, with the link.
4. Propose version bumps in one pull request. No version younger than 7 days, unless it fixes a published flaw.
   Bumps of crypto or SDK packages follow the review rule in AGENTS.md.
5. Run pnpm check and the release build. Report the new Wasm hash.
6. Write docs/upkeep/<year>-<month>.md: what you checked, what changed, and what the human has to do on mainnet.

Do not deploy. Do not change scripts/ or the crypto spec. Stop after the report.
```

## References

- vetKeys concept: https://docs.internetcomputer.org/concepts/vetkeys
- vetKeys skill: https://skills.internetcomputer.org/skills/vetkeys/
- vetKeys TypeScript API: https://dfinity.github.io/vetkeys/
- vetKeys repository: https://github.com/dfinity/vetkeys
- Canister security skill: https://skills.internetcomputer.org/skills/canister-security/
- Certified variables skill: https://skills.internetcomputer.org/skills/certified-variables/
- ICP skills index: https://skills.internetcomputer.org/
- icp-cli 1.6: https://cli.internetcomputer.org/1.6/
- Internet Identity guide: https://docs.internetcomputer.org/guides/authentication/internet-identity
- Internet Identity specification: https://docs.internetcomputer.org/references/internet-identity-spec
- Asset certification guide: https://docs.internetcomputer.org/guides/frontends/certification
- Management canister reference: https://docs.internetcomputer.org/references/management-canister
- Cycle costs: https://docs.internetcomputer.org/references/cycle-costs
- Resource limits: https://docs.internetcomputer.org/references/resource-limits
- Security concepts: https://docs.internetcomputer.org/concepts/security
- Node infrastructure and SEV-SNP: https://docs.internetcomputer.org/concepts/node-infrastructure/
- Confidential subnet on the dashboard: https://dashboard.internetcomputer.org/network/subnet/re2t4-faa75-v3vhk-kdmdr-uyrkl-aik2l-ixd6u-p3fyr-zlfkc-6c5af-zae
- WebAuthn PRF overview: https://www.corbado.com/blog/passkeys-prf-webauthn
- Prior art, Cipher Box: https://www.github.com/ahdrahees/cipher-box
- Prior art, Seed Vault: https://forum.dfinity.org/t/introducing-seed-vault-secure-on-chain-seed-phrase-storage-with-vetkeys-on-icp/61542
- SEV-SNP WebAssembly leakage research: https://arxiv.org/pdf/2512.14376
- BIP39 word encoding: https://github.com/bitcoin/bips/blob/master/bip-0039.mediawiki
- WebAuthn Level 3, exclude list and backup flags: https://www.w3.org/TR/webauthn-3/
- icp-cli identities: https://cli.internetcomputer.org/1.6/guides/managing-identities/
- Frozen canister report on the developer forum: https://forum.dfinity.org/t/cmc-canister-reject-topup-canister-frozen/27297
- icp-cli tokens and cycles: https://cli.internetcomputer.org/1.6/guides/tokens-and-cycles.md
- icp-cli deploying to mainnet: https://cli.internetcomputer.org/1.6/guides/deploying-to-mainnet.md
- icp-cli canister snapshots: https://cli.internetcomputer.org/1.6/guides/canister-snapshots.md
- icp-cli environments: https://cli.internetcomputer.org/1.6/guides/managing-environments.md
- Canister settings guide: https://docs.internetcomputer.org/guides/canister-management/settings.md
- Canister snapshots guide: https://docs.internetcomputer.org/guides/canister-management/snapshots.md
- Custom domains guide, with the default address: https://docs.internetcomputer.org/guides/frontends/custom-domains.md
- pnpm settings, minimumReleaseAge: https://pnpm.io/settings
- WebAuthn client capabilities: https://web.dev/articles/webauthn-client-capabilities
