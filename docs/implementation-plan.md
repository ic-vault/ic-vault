# Personal Vault: Implementation Plan

> Copied from `IMPLEMENTATION-PLAN.md` by task M0-02 on 2026-11-09. This is the living copy and the task tracker. The root file is frozen. Since the copy, this file changes only by ticks on finished tasks and by what an ADR in `docs/adr/` allows.
>
> Spec sections 2, 6 and 11.4 to 11.6 now live in `docs/threat-model.md`, `docs/crypto-spec.md` and `docs/operations.md`. A reference such as "spec 6.3" means those files.

Companion to `VAULT-PLAN.md`, revision 8. Written 2026-11-09. Nothing is built.

`VAULT-PLAN.md` says what the vault is and why. This document says in which order to build it, in which files, with which tests, and who has to look at what before it merges. Section 7 lists what still needs review, ranked.

## 0. How to use this document

- **You** read sections 1, 6 and 7, and decide the items marked **DECIDE**.
- **Agents** get one task from section 4 at a time, with the prompt of section 8. They read the sections of `VAULT-PLAN.md` that the task names, and section 3 of this document.
- **Section 3 is frozen at task M1-01.** After that a change to a type, an encoding or a limit needs an ADR and your approval, like the crypto spec.
- At bootstrap this file is copied to `docs/implementation-plan.md`. The task tables become the tracker: an agent ticks its row in the same pull request.
- "The spec" means `VAULT-PLAN.md`. Section numbers without a file name refer to this document.

**Fields of a task**

| Field | Meaning |
|---|---|
| ID | Milestone and number, for example M1-05. Spikes are SP1 to SP8 |
| Lane | A canister, B crypto worker, C tooling and docs, D web app, H human |
| Depends | Tasks that must be merged first |
| Proof | The tests that must exist and pass, or the result that can be observed |
| Review | R0 to R3, see section 6 |
| Size | S is one agent session. M is two or three. L is more, and the agent splits it into several pull requests |

**Rules for every task**

1. One task, one branch `task/<id>-<slug>`, one pull request.
2. Tests first in lanes A and B. The proof column is written as failing tests before the code.
3. An agent that finds this plan or the spec wrong stops, writes an ADR and waits. It does not work around the problem.
4. No pull request merges with a red gate. Nothing in lane H is ever done by an agent.
5. A task that needs a file owned by another lane asks for a task in that lane. It does not edit the file.

## 1. Build order at a glance

```
 SP3 SP8 ------+
               v
 M0 bootstrap ---> design review, DR-01 to DR-04
                        |
                        v
                   M1-01 freeze the interface
                        |
        +---------------+----------------+
        v               v                v
   Lane A canister  Lane B worker   Lane C tooling
   M1-02 .. M1-14   M2-01 .. M2-09  M4-02, M4-03
        |               |                |
        +-------+-------+                |
                v                        |
          M3 vertical slice              |
                |                        |
                v                        v
          M4 packaging and the staging vault
                |
                v
          M5 devices and recovery ---> M6 backup, offline tool
                                            |
                                            v
                    release gate, then the production vault ---> M7 sovereign upgrades

 SP1 runs right after SP3 and before M1-01. It decides whether the passphrase key stays out.
 SP2 SP5 SP6 run beside M1 and M2 and feed M1-05, M2-03 and M3-02.
```

**Lanes and file ownership**

| Lane | Owns | Working rule |
|---|---|---|
| A | `canisters/vault/` | One task at a time. The tasks share `state.rs` and `lib.rs` |
| B | `apps/web/src/crypto/`, `packages/vectors/` | One task at a time |
| C | `scripts/`, `docker/`, `.github/`, `ops-template/`, `docs/` | Independent of A and B |
| D | `apps/web/src/platform/`, `vault/`, `ui/`, `e2e/` | Starts at M3. Two tasks at once only on different screens |
| H | Mainnet, paper, your devices | You |

From M1-01 on, three cloud agents can work at the same time, one each in lanes A, B and C.

**Critical path.** M0, M1-01, M1-05, M2-03, M3-04, M4-01, M4-08, M5-03, M6-02, M6-04, release gate. A delay on any of these delays the first real secret in the vault.

**Rough size.** These are estimates, to be corrected after M1.

| Milestone | Tasks | Agent sessions | Your time |
|---|---|---|---|
| Spikes | 8 | 9 | 4 hours, mostly on your own devices |
| M0 | 6 | 9 | 2 hours |
| Design review | 4 | 1 | Done, except the package for the public call |
| M1 | 14 | 32 | 10 hours of review |
| M2 | 9 | 20 | 8 hours of review |
| M3 | 8 | 22 | 6 hours |
| M4 | 9 | 18 | 1 day, including the staging vault |
| M5 | 9 | 22 | 3 days, including the hallway test, the drill and the full rehearsal |
| M6 | 7 | 5 | Two hours |
| Release gate | 2 | 5, plus fixes | A day of your own, and 30 days of waiting |
| M7 | 6 | 14 | 1 day |

A session is counted as 1 for size S, 2.5 for M and 4 for L. Deferred tasks keep their row and count as zero: one in M2, four in M6, one in M7. That is about 136 sessions up to the release gate. Deferring the network key took about 7 away, and the cut line about 11 more.

With reviews answered within a day, the release gate is about nine to thirteen weeks away, plus the 30 days of the public call. That figure is a guess.

## 2. Spikes: answer the open questions first

A spike is throwaway code under `spikes/` that answers one question. It ends in an ADR, not in a feature. The spikes cover every **VERIFY** of the spec that could force a design change. SP3 and SP8 run inside M0, because a no there changes the packaging itself. SP1 follows SP3 at once: since the cut line, a no there brings the passphrase key back before the freeze. SP4 is deferred together with the network key.

| ID | Question | Method | Passes when | If it fails | Feeds |
|---|---|---|---|---|---|
| SP1 | Which of your devices give a PRF output, at creation or only at sign-in? What is the capability name? Does a passkey work on the real kind of address? | A test page served by the SP3 canister on mainnet. The address `<id>.icp.net` is itself on the public suffix list, so a page hosted elsewhere proves nothing. Create a passkey with PRF, read the output at creation and at sign-in. Run it on every device and browser you own, and in Playwright with a virtual authenticator. Run every passkey test at both `<id>.icp.net` and `<id>.icp0.io`, see ADR 0002. Note also whether the browser can scan a QR code by itself | A table of device, browser, authenticator and result exists, with a result for each address. Every device you want to use passes at the address of ADR 0002 | The passphrase key comes back into the first release before the freeze, by ADR, and is the default on the failing devices. If no device passes, stop and rethink gate 2. If the passkey fails at the address of ADR 0002 and passes at the other one, a new ADR changes decision D8 before M4-07 | M1-01, M2-03, M3-02, M3-07, decision D8 in ADR 0002 |
| SP2 | How long does Argon2id with 64 MiB and 3 passes take in the browser? | Benchmark `@noble/hashes` against a Wasm build on a desktop and a mid-range phone | A PIN unlock adds at most 2 seconds on the phone | Use the Wasm build by ADR, or lower the memory by ADR. Both change the page policy | M2-03 |
| SP3 | Does one Wasm that embeds the web app work on mainnet with the strict headers? | The M0 hello canister: React page with a worker, page policy with Trusted Types, a certified `config.json`, deployed locally and to a throwaway canister on the confidential subnet with the scripts-to-be | The page loads at `<id>.icp.net`. Internet Identity sign-in works in its popup. The Wasm installs, whatever its size. `icp canister status --public` shows the hash | Relax one header at a time by ADR, or split web app and API into two canisters. The second costs a hash to verify and is a spec change | M0-05, M4-01 |
| SP4 | Deferred with the network key, spec 6.5. The question was whether `key_1` answers a canister on the confidential subnet | Not run now | Nothing to pass | Nothing | The later release that adds the network key |
| SP5 | Does an Ed25519 check fit into `canister_inspect_message`? | PocketIC test that counts instructions | It stays under a tenth of the limit of that hook | Drop the early check. Wrong proofs on passkey and kit slots then cost the vault cycles, and need a rate limit instead | M1-05 |
| SP6 | Can the pinned SDK sign calls with a non-extractable session key, with two agents in one page? | A page that signs in, creates a session key, and calls the hello canister as both identities | Both callers arrive as expected. The session key cannot be exported | An extractable key held in the worker instead of the UI thread | M3-01 |
| SP7 | Can a canister that controls itself install code on itself? | Local network only. Belongs to M7 | An upgrade completes and the canister answers afterwards | A small upgrader canister per vault | M7-01 |
| SP8 | Do end-to-end tests run in CI with the local Internet Identity and a virtual authenticator? | Playwright against the local network, signing in and calling the hello canister | Green in CI, three runs in a row | End-to-end tests run locally before each merge, and CI runs the rest | M0-06, M3-08 |

## 3. Frozen definitions

The spec sketches the interface and the state. This section makes them exact enough that lanes A, B and D can work without waiting for each other. Task M1-01 turns it into `vault.did`, Rust types and generated TypeScript, and freezes it.

### 3.1 Limits and constants

One file, `packages/shared/limits.json`, is the source. A build step generates `limits.rs` and `limits.ts` from it, and a test fails when they drift. All times are nanoseconds of the canister clock. All integers in keys, labels and associated data are big endian.

| Name | Value | Enforced by |
|---|---|---|
| Header, padded plaintext | A multiple of 256 bytes, at most 3,840 | Worker pads, canister checks the ciphertext length |
| Body, padded plaintext | 1, 2, 4, 8, 16, 32, 64, 128 or 256 KiB | Same |
| Ciphertext overhead | 28 bytes: nonce 12, tag 16 | Both |
| File chunk, after the first release | 1 MiB of plaintext, 50 chunks at most | Both |
| Items, including trash and drafts | 10,000 | Canister |
| Vault quota | 1 GiB, fixed in the first release | Canister |
| Unlock keys | 16 | Canister |
| Sessions | 5 full, 1 enrolment | Canister |
| Session lifetime | 30 minutes idle, 8 hours at most | Canister |
| Wrong tries | 5 in a row, then 1 minute, doubling to 24 hours. Ten within thirty days across all keys, then 24 hours for each further one. After 30 that nobody cleared, locked until a full session clears it | Canister |
| Unlock nonce | One per unlock key | Canister |
| Handoff lifetime | 10 minutes, single use | Canister |
| Trash, tombstone | 30 days, 90 days | Canister, at unlock and at sync |
| Waiting period | 48 hours by default, 24 hours to 30 days | Canister |
| Activity log | The last 10,000 events | Canister |
| Alert log | 200 entries, each kept until acknowledged | Canister |
| Pending recovery | Lapses 7 days after it became ready | Canister |
| Retired credential ids | The last 64 | Canister |
| Slot kind in key labels | One byte: 1 passkey, 2 passkey with PIN, 3 passphrase, 4 kit. The first release uses 2 and 4 | Worker |
| Sync page | 500 records or 1.5 MiB, whichever comes first | Canister |
| Health | `Low` under 365 days until freezing, `Urgent` under 90 | Canister |
| Argon2id | 64 MiB, 3 passes, 1 lane, until SP2 says otherwise. The client refuses anything below the minimum of its release | Worker, stored per slot |

### 3.2 Interface

