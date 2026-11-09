# Threat model

> Copied from section 2 of `VAULT-PLAN.md`, revision 8, by task M0-02 on 2026-11-09. From then on this file is the source of truth. It changes only through an ADR in `docs/adr/`.
>
> Section numbers are those of `VAULT-PLAN.md`. Sections 2, 6 and 11.4 to 11.6 live in `docs/threat-model.md`, `docs/crypto-spec.md` and `docs/operations.md`. Any other section number means `VAULT-PLAN.md`.

## 2. Threat model

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