```candid
// vault.did, draft. Frozen by task M1-01. HttpRequest and HttpResponse are those of ic-http-certification.

type InstallArgs = variant {
  Init : record {
    claim_hash : blob;            // SHA-256 of the claim code
    host_suffix : text;           // "icp.net" on mainnet
    environment : text;           // "production", "staging" or "local"
  };
  Upgrade : record {};            // empty in the first release
};

type Error = variant {
  NotAllowed;                     // the caller fails the gate of this method
  WrongState : text;              // for example "setup done"
  NotFound;
  Locked : opt nat64;             // locked until this time. Nothing means: until a full session clears it
  BadProof : record { tries_left : nat8; settings_hash : blob };   // hash of the slot settings the canister holds
  StaleNonce : nat64;             // the current nonce of that slot, try again with it
  RecoveryPending : nat64;        // the kit principal may unlock at this time
  VersionConflict : nat64;        // the current version of the item
  TooLarge;
  QuotaExceeded;
  LimitReached : text;
  RateLimited : nat64;            // try again at this time
  Invalid : text;                 // a fixed text, never built from arguments
};
type Result = variant { Ok; Err : Error };

type SlotKind = variant { PasskeyPin; Kit };   // Passkey and Passphrase come after the first release, label bytes 1 and 3 stay reserved
type Kdf = record { memory_kib : nat32; passes : nat32; lanes : nat32; salt : blob };
type Slot = record {
  id : blob;                      // 16 bytes
  kind : SlotKind;
  credential_id : opt blob;       // passkey kinds
  prf_salt : opt blob;            // passkey kinds, 32 random bytes
  kdf : opt Kdf;                  // the kind with a PIN
  salt : blob;                    // 32 bytes
  auth_f_pub : opt blob;          // passkey with PIN: proof key of the passkey alone
  auth_pub : blob;                // proof key of the whole factor
  wrap : nat8;                    // 1 is the inner wrap alone. Room for the network key of spec 6.5
  wrapped : blob;                 // the slot blob of spec 6.3
  label : blob;                   // encrypted with k_meta
};
type SlotInfo = record {
  id : blob; kind : SlotKind; credential_id : opt blob; prf_salt : opt blob; kdf : opt Kdf; salt : blob;
  nonce : nat64; created : nat64; last_used : opt nat64;
  locked_until : opt nat64; needs_clearing : bool;
};
type Health = variant { Ok; Low; Urgent; Unknown };
type SlotsPublic = record {
  setup_done : bool;
  slots : vec SlotInfo;
  vk_epoch : nat32;
  kit_number : nat32;
  kit_created : nat64;
  pending_recovery : opt nat64;
  health : Health;
};

type UnlockArgs = record {
  slot_id : blob;
  nonce : nat64;                  // the nonce of that slot
  session : principal;
  proof_f : opt blob;             // passkey with PIN: made with the passkey alone, checked in the message hook
  proof : blob;                   // made with the whole factor, counted when wrong
};
type AuditEvent = record { seq : nat64; time : nat64; kind : text; caller : principal; slot_id : opt blob };
type UnlockOk = record {
  wrapped : blob;
  session_expires : nat64;
  previous_unlock : opt nat64;
  failed_since : nat32;           // failed tries on any key since the previous unlock
  alerts : vec AuditEvent;        // events newer than the last acknowledged one
};
type UnlockResult = variant { Ok : UnlockOk; Err : Error };

type SetupArgs = record {
  device_slot : Slot; kit_slot : Slot; kit_principal : principal;
  export_header : blob; vault_auth_pub : blob; session : principal;
};
type KitReplace = record { kit_slot : Slot; kit_principal : principal; export_header : blob };

type Handoff = record { id : blob; token_hash : blob; wrapped : blob };
type EnrollArgs = record { token : blob; session : principal };
type EnrollOk = record { handoff_id : blob; wrapped : blob; vk_epoch : nat32 };
type EnrollResult = variant { Ok : EnrollOk; Err : Error };

type ItemPut = record {
  id : blob;
  expected_version : nat64;       // 0 creates the item
  wrapped_dek : blob;
  header : blob;
  body : blob;
};
type ItemPutResult = variant { Ok : record { version : nat64; seq : nat64 }; Err : Error };
type ItemRecord = record {
  id : blob; version : nat64; seq : nat64; updated : nat64;
  deleted : opt nat64;            // set while the item is in the trash
  purged : bool;                  // true for a tombstone, the two fields below are then empty
  wrapped_dek : blob; header : blob;
};
type SyncPage = record {
  items : vec ItemRecord;
  next : opt nat64;               // pass it back to get the next page
  latest_seq : nat64;
  quota_used : nat64;
  quota : nat64;
};
type Body = record { version : nat64; body : blob };

type RekeyArgs = record {
  expected_seq : nat64;           // fails when anything changed since
  device_slot : Slot; kit_slot : Slot; kit_principal : principal; export_header : blob; vault_auth_pub : blob;
  wrapped_deks : vec record { blob; nat64; blob };   // item id, version, new wrapped data key
};

type BackupManifest = record {
  vault_id : blob; vk_epoch : nat32; kit_number : nat32; kit_created : nat64;
  kit_slot : Slot; kit_principal : principal; export_header : blob; vault_auth_pub : blob;
  latest_seq : nat64; item_count : nat32;
};
type Status = record {
  version : text; environment : text; vault_id : blob; health : Health;
  kit_number : nat32; kit_created : nat64;
};
type TrustInfo = record {
  balance : nat; liquid : nat; burn_per_day : opt nat;
  quota_used : nat64; quota : nat64; item_count : nat32;
  recovery_delay : nat64; sessions : nat8;
};

service : (InstallArgs) -> {
  // open to anyone
  http_request    : (HttpRequest) -> (HttpResponse) query;
  status          : () -> (Status) query;

  // owner principal: the Internet Identity principal or the kit principal
  claim           : (blob) -> (Result);
  setup_finish    : (SetupArgs) -> (Result);
  slots_public    : () -> (SlotsPublic) query;
  unlock          : (UnlockArgs) -> (UnlockResult);
  enroll_begin    : (EnrollArgs) -> (EnrollResult);

  // enrolment session
  enroll_finish   : (Slot, blob) -> (Result);           // the slot, and a proof of the vault key

  // full session
  renew           : () -> (nat64);
  lock            : () -> ();
  sync            : (nat64) -> (SyncPage);
  item_put        : (ItemPut) -> (ItemPutResult);
  item_delete     : (blob, nat64) -> (Result);
  item_restore    : (blob, nat64) -> (Result);
  item_purge      : (blob) -> (Result);
  item_body       : (blob) -> (opt Body) query;
  slot_add        : (Slot, opt blob) -> (Result);
  slot_remove     : (blob) -> (Result);
  slot_clear_lock : (blob) -> (Result);
  handoff_put     : (Handoff) -> (Result);
  kit_replace     : (KitReplace) -> (Result);
  recovery_cancel : () -> (Result);
  owner_set       : (principal) -> (Result);
  rekey           : (RekeyArgs) -> (Result);
  backup_manifest : () -> (BackupManifest);
  alerts_ack      : (nat64) -> (Result);
  limits_set      : (nat64) -> (Result);                   // waiting period
  audit           : (nat64, nat32) -> (vec AuditEvent) query;
  trust           : () -> (TrustInfo) query;
}
```

**Which answers can be trusted.** An update call is answered by the subnet as a whole. A query is answered by one node. So everything a decision hangs on comes from an update call: the unlock result with its alerts, `sync`, `backup_manifest`. The queries are either bound to a key by their associated data, which is true for `item_body`, or they are hints. `slots_public`, `audit` and `trust` are hints: a wrong answer makes an unlock fail or a screen show stale numbers, and nothing else.

**Refusals without a reason.** A call that `canister_inspect_message` turns away comes back without any `Error` value. The client then reads `slots_public` again and decides from that: a moved nonce, a lock, or a missing session.

### 3.3 State

| Memory | Structure | Key | Value |
|---|---|---|---|
| 0 | Cell | none | `Config`, with the schema version as its first field |
| 1 | Map | Slot id, 16 bytes | Slot, its nonce, created, last use, failed tries in a row, failed tries that nobody cleared, lock level, locked until |
| 2 | Map | Item id, 16 bytes | `ItemRecord` without the body |
| 3 | Map | Item id, 16 bytes | Body ciphertext |
| 4 | Reserved | none | File chunks, after the first release |
| 5 | Map | Event number | `AuditEvent`. The activity log |
| 6 | Map | Handoff id, 16 bytes | `Handoff` and its expiry |
| 7 | Map | Change number | Item id. This is the index that `sync` pages through |
| 8 | Map | Due time, then kind, then id | Empty. This is the index of what has to be removed, and when |
| 9 | Map | Event number | `AuditEvent`. The alert log, kept until acknowledged |

- **`Config` holds:** schema version, vault id, environment name, host suffix, Internet Identity owner, kit principal, claim hash, setup flag, vault key epoch, change number, event number, last acknowledged event, kit number and date, export header, waiting period, pending recovery, quota and quota used, item count, the times of the last ten wrong tries in the vault, the last eight balance samples, taken at an unlock and at most once a day, time of the last unlock, the vault proof key, the retired credential ids.
- **Values are Candid-encoded records** with an unbounded size. Keys are fixed-size bytes so the maps sort correctly.
- **The heap holds** the sessions and the asset router. The router is rebuilt in `init` and `post_upgrade`. An upgrade ends every session.
- **The expiry index** carries three kinds of entries: empty the trash for one item, drop one tombstone, drop one handoff. There is no timer. `unlock` and `sync` work off at most 100 due entries each time they run, and a handoff is also checked when it is used.

### 3.4 Who may call what

This table is the content of the authorization matrix test. A method without a row fails the build.

| Method | Anonymous | Stranger | Owner principal | Enrolment session | Full session |
|---|---|---|---|---|---|
| `http_request`, `status` | yes | yes | yes | yes | yes |
| `claim` | no | Once, with the claim code | no | no | no |
| `setup_finish` | no | no | Before setup is complete, Internet Identity owner only | no | no |
| `slots_public` | no | no | yes | no | no |
| `unlock` | no | no | After setup. The kit principal may try the kit key only, and waits out the waiting period | no | no |
| `enroll_begin` | no | no | After setup, Internet Identity owner only, with a live handoff token | no | no |
| `enroll_finish` | no | no | no | yes, once, with a proof of the vault key | no |
| Every method of the full-session block | no | no | no | no | yes |

- The owner principal is never a session. An owner who wants to read an item unlocks first.
- The matrix test runs twice: as ingress, and through a proxy canister. A call from another canister skips the message hook, so only the second run proves that the gate sits inside the method.
- `canister_inspect_message` repeats the caller column and nothing else, with the additions of spec 7.2. Every proof that rests on 256 bits is checked: the passkey proof of a passkey key, and the proof of the kit key. A locked key and a stale nonce are refused. The kit principal is refused on every key but the kit key, and altogether while a recovery is pending. The token of `enroll_begin` is checked. Every ingress call to a query method is refused.
- A proof that contains a PIN or a passphrase is never checked in the hook. In the method the failed try is written first, then the proof is checked, then the try is taken back on success.
- `slot_remove` ends the sessions opened through that key. `rekey` ends every session except its own.
- `slot_add` and `enroll_finish` check the slot itself: no kit kind, never the kit slot as the one to replace, no retired credential id, fields that fit the kind, and proof keys that are valid points of full order.
- A full session is only ever held by someone who can decrypt the vault key. An unlock proves a factor, and an enrolment proves the vault key itself.

### 3.5 Worker protocol

The UI thread and the crypto worker exchange typed messages. Buffers are transferred, not copied. This table is the whole surface of `crypto/`.

| Message | In | Out |
|---|---|---|
| `init` | Config of 3.9 | Nothing |
| `kit.create` | Kit number | The 24 words and the check code, for the sheet |
| `kit.confirm` | The words as typed | Yes, or the position of the first wrong word |
| `setup.build` | `F`, PIN, credential id, session principal | `SetupArgs` |
| `unlock.prove` | `SlotInfo`, `F`, PIN, nonce, caller principal, session principal | The proofs: two for a passkey key with a PIN, one otherwise |
| `unlock.open` | Slot blob | Nothing. The vault key now lives in the worker |
| `headers.open` | Item records | Decrypted headers |
| `body.open` | Item record and body ciphertext | The decrypted body |
| `item.seal` | Id, new version, header, body | The ciphertext fields of `ItemPut` |
| `slot.build` | `F`, PIN, kind, credential id | A `Slot` |
| `handoff.create` | Nothing | A `Handoff` record and the code to show |
| `handoff.open` | Code as typed or scanned, handoff blob | Nothing. The vault key now lives in the worker |
| `enroll.prove` | Handoff id, session principal | A signature made with the vault proof key |
| `kit.replace` | Nothing | New words and check code, then `KitReplace` after `kit.confirm` |
| `kit.check` | Check code as typed | Yes or no |
| `rekey.build` | `F` of the new passkey, the new PIN, all item records, session principal | New words and check code, then `RekeyArgs` after `kit.confirm` |
| `backup.mac` | The bytes of a backup file without its last record | The MAC that ends the file |
| `backup.verify` | Records read back from a file, and its MAC | A report per record, and whether the MAC fits |
| `lock` | Nothing | Nothing. Keys are overwritten and the worker ends |

Three kinds of plaintext cross to the UI thread, because a person has to see them: kit words and check code, the handoff code, and item contents. Nothing else does.

`kit.replace`, `handoff.create` and `rekey.build` are as good as a key export. The worker answers them only after a fresh passkey answer that opens the current slot. A session that was opened with the kit is exempt. The worker is the one place for key handling that a reviewer has to read. It does not stop a script in the page from asking it to use the keys.

### 3.6 States of the web app

| State | Entered when | Leaves to |
|---|---|---|
| Boot | Page load. Reads `config.json`, checks the origin | Wrong origin, Signed out |
| Wrong origin | The host is not an accepted host | Nowhere. Shows the right address |
| Signed out | No valid Internet Identity delegation | Not owner, Setup, Locked |
| Not owner | `slots_public` is refused | Signed out. Says that this identity does not own the vault |
| Setup | Owner, setup not complete | Locked, after `setup_finish` |
| Locked | Setup complete, no session | Unlocking, Recovery pending |
| Recovery pending | The kit principal is waiting | Locked, Unlocking |
| Unlocking | Factor entered | Unlocked, Locked with an error |
| Alerts | The unlock result carried alerts | Unlocked, after `alerts_ack` |
| Unlocked | Session live, worker holds the vault key | Locking |
| Locking | Idle, sleep, button, tab close | Locked |

### 3.7 Plaintext formats

All plaintext is UTF-8 JSON with a format number, so later versions can read older items.

```text
header  { "v": 1, "t": "login" | "note" | "card" | "identity" | "file" | "draft" | "settings",
          "title": text, "sub": text, "tags": [text], "fav": bool }

body    { "v": 1,
          "fields":  [ { "k": text, "v": text, "secret": bool } ],
          "notes":   text,
          "urls":    [text],
          "history": [ { "k": text, "v": text, "at": time } ],     last 10 changes
          "files":   [ { "id": hex, "name": text, "size": number, "mime": text, "key": base64, "chunks": number } ],
          "draft_of": hex or absent,                                for drafts
          "import_batch": hex or absent }

settings body
        { "v": 1, "kit_check_code": text, "kit_checked_at": time, "backup_at": time,
          "synced_passkey_confirmed": bool, "auto_lock_minutes": number, "profile": "strict" | "standard" | "fallback" }
```

The settings live in one reserved item. Its id is the first 16 bytes of SHA-256 of `ic-vault/v1/settings`. It is versioned and synced like every other item.

Reserved, and unused in the first release: the type `file`, the list `files`, the field `import_batch`, and the profiles `standard` and `fallback`. The worker refuses an item that uses one of them.

### 3.8 Backup file

```text
line 1      "ICVAULT-BACKUP 1"
then        records, each:  type (1 byte) || length (4 bytes, big endian) || Candid bytes
            type 1  BackupManifest          exactly once, first
            type 2  ItemRecord and body     one per item, trash included, tombstones left out
            type 3  reserved for file chunks, after the first release
            type 9  HMAC-SHA-256 of all bytes before this record, under a key derived from the vault key, last
```

The file name is `vault-<first five characters of the vault id>-<date>.icvb`. The file contains ciphertext and public values only.

### 3.9 The config file

The canister builds `/config.json` in `init` and in `post_upgrade`, and certifies it like every other asset.

```text
{ "canister_id": text, "vault_id": text, "hosts": [text], "environment": text,
  "version": text, "api_host": "https://icp-api.io" }
```

The app refuses to run when `location.host` is not in `hosts`, or when the first label of the host is not `canister_id`.

### 3.10 Corrections to the spec

Writing this section, and two independent reviews, showed gaps and slips in the spec. C1 to C13 come from the writing, C14 to C23 from the feasibility review, C24 to C36 from the cryptography review. C37 to C47 come from the author's own review of 2026-11-09 and from your decisions on it. C48 is the cut line, which you took the same day. C49 puts a release gate in the place of the audit, because there is no budget for one. The spec carries all of them from revision 7 on, so both documents agree.

| # | Correction | Why |
|---|---|---|
| C1 | Two counters get two names. The unlock nonce only grows. The failed tries of a slot are reset by a success | The spec said "resets the counter" in a way that could be read as resetting the nonce, which would allow replays |
| C2 | Whatever a decision hangs on is answered by an update call: alerts and the previous unlock come with the unlock result, and `backup_manifest` replaces the `export_header` query | A single node answers queries and could hide a pending recovery or hand out a broken export header |
| C3 | New method `alerts_ack` | Alerts need a stored mark of what was acknowledged |
| C4 | Settings are a reserved item. `settings_get` and `settings_put` are gone | One mechanism less, and settings can no longer be rolled back by one node |
| C5 | New method `limits_set`. Its quota half waits with C48 | The spec lets the owner change quota and waiting period but named no method for it |
| C6 | At most 10,000 items, and sync pages of 500 | A re-key has to fit into one call, and a reply has a size limit |
| C7 | Two more memories: the change index and the expiry index | `sync` and the expiry work need them to stay bounded |
| C8 | `canister_inspect_message` checks the token of `enroll_begin` | Otherwise a stolen Internet Identity could burn cycles with wrong tokens |
| C9 | The canister id, vault id and hosts are served as a certified `config.json` instead of being written into the index page | Simpler than templating HTML, and the page policy forbids inline data anyway |
| C10 | The UI thread holds all decrypted headers and the body of one item | A list needs titles. The spec said one item only |
| C11 | The Trust screen reads module hash and controllers from the public state tree in the browser. The canister reports balance and health only | A canister cannot read its own module hash in a query |
| C12 | `item_body` returns the version with the body | The client then knows which header belongs to it |
| C13 | The web app uses hash routes | One certified page, no fallback rules |
| C14 | No production vault before the release gate. M4 creates the staging vault, the full rehearsal of spec 11.5 runs at the end of M5, and the production vault is created from the release that passed the gate | The rehearsal needs recovery and a second unlock key, which M5 builds. A real vault should start on code that passed the gate |
| C15 | The outer wrap of a device slot uses the AEAD layout of spec 6.1 with an AES key derived from the vetKey | The message format of the vetKeys library is not documented, and a library update could make stored slots unreadable |
| C16 | The rule about WebCrypto is restated: vault key material lives only in `crypto/`. `platform/` uses WebCrypto through the SDK for identities | The session identity and the sign-in need WebCrypto on the UI thread |
| C17 | Only raw domains are redirected. Any other host gets the same certified page, and the app refuses to run there | A response that depends on the host cannot be certified with plain asset certification |
| C18 | No timers in the first version. Expiry is worked off at unlock and at sync, and the balance is sampled at unlock | Nothing to re-arm after an upgrade, and no cycles burned while nobody uses the vault |
| C19 | Every integer is big endian, and the encryption of slot labels is defined | The spec left both open |
| C20 | The page policy allows passkeys and clipboard writes for the vault itself, and names one Trusted Types policy for starting the worker | "Everything else off" would have blocked passkeys, and starting a worker is a Trusted Types sink |
| C21 | Size limits are stated for the padded plaintext. The ciphertext is 28 bytes longer | A full body would have exceeded its own limit |
| C22 | A rule for refusals that come without a reason. An error `Unavailable` came with it and went again with the network key | A refusal by the message hook was not covered |
| C23 | The idle logout of the Internet Identity client is switched off | By default it signs out and reloads the page after ten idle minutes |
| C24 | The claim of spec section 1 is restated. It covers whoever can read, not whoever can change the code the browser runs | The gateway, a majority of the subnet and a stage A controller can serve altered code, and altered code reads everything at the next unlock |
| C25 | The client refuses Argon2id parameters below a built-in minimum | The parameters arrive in an answer from a single node |
| C26 | Passphrases are always generated. A self-chosen one is not accepted | A passphrase key has no hardware secret, so a guessable passphrase can be tested offline against a state copy |
| C27 | A re-key creates a new passkey and asks for a new PIN. Old credential ids are retired and refused. The PRF salt is random per slot, and the slot kind is part of the key labels | The old flow kept the passkey, so a lost device with a synced passkey stayed in. It also could not have built a PIN slot, because the PIN was gone |
| C28 | An enrolment proves knowledge of the vault key with a key derived from it. Only then does the canister release the network key and promote the session. A handoff begins once, and only for the Internet Identity owner. Since the network key is deferred, the proof travels with `enroll_finish` | Internet Identity plus an observed token had been enough for a full session, which can destroy a vault without reading it |
| C29 | A second limit on wrong tries: ten within thirty days, whatever happens in between. After a lock expires there is one try | Four guesses after each successful unlock would never have triggered a lock |
| C30 | A backup file ends with a MAC under a key derived from the vault key. A restore is confirmed only after the first unlock has checked it | A tampered file could have dropped or rolled back items unnoticed |
| C31 | A pending recovery belongs to one kit number, is used up by one unlock, lapses after 7 days, and is cleared by cancel, kit replacement and re-key | Its lifetime was undefined. A recovery left over from a drill would have let a later thief skip the wait |
| C32 | The message hook refuses ingress calls to query methods, and an unlock with a stale nonce | Both would have let anyone, or a stolen Internet Identity, burn cycles and fill the log |
| C33 | Two logs: an alert log that keeps what the owner must see, and an activity ring for the rest | A burst of noise could have pushed an important event out of one shared ring |
| C34 | `slot_add` and `enroll_finish` check the slot: no kit kind, no kit slot as a replace target, no retired credential, fields that fit the kind | The rules of spec 7.9 were stated but not enforced by any method |
| C35 | The relying party id of a passkey is the full host, and user verification is required at every use | With a parent domain as relying party, any page under it could have asked for the passkey secret |
| C36 | `status` carries the number and date of the current kit. `restore_put` writes items and file chunks only. Files are immutable. Both wait with C48 | Smaller gaps between flows and interface |
| C37 | The network key is deferred until after the first release. Every slot records its wrapping, so the key can be added later. Three methods, the transport keys and the key cache are gone. The canister calls no other canister | It protects one pair of thefts and is the most complex layer. Review finding F6, your decision |
| C38 | Each unlock key has its own nonce. The kit principal may try the kit key only, and the hook refuses it while a recovery is pending | With one shared nonce a kit thief could keep the owner's unlocks failing and the owner's PIN keys locked until the waiting period ended. Finding F1 |
| C39 | An unlock of a passkey key carries two proofs: one made with the passkey alone and checked in the hook, one made with passkey and PIN and counted | A stolen Internet Identity could spend the tries of a PIN key without holding its passkey. Finding F2 |
| C40 | The failed try is written first, then the proof is checked, then the try is taken back on success | A trap on the failure path would otherwise make wrong guesses free. Finding F3 |
| C41 | Removing an unlock key ends its sessions. A re-key ends every session except its own | A stale session held the old vault key and could save items nobody can read. Finding F4 |
| C42 | The limit of ten wrong tries in thirty days counts all keys of the vault together. After 30 wrong tries that nobody cleared, a key stays locked until a full session clears it | Limits per key multiplied the guesses, and one try a day never ended. Finding F5 |
| C43 | The unlock proof names the caller. A proof key must be a valid point of full order | Hygiene. Findings F8 and F9 |
| C44 | A new kit, a code for adding a device and a re-key need a fresh passkey answer inside the worker. The worker is described as a discipline, not a wall | Those three requests are as good as a key export. Finding F10 |
| C45 | The claim names all five parties that can change the code, and lists the pairs of thefts that are enough. The re-key screen says what a re-key cannot undo | Honesty. Findings F11 and F12 |
| C46 | A restore shows the dates of the file and stays marked in the alert log until the MAC was checked. A recovery is cancelled only together with a kit replacement or a re-key | An older genuine backup passes every check, and a cancelled recovery could start again at once. Findings F13 and F14 |
| C47 | The canister serves no file for alternative origins. A wrong proof is answered with a hash of the slot settings | Another origin must not share the principal, and the app should notice a node that lied. Findings F15 and F18 |
| C48 | The cut line. The first release has no files, no import, no unlock keys without a PIN or without a passkey, no restore and so no "Move a vault", no flow for a change of address and no second accepted host, no quota setting, no QR decoder library and no launcher. Five methods are gone and 29 remain. The formats keep the reserved fields | Less for an auditor to read, and nothing of it weakens the claim. Decision I2. It holds only if spike SP1 passes on every device you use |
| C49 | No paid audit. A release gate takes its place: a review by a model of another vendor, a public call that waits 30 days, and the label "Not independently reviewed". Trust grows in three stages, and the gate opens the second only | There is no budget for an audit. The gate is weaker, so the product says so, and what cannot be rotated stays out. Decision I4, spec D10 and D19, spec R26 |

## 4. Work packages

Paths under `canisters/vault/` are relative to `canisters/vault/src/`. Paths under `crypto/`, `platform/`, `vault/` and `ui/` are relative to `apps/web/src/`. "Spec" refers to sections of `VAULT-PLAN.md`.

### M0 Bootstrap

Goal: an empty project in which a certified page loads from a canister, locally and once on mainnet, with the gate and CI in place.

| ID | Task | Lane | Depends | Files | Proof | Review | Size |
|---|---|---|---|---|---|---|---|
| M0-01 ✓ | Root files of spec section 10, except `icp.yaml`, `Cargo.toml` and `deny.toml`, toolchain pins, license, a README that says experimental and not independently reviewed, a `SECURITY.md` that points to GitHub's private vulnerability reporting, ignore rules with `.icp/`, pnpm workspace with the 7-day release age | C | none | Root files | A clean clone installs with the frozen lockfile and without scripts | R1 | S |
| M0-02 ✓ | Documents: threat model, crypto spec, operations, this plan, `AGENTS.md`, `CLAUDE.md`, pinned skills, ADR 0001 on the toolchain, ADR 0002 on the gateway domain | C, H | M0-01 | `docs/`, `AGENTS.md` | The copies match the spec. You have signed ADR 0002, which is decision D8 | R1 | S |
| M0-03 | `setup.sh`, `dev.sh`, `check.sh`, wired to pnpm. The gate of 5.1 with its eight steps, still empty | C | M0-01 | `scripts/`, `package.json` | `pnpm check` runs all steps and passes | R2 | S |
| M0-04 | CI: the gate on every pull request, merge blocked when red, no credentials | C | M0-03 | `.github/workflows/check.yml` | A pull request with a failing test cannot merge | R1 | S |
| M0-05 | Hello canister, with the first `icp.yaml`, the Cargo workspace and the `cargo deny` policy: certified page with a worker, the final headers of spec 7.6 including the Trusted Types policy for the worker, `config.json`. Spike SP3 on a throwaway mainnet canister | A, D, H | M0-03 | `icp.yaml`, `Cargo.toml`, `deny.toml`, `http.rs`, `apps/web/` | Local: the page loads and a PocketIC test verifies the response. Mainnet: the pass conditions of SP3 | R2 | M |
| M0-06 | End-to-end harness with the local Internet Identity and a virtual authenticator, spike SP8. Tests load the page the canister serves, never the development server | D | M0-05 | `apps/web/e2e/` | A sign-in test is green in CI three times in a row | R1 | M |

### Design review

Goal: the design is read against the questions of `DESIGN-REVIEW-BRIEF.md` before any format is frozen. On 2026-11-09 you first chose an outside specialist and then had Claude do the review instead. Claude wrote the design, so this is a second pass by the author and not an independent review. See section 6.4.

| ID | Task | Lane | Depends | Files | Proof | Review | Size |
|---|---|---|---|---|---|---|---|
| DR-01 ✓ | Review package: the brief, both documents, and a map from each claim of spec section 1 to the mechanism that upholds it | C | M0-02 | `docs/review/` | A stranger can read the folder in the order of the brief. Every question of the brief points to a section that exists | R1 | S |
| DR-02 | Dropped on 2026-11-09. No outside reviewer is booked. The brief stays, in case you want one later | H | None | None | Nothing to prove | None | None |
| DR-03 | The review. Done on 2026-11-09 by Claude, against the eleven questions of the brief | Claude | None | `DESIGN-REVIEW-FINDINGS.md` | Eighteen findings, and a verdict for each question | None | Done |
| DR-04 | Triage. Done on 2026-11-09: you chose every proposed correction and deferred the network key. The spec went to revision 5 with it, and section 3 follows | C, H | DR-03 | The spec, this plan, `DESIGN-REVIEW-FINDINGS.md` | Corrections C37 to C47 in 3.10. The consistency checks pass | R2 | Done |

The triage is done. M1-01 now waits for M0 only.

### M1 Canister core

Goal: the whole canister of spec section 7, driven by tests with fixed keys. No web app. Spec 6.6, 7.1 to 7.5, 7.7 to 7.9.

| ID | Task | Lane | Depends | Files | Proof | Review | Size |
|---|---|---|---|---|---|---|---|
| M1-01 | Interface freeze: `vault.did` from 3.2, Rust types, `limits.json`, generated TypeScript bindings, the matrix of 3.4 as test data | A | M0-06, DR-04, SP1 | `vault.did`, `types.rs`, `packages/shared/`, `packages/declarations/` | Bindings generate. A test fails for a method without a matrix row. The Candid compatibility check runs in the gate | R2 | M |
| M1-02 | State of 3.3: memories, stored types, config with schema version, `init`, `post_upgrade`, refusal of newer data | A | M1-01 | `state.rs`, `upgrade.rs` | Round trips for every stored type. Upgrade from an empty and from a full state. Newer schema with older code is refused. Instruction count recorded | R2 | M |
| M1-03 | Gates, sessions on the heap, `canister_inspect_message`, the matrix test | A | M1-02 | `access.rs`, `sessions.rs`, `tests/authz.rs` | The matrix passes for every method, with stubs behind the gates, as ingress and through a proxy canister that skips the message hook. Session expiry, renewal and the cap of five. The hook refuses every ingress call to a query method | R2 | M |
| M1-04 | `claim`, `setup_finish` | A | M1-03 | `claim.rs` | Claim works once. Wrong code and second claim refused. Setup runs once, stores both slots and the vault proof key, and opens the first session | R2 | S |
| M1-05 | `unlock`: two proofs, a nonce per key, failed tries, the three limits, alerts in the result | A | M1-04, SP5 | `unlock.rs`, `slots.rs`, `tests/unlock.rs` | Valid proofs, wrong passkey proof refused in the hook and never counted, wrong PIN proof counted, stale nonce, replayed call, locked slot, doubling up to 24 hours. A trap forced after the check leaves nothing behind, and a wrong PIN is counted even when the log cannot be written. Ten wrong tries across two keys reach the vault limit, and after 30 the key stays locked until cleared. The hook never reveals whether a PIN was right | R2 | L |
| M1-06 | Waiting period, `recovery_cancel`, `owner_set` | A | M1-05 | `unlock.rs`, `access.rs`, `tests/recovery.rs` | With the PocketIC clock: pending, cancelled from a device session, pending again, open after the delay. Internet Identity plus kit opens at once. A ready recovery is used once, lapses after 7 days, and is cleared by kit replacement and re-key. The kit principal is refused on every other key, and in the hook while a recovery is pending. `owner_set` refuses the three forbidden principals | R2 | M |
| M1-07 | Items: put, body, trash, restore, purge, quota, item limit | A | M1-03 | `items.rs`, `tests/items.rs` | Create, update, version conflict, every size limit, quota, the limit of 10,000, delete, restore, purge, body with its version | R1 | M |
| M1-08 | `sync` with the change index and paging | A | M1-07 | `sync.rs` | Paging over 2,500 items. No item is missed when writes happen between pages. Tombstones. Page size limit | R1 | M |
| M1-09 | Expiry index, worked off at unlock and at sync | A | M1-07 | `limits.rs`, `items.rs` | With the clock: after day 30 the next unlock empties the trash, after day 90 the tombstone goes, at most 100 entries per call | R1 | S |
| M1-10 | Activity log, alert log, `alerts_ack` | A | M1-05 | `audit.rs` | Every event kind of spec 7.5 lands in the right log. A burst of failed unlocks pushes nothing out of the alert log. The unlock result respects the acknowledged mark | R1 | S |
| M1-11 | `slot_add` with replace, `slot_remove`, `slot_clear_lock`, `kit_replace`, `limits_set` | A | M1-05 | `slots.rs` | Add, replace in one call, remove. The kit slot cannot be removed. Limit of 16. `kit_replace` swaps slot, principal, number and header at once. The kit kind, the kit slot as a replace target, a retired credential id, a malformed proof key and fields that do not fit the kind are refused. Removing a key ends its sessions | R2 | M |
| M1-12 | `rekey` | A | M1-11 | `slots.rs`, `items.rs`, `tests/rekey.rs` | Every data key replaced, the epoch up, other slots gone and their credential ids retired, every other session ended. Fails when the change number moved. 10,000 items fit into one call | R2 | M |
| M1-13 | `handoff_put`, `enroll_begin`, `enroll_finish` | A | M1-11 | `sessions.rs`, `slots.rs` | Handoff stored and expired. A wrong token is refused in the message hook. A handoff begins once. Without a proof of the vault key there is no session | R2 | M |
| M1-14 | `status`, health from balance samples taken at unlock, `trust`, budgets | A | M1-09 | `lib.rs`, `limits.rs` | `status` carries no balance and no size. Health follows the samples. The budgets of 5.4 are recorded for a full vault | R1 | M |

Watch out in M1:

- **Write the failed try first.** In `unlock` the failed try is written, then the proof is checked, then the try is taken back on success. A test forces a trap after the check and shows that nothing is left behind and nothing was learned.
- **No `await` anywhere.** The first version calls no other canister, so every method is one atomic message.
- **Stored records grow by optional fields only.** That keeps every old snapshot readable.
- **The caller is read once, at the top.** After an `await` it is not read again.
- **Sync is at least once.** An item that changes while a client pages comes again on a later page. The client keeps the higher version.
- **One decision function for `unlock`.** The message hook and the method call the same pure function, so the two cannot drift apart.
- **Two proofs, two places.** The passkey proof is checked in the hook and again in the method. The proof that contains the PIN is checked in the method only.
- **Other canisters can call every method.** They skip the message hook. The gate inside the method is the real one, and it runs before anything else so that a refusal costs as little as possible.

### M2 Crypto core

Goal: spec section 6 as a worker with vectors. No network code. Spec 6.1 to 6.5, 8.1.

| ID | Task | Lane | Depends | Files | Proof | Review | Size |
|---|---|---|---|---|---|---|---|
| M2-01 | Foundations: bytes, `lp`, AEAD and HKDF wrappers, padding, vector format | B | M1-01 | `kdf.ts`, `padding.ts`, `types.ts` | Published vectors for HKDF and AES-GCM. `encoding.json`. Round trip and single-bit-flip properties | R2 | M |
| M2-02 | Kit: 24 words, kit derivations, check code, kit slot | B | M2-01 | `kit.ts`, `wordlist.ts` | The BIP39 test set. `kit.json`. Wrong word, wrong checksum, swapped words. Word list hash pinned | R2 | M |
| M2-03 | Slot keys with a PRF salt per slot, Argon2id, inner wrap, both unlock proofs, PIN rules | B | M2-01, SP1, SP2 | `slots.ts`, `proof.ts` | RFC vectors for Argon2id and Ed25519. `slot-keys.json`, `unlock-proof.json`, `slot-wrap.json` for the two kinds of the first release. A slot of another kind is refused. Every refused PIN pattern is refused. Parameters below the minimum are refused | R2 | M |
| M2-04 | Deferred with the network key, spec 6.5 | B | None | None | Nothing now | None | None |
| M2-05 | Items: formats of 3.7, header and body encryption, data key wrap | B | M2-01 | `items.ts` | `items.json` for every type. A wrong vault id, item id or version fails. Lengths inside one bucket look the same | R2 | M |
| M2-06 | Worker shell: protocol of 3.5, key store, lock, branded types, import lint | B | M2-02 to M2-05 | `worker.ts`, lint config | Every message of 3.5 has a test. After `lock` no key is reachable. `Plaintext` cannot be passed to `platform/` | R2 | M |
| M2-07 | Re-key and handoff builders, export header, vault proof, backup MAC | B | M2-06 | `slots.ts`, `kit.ts` | `handoff.json`, `export.json`. A re-key opens with the new kit and the new slot and not with the old ones. The enrolment proof and the MAC are pinned in the vector files | R2 | M |
| M2-08 | A Rust reference of every construction in spec 6.3, the core of the later offline tool. Interop with the canister | A, B | M2-03, M1-05 | `tools/cli/core/`, `packages/vectors/`, `canisters/vault/tests/` | Both implementations produce every vector file byte for byte. A slot and a proof built by the worker unlock a PocketIC canister | R2 | L |
| M2-09 | Coverage and remaining properties | B | M2-07 | Tests | Coverage of `crypto/` at 95 percent in the gate. The properties of spec section 12 | R1 | S |

Watch out in M2:

- **Passkey calls cannot run in a worker.** `F` is read on the UI thread and transferred, so the UI thread loses the buffer.
- **A PIN is a string before it is bytes.** Strings cannot be overwritten. The worker converts at once and keeps no reference. Section 7 lists this.
- **Library names come from the pinned skills**, not from memory.
- **Vectors use fixed nonces through a test-only hook.** The hook does not exist in the production bundle, and a bundle test proves it.

### M3 Vertical slice

Goal: claim, setup, unlock, logins and notes, on the local network. Spec 6.6, 8.1 to 8.3, parts of 8.5.

| ID | Task | Lane | Depends | Files | Proof | Review | Size |
|---|---|---|---|---|---|---|---|
| M3-01 | Platform: config and origin check, Internet Identity with its idle logout switched off, owner agent and session agent, typed actor | D | M0-06, M1-03, SP6 | `platform/identity.ts`, `agents.ts`, `vault-actor.ts` | A wrong host is refused. Owner and session calls arrive as the right callers. The actor accepts only `Ciphertext` | R2 | M |
| M3-02 | Platform: passkeys with PRF, exclude list, device check | D | SP1 | `platform/passkey.ts` | Create and read with the virtual authenticator. A second enrolment is refused. The relying party id is the full host, and user verification is asked at every use. The device check reports what the authenticator can do | R2 | M |
| M3-03 | State machine of 3.6 and the sync store | D | M3-01, M2-06 | `vault/machine.ts`, `sync.ts`, `store.ts` | Every transition has a test. Pages are applied in any order. A conflict becomes a copy | R1 | M |
| M3-04 | Setup screens: claim, device check, kit sheet, full kit entry, passkey and PIN, practice unlock | D | M3-02, M3-03, M1-04 | `ui/setup/` | A wrong word blocks. An abandoned setup leaves nothing behind. The practice unlock is required. The sheet prints alone | R2 | L |
| M3-05 | Unlock, lock, auto-lock, drafts | D | M3-04, M1-05 | `ui/unlock/`, `vault/machine.ts` | Tries left are shown. Lock on idle and on sleep. A draft is saved and offered again. The worker is gone after a lock | R2 | M |
| M3-06 | Items: list, view, edit, trash, for logins and notes | D | M3-05, M1-08, M2-05 | `ui/items/` | Add, view, edit with history, copy with clipboard clearing, delete, restore, delete forever | R1 | L |
| M3-07 | Test banner, the label "Not independently reviewed", first error screens, one module for all texts | D | M3-05 | `ui/`, `ui/text.ts` | The banner shows on every route of a vault that is not production. Setup cannot be finished before the label is acknowledged. Screens for wrong identity, missing passkey, locked key, missing PRF | R0 | S |
| M3-08 | End-to-end slice and canary | D | M3-06 | `e2e/` | The M3 scenarios of 5.3, in CI | R1 | M |

### M4 Packaging and the staging vault

Goal: one reproducible Wasm, the scripts, and a staging vault on mainnet that is created, verified, upgraded and watched. There is no production vault yet, see correction C14. Spec 7.6, 9, 11.4 to 11.6.

| ID | Task | Lane | Depends | Files | Proof | Review | Size |
|---|---|---|---|---|---|---|---|
| M4-01 | Packaging: embed the build, certify, headers, Trusted Types policy, redirects | A, D | M3-08, SP3 | `http.rs`, `vite.config.ts` | Every response verifies against the certified data. No policy violation in the end-to-end run. Raw hosts redirect, and any other host shows the wrong-origin screen. The path for alternative origins answers not found | R2 | L |
| M4-02 | Reproducible build: container, script, release build in CI, archived inputs, monthly rebuild | C | M0-04 | `docker/`, `build-release.sh`, `.github/workflows/` | Two clean builds give one hash, locally and in CI | R2 | M |
| M4-03 | Scripts of spec 9.6 and 11.6 and the deployment template | C | M4-02 | `scripts/`, `ops-template/` | On the local network: a wrong setting and a low balance stop `deploy.sh`. `doctor.sh` names each deviation. `watch.sh` fails on each of its four causes. No deletion and no reinstall anywhere | R2 | L |
| M4-04 | Trust screen and cycles display | D | M4-01 | `ui/trust/` | Module hash and controllers are read from the public state tree. Health and the top-up command are shown | R1 | S |
| M4-05 | Runbooks: setup day, top-up, upgrade, recovery, hot-fix, release | C | M4-03 | `docs/runbooks/` | Each was carried out once on the local network by an agent that had the runbook and nothing else | R1 | M |
| M4-06 | Security pass 1 | All | M4-01 to M4-05 | None | One review pass per adversary of spec section 2. Each finding is fixed, or accepted by you in writing | R2 | M |
| M4-07 | Staging vault: create, verify, first run and one upgrade, as far as spec 11.5 goes with what is built | H | M4-06 | Private directory | `doctor.sh` and `verify.sh` report healthy. The three open questions of spec 11.4 are answered. The budgets of 5.4 are measured | You | Half a day |
| M4-08 | Watchdog and upkeep on the staging vault | H | M4-07 | Private directory | The watchdog runs daily and has caught a deliberately wrong hash. The log has its first lines | You | Two hours |
| M4-09 | First monthly upkeep task, with the prompt of spec Appendix E | C, H | M4-08 | `docs/upkeep/` | The report exists and you have read it | R1 | S |

### M5 Devices and recovery

Goal: every flow of spec 6.6 that belongs to the first release, except backup. Spec 6.6, 8.5.

| ID | Task | Lane | Depends | Files | Proof | Review | Size |
|---|---|---|---|---|---|---|---|
| M5-01 | Add a device: handoff, code shown and scanned or typed, shortcut for a synced passkey | D | M4-08, M1-13, M2-07 | `ui/devices/` | A passkey prompt comes before the code is shown. With two browser contexts: the new key works. A synced passkey needs no enrolment. Expired and reused codes are refused | R2 | L |
| M5-02 | Unlock keys: list, remove with the one question, change PIN, clear a lock | D | M5-01, M1-11 | `ui/devices/` | Label and last use shown. The question leads to remove or to re-key. For a lost device with a synced passkey the screen names the account step first. A changed PIN works and the old one does not | R2 | M |
| M5-03 | Recovery: with Internet Identity, and with the kit alone | D | M4-08, M1-06 | `ui/recovery/` | Both paths end to end, with pending, cancel, and completion after the clock moves. An old kit number is recognised before typing. Cancelling a recovery always leads on to a kit replacement or a re-key | R2 | L |
| M5-04 | Kit replacement and kit check | D | M5-03 | `ui/recovery/`, `ui/safety/` | A passkey prompt comes first. An interrupted replacement leaves the old kit valid. The check code is accepted and refused correctly. An overdue check blocks | R2 | M |
| M5-05 | Re-key | D | M5-02, M1-12 | `ui/devices/` | A new passkey and a new PIN are created. The new kit is entered in full before the call. Other keys are gone. The old kit and the old passkey fail. The screen says what a re-key cannot undo | R2 | M |
| M5-06 | Safety check, alerts that block, audit screen | D | M5-04, M1-10 | `ui/safety/` | The four rows turn green and amber as specified. The label stands above them. Each alert kind blocks until acknowledged | R1 | M |
| M5-07 | Remaining error screens and lockout texts | D | M5-06 | `ui/` | Every row of the error table of spec 8.5 has a screen and a test | R0 | S |
| M5-08 | Hallway test, recovery drill, and the full rehearsal of spec 11.5 on a fresh staging vault | H | M5-07 | None | A protocol of all three. Every stumble is a task | You | Two days |
| M5-09 | Fixes from M5-08 | D | M5-08 | As needed | The tasks are closed and the ratings of the owner-mistake table are corrected | R1 | M |

Watch out in M5:

- **Two browser contexts, one Internet Identity.** The test copies the credential of the virtual authenticator into the second context. The product code gets no test identity and no bypass.
- **Scanning a QR code** uses the detector of the browser where one exists. Elsewhere the code is typed. A decoder library waits until after the first release.

### M6 Backup and the offline tool

Goal: a backup that opens offline. Files, import and "Move a vault" wait until after the first release, see correction C48. The four deferred rows keep their ids. Spec 6.6 "Backup".

| ID | Task | Lane | Depends | Files | Proof | Review | Size |
|---|---|---|---|---|---|---|---|
| M6-01 | Deferred with the cut line: files | A, B, D | None | None | Nothing now | None | None |
| M6-02 | Backup: `backup_manifest`, the file of 3.8, verify, reminder | A, B, D | M5-09 | `items.rs`, `ui/safety/` | The file ends with its MAC. Verify finds a flipped byte and a missing record. The reminder shows after 90 days | R2 | M |
| M6-03 | Deferred with the cut line: "Move a vault" with its two restore methods | A, D | None | None | Nothing now | None | None |
| M6-04 | Offline tool around the Rust reference of M2-08: read a backup file, decrypt it with the 24 words, verify it. A runbook for leaving a canister with a backup and this tool | C | M6-02, M2-08 | `tools/cli/`, `docs/runbooks/` | It decrypts a backup made in the browser and reports a damaged one. An agent with the runbook alone gets every item out of a backup on the local network | R2 | M |
| M6-05 | Deferred with the cut line: import | D | None | None | Nothing now | None | None |
| M6-06 | Deferred with the cut line: `retire.sh` and the move runbook | C | None | None | Nothing now | None | None |
| M6-07 | Backup drill | H | M6-04 | None | A backup of the staging vault is decrypted offline with the 24 words of its kit, and a copy with one changed byte is reported | You | Two hours |

### Release gate

Goal: the three free steps of 6.5 in place of an audit, then the production vault. See "The release gate" in spec section 12.

| ID | Task | Lane | Depends | Files | Proof | Review | Size |
|---|---|---|---|---|---|---|---|
| AG-01 | Review package of 6.5. A review by a model of another vendor. A public call for review, and 30 days of waiting | C, H | M6-07 | `docs/review/` | The package is public. The findings of the second model and every answer to the call are listed, each fixed or accepted in writing. 30 days have passed since the call | R3 | M |
| AG-02 | Fixes. Then the production vault is created from the release that passed the gate by spec 11.5, and recovered once with the real kit on a second device | All, H | AG-01 | As needed | Every finding is closed or accepted in writing. The release flag is set, and the label is still in place. Stage 2 of spec section 1 opens: secrets you can rotate, with your current password manager as the master copy | R2 | M |

### M7 Sovereign upgrades

Goal: the vault controls itself. Spec 9.1, 9.5.

| ID | Task | Lane | Depends | Files | Proof | Review | Size |
|---|---|---|---|---|---|---|---|
| M7-01 | Spike SP7 and the design of break-glass, decision D15 | A | AG-02 | `spikes/SP7/` | An ADR with the upgrade mechanism and the break-glass path | R2 | M |
| M7-02 | Release registry canister | A | M7-01 | `canisters/registry/` | Publish, list, look up by hash. Only the project principal publishes | R2 | M |
| M7-03 | Upgrade from inside the vault, with approval and cooldown | A, D | M7-02 | `upgrade.rs`, `ui/trust/` | A listed release installs after approval and cooldown. An unlisted hash is refused. A snapshot is taken first | R2 | L |
| M7-04 | Approval through the offline tool, break-glass after the waiting period | A, C | M7-03 | `tools/cli/`, `upgrade.rs` | An upgrade is approved without the web app. Break-glass control arrives only after the wait | R2 | M |
| M7-05 | The two-step switch | C, H | M7-04 | `docs/runbooks/` | Staging is switched and has upgraded itself once. Then production. The backup controller is removed last | R2 | M |
| M7-06 | Deferred with the cut line: the launcher, if decision D11 says yes | A, D | None | None | Nothing now | None | None |

## 5. Tests and gates

### 5.1 The gate

`pnpm check` runs these in order and stops at the first failure. CI runs the same command on every pull request, then the end-to-end tests and the release build.

| Step | What it checks | Budget |
|---|---|---|
| 1. Format and lint | Formatting, the import boundaries of spec 8.1, no `innerHTML`, no `eval` | 30 seconds |
| 2. Types | TypeScript strict, generated bindings up to date with `vault.did` | 30 seconds |
| 3. Rust unit tests | Encodings, lockout arithmetic, limits | 1 minute |
| 4. PocketIC tests | The canister, including the authorization matrix and the upgrade tests | 5 minutes |
| 5. Worker tests | Vectors and property tests, coverage of `crypto/` at 95 percent | 2 minutes |
| 6. Bundle checks | No `fetchRootKey`, no absolute URL outside the allowlist, size report | 30 seconds |
| 7. Script checks | No canister deletion and no reinstall mode under `scripts/`, shell lint | 10 seconds |
| 8. Audits | `pnpm audit`, `cargo deny`, release age of new versions | 30 seconds |

End-to-end tests always load the page that the canister serves, with the final headers. They never use the development server, so a header that breaks the app is found at once.

### 5.2 Vectors

Vectors live in `packages/vectors/` as JSON. The worker tests, the canister tests and later the offline tool all read the same files.

| File | Pins |
|---|---|
| `encoding.json` | `lp`, integer encodings, padding buckets |
| `kit.json` | 32 bytes to 24 words and back, checksum failures, kit factor, kit principal, export key, check code |
| `slot-keys.json` | `F`, `P`, salts, vault id and slot id to `slot_kek` and both proof keys, for the two slot kinds of the first release |
| `unlock-proof.json` | The exact message bytes and both proofs, as the canister code accepts them |
| `slot-wrap.json` | Inner wrap with a fixed nonce, its associated data, and an encrypted label |
| `items.json` | Data key wrap, header and body with fixed nonces, every item type |
| `handoff.json` | Token, handoff key, handoff blob, enrolment proof |
| `export.json` | Export header, backup manifest, and the MAC of a small backup file |
| `files.json` | Deferred with files: chunk encryption with the last-chunk flag |

Two rules keep the vectors honest:

- **Standard primitives are checked against published vectors first**: HKDF from RFC 5869, AES-GCM from NIST, Argon2id from RFC 9106, Ed25519 from RFC 8032, the word encoding from the BIP39 test set. A vector produced by our own code only proves that the code did not change.
- **Every construction gets a second implementation.** M2-08 builds a Rust reference of all of spec 6.3, and both implementations must produce every vector file byte for byte. The offline tool of M6-04 is built around that reference.

### 5.3 End-to-end scenarios

| From | Scenario |
|---|---|
| M3 | Claim, device check, setup with full kit entry, practice unlock, add a login, lock, unlock, edit, delete, restore from trash |
| M3 | Canary: a marker string stored as a password never appears in any request or in browser storage |
| M3 | Setup cannot be finished with a wrong word. Five wrong PINs lock the key, and the screen shows the wait |
| M4 | The same slice against the single Wasm, with the page policy active and no console violation |
| M5 | Add a device by code. Remove a device. Change a PIN. Replace the kit, interrupted and completed. Re-key |
| M5 | Recovery with Internet Identity and kit. Recovery with the kit alone: pending, cancelled from a device, then completed after the clock moves |
| M5 | An unlock key added on one device blocks the next unlock on another until acknowledged |
| M6 | Backup, then verify. The offline tool decrypts the same file with the 24 words |
| M6 | A backup file with one changed byte is reported by "Verify a backup" and by the offline tool |
| After the first release | Files, import, and a restore into a second canister |
| M7 | A vault upgrades itself to a listed release and refuses an unlisted one |

### 5.4 Budgets

The gate records these numbers on every run and fails when one gets worse by more than a tenth without an ADR.

| Budget | Target | Measured in |
|---|---|---|
| Unlock on mainnet, from the PIN to the visible list | 6 seconds on a desktop, excluding the passkey prompt | M4-07 |
| Argon2id for one PIN | 2 seconds on a mid-range phone | SP2, M2-03 |
| Wasm size, compressed | Recorded at M4-01. Alarm when it grows by a tenth | M4-01 |
| Instructions of `post_upgrade` with a full vault | Under a tenth of the limit | M1-02, M1-14 |
| Instructions of `unlock` | Recorded | M1-05 |
| Instructions of the signature check in the message hook | Under a tenth of the limit | SP5 |
| Size of one sync page | At most 1.5 MiB | M1-08 |
| Cycles for unlock plus first sync | Recorded on staging | M4-07 |

## 6. Review plan

### 6.1 Levels

| Level | What happens | Applies to |
|---|---|---|
| R0 | Gate and CI only | Texts, styles, screens without secrets |
| R1 | R0, then a fresh agent session reviews the diff against the spec and this plan. It sees neither the author's reasoning nor the task discussion | Most code |
| R2 | R1, then you approve | Everything under `crypto/`. In the canister: `access.rs`, `unlock.rs`, `sessions.rs`, `slots.rs`, `claim.rs`, `http.rs`, `upgrade.rs`. In the web app: `platform/`. All of `scripts/`. `vault.did` and section 3 |
| R3 | The review of the release gate: a model of another vendor, then the public call of 6.5 | The crypto spec, the canister access logic, the worker, the owner-mistake table. Before any real secret |

A task inherits the highest level of any file it touches.

### 6.2 What a reviewer checks

**Canister**

- The gate is the first statement of the method, and the method has a row in the authorization matrix. The message hook is an extra. It is never the gate.
- No trap after a state change that has to persist. Failed tries are the known case.
- The caller is read before the first `await`.
- Every input has a size check before it is stored or looped over.
- Errors are values of the `Error` type. No text built from arguments, no logging of arguments.
- Time comes from the canister clock only.
- A method that takes a slot checks its kind and its fields. A method that ends in a full session has seen a proof of a factor or of the vault key.

**Worker**

- No key and no factor leaves the worker. Results are ciphertext, proofs, or the plaintext of the one item asked for.
- Associated data matches spec 6.3 byte for byte, proven by a vector.
- Nonces are 12 fresh random bytes. No counter, no reuse.
- Buffers that held `F`, `P` or the vault key are overwritten on every path, including errors.
- No new dependency.

**Web app**

- Kit words are asked for only at kit creation and on the recovery screens.
- The relying party id of every passkey call is the full host, and user verification is required.
- No request to a host outside the allowlist. No new entry in browser storage.
- Every error screen says what happened, whether the data is safe, and what to do next.

**Scripts**

- The environment is the first argument. Production asks for the typed confirmation.
- Every mainnet install names its mode. `icp deploy` does not appear.
- A failed check stops the script before the next mainnet call.

### 6.3 Reviews on the calendar

| When | Review | By | Why then |
|---|---|---|---|
| Before M1-01, tasks DR-03 and DR-04 | Design review of spec sections 6 and 7 and of section 3 here | Done on 2026-11-09 by Claude, the author, instead of a specialist. See 6.4 | Changing a format is free before the freeze and expensive after |
| End of M2 | Vectors and constructions | A fresh agent session with the strongest model, then you | The crypto core is complete and nothing depends on it yet |
| At M4-06 | Full security pass against `docs/threat-model.md` | Strongest model, fresh context, one pass per adversary | Last stop before code runs on mainnet |
| At M5-08 | Hallway test and recovery drill | A person who has never seen the project, and you | Usability findings change screens, and screens are cheap to change now |
| Release gate | A review by a second model, and a public call that waits 30 days. R3 | A model of another vendor, and whoever answers the call | Before real secrets. There is no budget for an audit |
| Before M7 closes | Security pass on the upgrade path | Strongest model, then you | A flaw here makes vaults unfixable |

### 6.4 The design review before the freeze

Decision I1 changed twice on 2026-11-09. First it was yes, by an outside specialist. Then you had Claude review instead. That review is done, and its findings are in `DESIGN-REVIEW-FINDINGS.md`.

- **What this means.** The author has read the design twice. Nobody else with the right experience has read it once. There is no budget for an audit either, decision I4. So no person with the right experience reads the design before real secrets go in, unless the public call of 6.5 brings one. The trust stages of spec section 1 are the answer to that.
- **What came of it.** On the same day you chose every proposed correction and deferred the network key. They are corrections C37 to C47 in 3.10, and the spec went to revision 5.

The rest of this section describes the outside review that did not happen. It stays, because the brief is the core of the public call of 6.5, and it still works if the money for a reviewer turns up.

- **Who.** An applied cryptographer who has reviewed a password manager, key management or end-to-end encrypted storage. Experience with passkeys helps. Knowledge of the Internet Computer is not needed, because the brief states the platform facts. If you want the platform side read too, a person who audits canisters can take section 3 of this plan for half a day.
- **Where to look.** An independent consultant is the right size for two days. Audit firms mostly sell weeks. In the Internet Computer ecosystem, firms that have published canister audits are a starting point, and DFINITY lists the firms that audited its own code. These come from one search. They are examples, not recommendations.
- **What to ask for.** A fixed price for two days of reading and writing and a one-hour call. Send the brief with the first message, so the reviewer can judge the size before answering.
- **What it costs.** Day rates I found range from about 600 EUR for a freelancer to 2,000 to 5,000 USD at established firms. They are statements by vendors, not a survey. Two to three days therefore cost somewhere between 1,500 and 10,000. Some consultants give a discount for open-source work.
- **While you wait.** See the note under the task table: everything that fixes no format goes on.
- **Afterwards.** Task DR-04. Each finding ends as a correction, an accepted risk or a deferred item, each with your decision.
- **If nobody is booked three weeks after the package is ready,** you choose between waiting and freezing with RV1, RV5, RV15 and RV20 accepted in writing. In both cases the package is also published for comment, since all of it will be open source anyway.

### 6.5 The release gate

Decision I4 was settled on 2026-11-09: there is no budget for a paid audit. Correction C49 puts a release gate in its place. It is weaker than an audit, and the product says so.

**The review package** is prepared by AG-01 and published: the spec, section 3 of this plan, the threat model with all tables, the vectors, a map from each claim of spec section 1 to the code and tests that uphold it, the list of section 7 with its current answers, and `DESIGN-REVIEW-BRIEF.md` brought up to date.

1. **A review by a second model.** A model from another vendor gets the package and the last prompt of section 8. It did not write the design. A second session of the authoring model shares the blind spots of the first, so it does not count here. The findings are triaged like those of 6.4.
2. **A public call for review.** The package goes to the Internet Computer developer forum and to one cryptography community. The post asks the three most important questions of the brief, because "please review" gets no answers. The gate waits 30 days. Silence is not approval. The call stays open, and the README links to it.
3. **The label.** "Not independently reviewed" stands in the README, at setup and in the safety check. It goes only when a named person who is independent of the project has read the design and published the result. An agent does not count, and neither do you.
4. **The trust stages** of spec section 1. The gate opens stage 2 and nothing more.

**What could end the label.** A reader who answers the call, or a funded review. One search on 2026-11-09 found no open route. The last call of NLnet closed in June 2026. The grants of DFINITY do not name audits. OSTIF arranges audits for established projects. The yearly upkeep looks again.

**If the money turns up after all,** two days of an applied cryptographer on the crypto spec buy the most. The rest of 6.4 says how to book one.
## 7. What needs more review

Both documents were written in one day by one author. Nothing in them has been compiled, run or read by a specialist. Two reviews by fresh agent sessions found 28 problems in about half an hour each, see 7.9. A specialist will find more. Each item below has a deadline:

- **P1** before the interface freeze, task M1-01. Changes are free until then.
- **P2** before code runs on mainnet, task M4-07.
- **P3** before real secrets, the release gate.
- **P4** before the vault controls itself, M7.

### 7.1 The short version

1. **The headline claim was too strong, and is corrected.** Whoever can change the code your browser runs reads everything at the next unlock. That is the gateway, a majority of the subnet's nodes, and a controller in stage A. The claim now covers readers only. Whether real secrets wait for a client that does not arrive through the gateway is decision D19. Item RV6.
2. **No specialist has read the design.** You had Claude review it instead, on 2026-11-09. That review found eighteen more problems, two of them high, see 7.10. It was done by the author and is not independent. There is no budget for an audit, so the public call of 6.5 is the only chance of an independent reading. Trust therefore grows in stages, spec section 1.
3. **The PIN is weaker than it looks when two things are stolen.** Item RV1.
4. **The network key is deferred.** You decided that on 2026-11-09. A copy of the canister state together with the secret of a vault passkey now reads the vault, spec R24.
5. **Confirm or reverse what was changed without you.** Ten defaults of the spec, and the corrections C1 to C36 of 3.10.
6. **Two spikes can still change the design**: passkeys with PRF on the real kind of address, and the single Wasm on mainnet. SP1, SP3. Since the cut line SP1 decides more: one device of yours without PRF brings the passphrase key back.
7. **The first release is smaller.** You took the cut line of 7.7 on 2026-11-09, correction C48. Until "Move a vault" exists, leaving a canister means a backup, the offline tool and entering the data again, spec R25.

### 7.2 Decisions that are yours

| # | Decision | Recommendation | By |
|---|---|---|---|
| I1 | A design review before the freeze | Decided: done by Claude instead of a specialist. Triage done the same day | Done |
| I2 | Full scope, or the cut line of 7.7 for the first release | Decided on 2026-11-09: the cut line. Correction C48, spec D20 | Done |
| I3 | Language of the interface | Decided on 2026-11-09: English only, with every text in one module | Done |
| I4 | Auditor and budget | Decided on 2026-11-09: no paid audit. The release gate of 6.5 takes its place, correction C49 | Done |
| I5 | Argon2id in JavaScript or as Wasm | Whatever SP2 measures | M2-03 |
| I6 | The repository: name, license, visibility, owner, reporting path, place on your machine | Decided on 2026-11-09: `ic-vault`, Apache-2.0, public from the first commit, a GitHub organization, GitHub private vulnerability reporting, a new folder under `C:Dev`. Spec D1, D2 and D21 | Done |

Defaults of the spec that I set or changed without your answer:

| Decision | Now | Before, or the alternative |
|---|---|---|
| D3 protection profile | Passkey plus PIN, the only profile in the first release | Passkey alone |
| D8 gateway domain | Decided by you on 2026-11-09: `icp.net`, ADR 0002, on condition that SP1 passes there | `icp0.io` |
| D9 kit format | 24 words | 52 characters |
| D12 waiting period | 48 hours for the kit alone | None |
| D13 trash | 30 days | Immediate deletion |
| D14 kit check | After 30 days, then yearly | None |
| D16 vault id | Separate from the canister id | Keys bound to the canister id |
| D17 watchdog | A scheduled workflow in a private repository | A task on your own machine |
| D18 staging key | Moot while the network key is deferred | `test_key_1` |
| D19 first real secrets | Decided by you on 2026-11-09: in stages, spec section 1 | The web app alone for everything |

Corrections C1 to C36 of 3.10 belong on this list too. They change the interface, the build order and several formats of the spec. C37 to C48 were your own decisions.

### 7.3 Design questions for a specialist

| # | Question | Why it matters | Options | Default here | By |
|---|---|---|---|---|---|
| RV1 | The PIN can be tested offline by someone who holds the passkey secret and either a copy of the canister state or one recorded unlock request | The proof key is derived from passkey secret and PIN. Its public half is stored, and every unlock request carries a signature made with it. A passkey provider has the passkey secret, and so has a thief who knows the device PIN. With it, each PIN can be tried without the canister counting. This takes two thefts, so the claim of spec section 1 holds, but the tries protect less than they seem to | Accept and say so. Or, once the network key returns, a PIN proof that also depends on it. Or a vetKD derivation at every unlock, or a password protocol such as OPAQUE | Accept for the first version and write it into the threat model | P1 |
| RV2 | A re-key changes the vault key, not the keys of the items | Someone who copied an old vault key and an old state can still read every item that has not changed since. "Re-key after a break-in" promises more than it does | Say so on the re-key screen | Done, correction C45. A deeper re-key would add nothing, review finding F12 | Done |
| RV3 | Keys are bound to a vault id, not to the canister | A ciphertext is no longer tied to the place that stores it, and a vault can exist in two canisters for a while. Decision D16, my change | Keep, or go back to the canister id and move vaults by decrypting | Keep | P1 |
| RV4 | The waiting period covers one case only: a thief who has the kit and nothing else | Kit plus Internet Identity, kit plus a backup file, and kit plus a controller key all open the vault at once. In stage B the break-glass path is still open, decision D15 | Keep it and describe it honestly. Or drop it and save the code | Keep. The way a kit thief could keep the owner out is closed, correction C38 | P2, P4 |
| RV5 | Correction C27 changes how passkeys are handled: a new passkey at every re-key, a random PRF salt per slot, the slot kind in the key labels | It closes a real hole, but it was designed in an hour. Retired passkeys also pile up in the authenticator | Keep. Or also remove old passkeys through the browser's signal interface where it exists | Keep | P1 |
| RV6 | The page itself is delivered by others on every visit | Altered code served by the gateway, by a majority of the subnet, or by a stage A controller reads everything at the next unlock. The claim of spec section 1 now says so, correction C24 | A browser extension. A verifying gateway on your own machine. Or accept | Accepted for secrets you can rotate. Everything else waits for stage 3, decision D19 | P3 |
| RV7 | Seven nodes guard the try counter and the waiting period, and certify the code | The confidential subnet hides data from node providers but has few nodes. A 34-node subnet has no such protection. Decision D4 trades one for the other | Confidential subnet, or a larger one without memory encryption | Confidential subnet | P2 |
| RV8 | Some answers before unlock still come from a single node | The pending-recovery warning, the slot descriptors and the stretching parameters come from a query. A lying node can hide a warning, or make a correct PIN count as a wrong try. After unlock the same facts arrive certified | Accept. Or pay cycles for a certified call before every unlock | Accept. A refused try now carries a hash of the slot settings, so the app can tell that a node lied, correction C47 | P2 |
| RV9 | The worker keeps keys from being copied, not from being used | A script that runs in the page while the vault is open can ask the worker to decrypt, and can sign calls with the session key | The strict page policy is the defence. A reviewer should try to break it | Keep. Three requests that equal a key export now need a fresh passkey answer, correction C44 | P2 |
| RV10 | Erasing secrets in JavaScript is best effort | A PIN is a string before it is bytes, and strings cannot be overwritten. The engine may keep copies | Document it | Document it | P3 |
| RV11 | Anyone can restore a backup file into a canister of their own | It gives them nothing without the kit, but the new canister takes the kit principal from the file as an owner | Review the restore path for surprises when "Move a vault" is built | Deferred with the cut line | After the first release |
| RV12 | `owner_set` trusts a code that the owner scans | Scanning a stranger's code locks the owner's Internet Identity out. The data stays safe and the kit repairs it | Typed confirmation and a blocking alert | Both | P2 |
| RV13 | A boundary node could claim a freshly created vault first | Harmless in stage A, where you simply create another one. It matters for a public launcher | Bind the claim to the caller when the launcher is built, after the first release | Defer | P4 |
| RV14 | The code for adding a device is as long as the kit | A short code would need a password protocol | QR code where the browser can scan, 24 words elsewhere | Keep | P3 |
| RV15 | What the network key buys | It stops someone who has a copy of the state or watches the traffic, and who knows what a slot needs, but who cannot unlock through the canister. For a PIN slot that is: state plus passkey secret, without the Internet Identity. For a passphrase slot it is all that stands between a guessed passphrase and the vault key. Against a provider that holds both passkeys it adds nothing. It is also the most complex layer: a threshold call, transport keys, a key cache in the browser, epochs, a daily limit | Keep. Or leave it out of the first version together with the passphrase slots, and keep room for it in the slot format. Or keep it and advise a separate authenticator for the Internet Identity | Decided on 2026-11-09: deferred until after the first release, correction C37 | Done |
| RV16 | Any canister can make the vault pay for refusing calls | A call from another canister skips the message hook. Draining a vault costs the caller several times less than it costs the owner. The result is a frozen vault, not a leak. Spec R22 | Refuse in the first statement, fund generously, watch the balance. The platform offers nothing better | Accept | P2 |
| RV17 | A controller can turn the vault back | Restoring a snapshot brings back removed unlock keys, an old PIN and a replaced kit. Spec R23 | Devices remember the newest state they have seen and stop when it goes back | Not built in the first version. Stage B removes the outside controller | P4 |
| RV18 | An overwritten item is not covered by the 30-day rule | A faulty client could replace an item with rubbish. Earlier values live only inside the item | Keep one previous version on the server for 30 days | Not in the first version | P3 |
| RV19 | The flow for a change of address is an outline with a known gap | After the owner is switched, the old address can no longer unlock | A second owner for a transition period | Deferred with the cut line | P4 |
| RV20 | A failed try is written by the same message that checks the proof | A trap between the check and the write would lose the count. The rule is that this code never traps | Count first and give the try back on a success | Done, correction C40 | Done |
| RV21 | A stolen Internet Identity can keep every PIN key locked | One wrong try per period is enough. The owner then needs the kit until the vault has a new Internet Identity | Two proofs per unlock. Only someone who holds the passkey can then use up tries | Done for passkey keys, correction C39. A passphrase key could still be kept locked, and the first release has none | Done |

### 7.4 Facts that only a spike or a rehearsal can settle

| Fact | Settled by | If it is false |
|---|---|---|
| Every device you use gives a PRF output | SP1 | The passphrase key comes back into the first release, and RV1 gets worse on those devices |
| A passkey works on an address that is itself a public suffix | SP1, on the SP3 canister | Another gateway domain, or a custom domain, which the spec avoids |
| PRF is available at creation, not only at sign-in | SP1 | Setup needs a second passkey prompt |
| The browser can scan a QR code by itself | SP1 | The typed code. A decoder library waits until after the first release |
| Argon2id is fast enough in JavaScript | SP2 | A Wasm build, and one more exception in the page policy |
| One Wasm with the web app installs and loads on mainnet | SP3 | Two canisters and two hashes |
| The final page policy works with React, the SDK, a worker and the Internet Identity popup | SP3 | Single headers are relaxed by ADR |
| An Ed25519 check fits into the message hook | SP5 | Wrong proofs cost cycles and need a rate limit |
| The SDK signs with a non-extractable session key | SP6 | The session key moves into the worker |
| End-to-end tests with Internet Identity run in CI | SP8 | They run locally before each merge |
| A frozen canister stops serving its page | M4-07 | Nothing changes. The kit sheet covers both cases |
| A keyring exists under WSL | M4-07 | Staging uses a password too |
| `icp.yaml` can declare a released Wasm by hash | M4-03 | The scripts check the hash themselves |
| The canister can read its balance above the freezing reserve | M1-14 | Health is estimated from the total balance |
| The cost table of spec 11.3 | M4-07 | The funding advice changes |
| A canister can install code on itself | SP7 | An upgrader canister per vault |

### 7.5 Assumptions about people

None of these has been tried with a person. The hallway test of M5-08 is late for the first three, so a paper mock-up of setup, shown to one person before M3-04, is worth an hour.

- Someone will type 24 words in full at setup without giving up.
- A PIN on every unlock is acceptable several times a day.
- The plain questions for removing a device and replacing the kit are understood the way they are meant.
- An unlock that takes about five seconds is acceptable.
- An owner reacts to a blocking alert instead of clicking it away.
- An owner keeps the kit on paper in two places for years.
- After losing a device, an owner removes it from the cloud account that syncs the passkeys.

### 7.6 Estimates

- **Sizes and weeks** in section 1 are guesses. Correct them after M1.
- **Costs** in spec 11.3 are scaled from a 13-node price list.
- **The ratings** in the three risk tables of the spec are my judgement, not measurements.
- **Wasm size, unlock time and instruction counts** are unknown until the budgets of 5.4 are measured.

### 7.7 A cut line for the first release

**Taken on 2026-11-09, correction C48.** Every row below now waits until after the first release. None of them weakens the claim of spec section 1. Together they remove about a fifth of the paths a reviewer has to read.

| Cut or defer | Saves | What you lose until later |
|---|---|---|
| The slot kinds without a PIN and without a passkey. Keep passkey plus PIN, and the kit | Branches in unlock, lockout, message hook and setup. No passphrase that could be guessed offline | Devices without PRF cannot unlock. Only if SP1 passes on all your devices |
| Files, task M6-01 | One large task and a chunk protocol | No attachments |
| Import, task M6-05 | Plaintext files in the browser | Typing your entries in, or waiting |
| Change of address: the screens and the second accepted host | One flow and an upgrade argument | A retired domain is handled with the kit and its wait |
| "Move a vault", task M6-03. The vault id stays in every format | A second way to fill a canister | Leaving a subnet means reading a backup with the offline tool and entering the data again |
| A QR decoder library | A dependency | Where the browser cannot scan, the code is typed |
| The quota half of `limits_set` | A setting | A fixed quota of 1 GiB |
| The launcher, task M7-06 | A second canister type | Self-hosters only |

The network key was deferred before, correction C37.

One condition remains: SP1 passes on every device you use. If it does not, the first row is undone before the freeze, by ADR.

### 7.8 What this plan has not had

- **Section 3 has not been compiled.** The Candid may not parse and the types may not fit the libraries. M1-01 finds out.
- **No task has been tried.** The first tasks of each lane will show whether the tasks are the right size.
- **The corrections of 3.10 were found by writing and by two agent reviews, not by testing.** More will appear in M1.
- **Two agent reviews are not an audit.** Each read revision 3 of the spec once. The corrections they led to were designed the same day and have not been reviewed themselves.
- **No specialist has read the design.** The review of 2026-11-09 was done by its author.
- **There will be no audit.** Decision I4. The release gate of 6.5 is weaker, the label says so, and what cannot be rotated stays out of the vault.

### 7.9 What the independent reviews found

Two reviews by fresh agent sessions were started for this section. They read revision 3 of the spec and did not see this document. I checked each finding against the spec before acting on it. Together they raised 28 findings. All but one were accepted in full or in part.

**Feasibility on the Internet Computer**

| Finding | Verdict | Where it went |
|---|---|---|
| Sections 6 and 7 of the spec were a sketch: no types, no error model, no encodings | Agreed | Section 3. Integers and labels in C19 |
| Deciding platform facts were tested only at M4. The address is itself a public suffix | Agreed | SP1, SP3 and SP4 run at M0 on a real mainnet canister. New rows in 7.4 |
| The M4 rehearsal needed M5 features, and a production vault came too early | Agreed | C14 |
| The authorization matrix could not see a missing gate, because the message hook answers first | Agreed | Second run through a proxy canister, 3.4 and M1-03 |
| Unlock was ambiguous: which counter resets, and what a refusal looks like | Agreed | C1, C22, one decision function |
| The page policy would have broken the app: worker start, passkeys, clipboard | Agreed | C20, and the final headers already in M0-05 |
| The format of the outer wrap belonged to a library | Agreed | C15 |
| Alerts and settings rested on answers from a single node | Agreed, and found independently | C2, C4 |
| A re-key and a sync page had no size limits | Agreed, and found independently | C6, C21 |
| Client rules contradicted each other: storage, and WebCrypto outside the worker | Agreed | C16. Drafts and acknowledgements live in the canister |
| A redirect that depends on the host cannot be certified | Agreed | C17 |
| Cycles: any canister can make the vault pay, uploads cost more than stated | Agreed | RV16, spec R22 and spec 11.3 |
| End-to-end tests were outside the gate, and adding a device needs one identity in two browsers | Partly. They run in CI. A test identity inside the product is refused | 5.1, watch-out in M5 |
| The sign-in client logs out after ten idle minutes. Scanning a QR code needs a decoder | Agreed | C23, SP1 |

It also proposed cuts. Timers are gone, C18. No production vault before the release gate, C14. Change of address and "Move a vault" are on the cut line of 7.7, which you have since taken. Its largest proposal was to drop the network key. You have since deferred it, item RV15.

**Cryptography and protocol**

| Finding | Verdict | Where it went |
|---|---|---|
| The claim of section 1 did not hold against a party that can alter the delivered code | Agreed. The most important finding of both reviews | C24, RV6, spec D19 |
| The stored proof key and every unlock signature let PINs be tested offline | Agreed, and found independently. Not fixed, because every fix changes the protocol | RV1, RV21. C26 removes self-chosen passphrases |
| A re-key did not lock out a lost device that shares a synced passkey, and could not have built a PIN slot | Agreed | C27, RV5 |
| Decisions rested on answers from a single node, including the stretching parameters | Agreed, partly found before | C2, C4, C25, RV8 |
| An enrolment could end in a full session without the vault key | Agreed | C28 |
| Four guesses after each of the owner's unlocks never triggered a lock | Agreed | C29 |
| A backup file was not authenticated as a whole | Agreed, with a different fix. The MAC key comes from the vault key, because making a backup must not need the kit | C30 |
| The rules of spec 7.9 were not enforced by the methods | Agreed for slots. A stored previous version of every item is deferred | C34, RV18 |
| The life of a pending recovery was undefined | Agreed | C31 |
| The flow for a change of address breaks at the first lock | Agreed. The flow is deferred and marked as an outline | RV19 |
| Cycle drain through query methods, a permanently locked PIN key, a flooded log | Agreed | C32, C33, RV21 |
| A snapshot restore turns the vault back unnoticed | Agreed, not built | RV17, spec R23 |
| Smaller gaps between flows and interface | Agreed | C19, C36 |
| Passkey parameters were not pinned | Agreed | C35 |

### 7.10 The author's own review

On 2026-11-09 Claude reviewed revision 4 against the eleven questions of `DESIGN-REVIEW-BRIEF.md`. The full text is in `DESIGN-REVIEW-FINDINGS.md`. You decided on each the same day. Every proposed correction is applied, as C37 to C47 in 3.10. The network key is deferred. F7 waits for spike SP2, and F17 stays deferred.

| Finding | Severity | In one line |
|---|---|---|
| F1 | High | A kit thief can keep the owner out during the waiting period and gets in when it ends |
| F2 | High | A stolen Internet Identity can keep every PIN key locked without holding a passkey |
| F3 | Medium | A trap on the failure path could make wrong guesses free |
| F4 | Medium | A session outlives the key it was opened with, and a re-key leaves stale sessions that can spoil items |
| F5 | Medium | Guess limits are per key and never end |
| F6 | Medium | The network key costs more than it protects. You deferred it |
| F10 | Medium | Three worker messages are as good as a key export |
| F11 | Medium | The claim omits two ways to change the code, and the pairs of thefts that are enough |
| F7 to F9, F12 to F18 | Low or note | Parameters, wording, and small gaps |

No finding lets a single thief read the vault. The two high ones break the waiting period and the availability of PIN keys.
## 8. Agent prompts

**For a task**

```text
You are working on ic-vault. Do task <ID> from docs/implementation-plan.md and nothing else.

1. Read AGENTS.md, then the row of task <ID> and section 3 of docs/implementation-plan.md.
2. Read the spec sections the task names, in docs/crypto-spec.md, docs/threat-model.md or VAULT-PLAN.md.
3. Load the pinned ICP skills that fit the task before you write canister or SDK code.
4. Write the tests of the proof column first and watch them fail.
5. Implement until pnpm check passes. Do not weaken, skip or delete a test.
6. If the plan or the spec is wrong or unclear, stop. Write docs/adr/<number>-<slug>.md with the question
   and your proposal, and report.
7. Open one pull request from task/<ID>-<slug>. Say in its description what you built, which tests you added,
   and what you could not verify. Tick the row in the task table.

Do not touch files of another lane. Do not add a dependency. Do not run anything against mainnet.
```

**For a review**

```text
You are reviewing a pull request for ic-vault. You did not write it and you have not seen its discussion.

1. Read AGENTS.md, the row of task <ID> and section 6.2 of docs/implementation-plan.md.
2. Read the spec sections the task names. Then read the diff.
3. Check the diff against the list in section 6.2 that fits its files. Check that every test of the proof
   column exists and tests what it claims to test.
4. Try to break it. Name inputs, callers and orders of calls that the tests do not cover.
5. Report findings ranked by severity, each with file and line, what goes wrong, and a failing test if you
   can write one. Say plainly when you found nothing.

Do not fix anything and do not approve. The human decides.
```

**For the review of the release gate, task AG-01**

```text
You are reviewing the design and the security-critical code of ic-vault before its first release.
You wrote none of it. Its authors were models of another vendor. Do not assume they were right.

1. Read docs/review/ in the order its brief gives: the claim, the threat model, the crypto spec,
   then section 3 of docs/implementation-plan.md.
2. Answer every question of the brief. Say "I cannot tell from this" where that is the truth.
3. Read the code under canisters/vault/src/ and apps/web/src/crypto/ against the spec.
4. Look for what the tests do not cover: a way to a session without a factor, a guess that is not
   counted, a key or a plaintext that leaves the worker, a format that binds to the wrong id.
5. Report findings in the format of the brief, ranked by severity. List what you did not read.

Do not fix anything. Do not soften a finding because the project cannot pay for an audit.
```

**For a spike**

```text
You are answering one question for ic-vault with throwaway code. Do spike <ID> from section 2 of
docs/implementation-plan.md. Work under spikes/<ID>/. Nothing there is reused later.

End with docs/adr/<number>-<slug>.md: the question, what you ran, the numbers, the answer, and what it means
for the tasks in the Feeds column. If the spike fails its pass condition, say so in the first line.
```

## Appendix. The first ten steps

1. Read `DESIGN-REVIEW-FINDINGS.md` once, so that you know what revision 5 changed and why. Revision 6 only removed scope. Revision 7 put a release gate in the place of the audit.
2. You create the GitHub organization and the public repository `ic-vault`, switch on private vulnerability reporting, and clone it into a new folder under `C:\Dev`. The first commit holds both plans, the review findings and the review brief. Then M0-01: pins, ignore rules.
3. M0-02: documents copied. You decide D8, the gateway domain. Then DR-01, the review package.
4. M0-03 and M0-04: gate and CI.
5. M0-05: the hello canister, with spike SP3 on a throwaway mainnet canister. You run the mainnet part.
6. M0-06: end-to-end harness, spike SP8.
7. SP1 on every device you own, on the page that the SP3 canister serves. This is the spike only you can run.
8. SP2, SP5 and SP6, each by one agent. M4-02 can start as well.
9. Look at the SP1 table. If a device you use failed, the passphrase key comes back by ADR before the next step.
10. M1-01: the interface freeze. From here on three lanes run side by side.
