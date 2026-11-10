//! Throwaway spike SP9. One thin path through the interface of section 3.2 of
//! docs/implementation-plan.md: claim, setup, unlock, save one item, read it back.
//!
//! Not the product. State lives on the heap, most methods are stubs, and nothing here
//! was reviewed. The types follow section 3.2 exactly, so that a test can compare the
//! exported interface with `vault.did`.

use candid::{CandidType, Nat, Principal};
use ed25519_dalek::{Signature, VerifyingKey};
use ic_cdk::api::{accept_message, canister_self, msg_arg_data, msg_caller, msg_method_name, time};
use serde::Deserialize;
use serde_bytes::ByteBuf;
use sha2::{Digest, Sha256};
use std::cell::RefCell;
use std::collections::BTreeMap;

// ---------- Types of section 3.2 ----------

#[derive(CandidType, Deserialize)]
pub enum InstallArgs {
    Init {
        claim_hash: ByteBuf,
        host_suffix: String,
        environment: String,
        local_root_key: Option<ByteBuf>,
    },
    Upgrade {},
}

#[derive(CandidType, Deserialize, Clone, Debug)]
pub enum Error {
    NotAllowed,
    WrongState(String),
    NotFound,
    Locked(Option<u64>),
    BadProof {
        tries_left: u8,
        settings_hash: ByteBuf,
    },
    StaleNonce(u64),
    RecoveryPending(u64),
    VersionConflict(u64),
    TooLarge,
    QuotaExceeded,
    LimitReached(String),
    RateLimited(u64),
    Invalid(String),
}

#[derive(CandidType, Deserialize)]
pub enum VResult {
    Ok,
    Err(Error),
}

#[derive(CandidType, Deserialize, Clone, Copy, PartialEq)]
pub enum SlotKind {
    PasskeyPin,
    Kit,
}

#[derive(CandidType, Deserialize, Clone)]
pub struct Kdf {
    pub memory_kib: u32,
    pub passes: u32,
    pub lanes: u32,
    pub salt: ByteBuf,
}

#[derive(CandidType, Deserialize, Clone)]
pub struct Slot {
    pub id: ByteBuf,
    pub kind: SlotKind,
    pub credential_id: Option<ByteBuf>,
    pub prf_salt: Option<ByteBuf>,
    pub kdf: Option<Kdf>,
    pub salt: ByteBuf,
    pub auth_f_pub: Option<ByteBuf>,
    pub auth_pub: ByteBuf,
    pub wrap: u8,
    pub wrapped: ByteBuf,
    pub label: ByteBuf,
}

#[derive(CandidType, Deserialize)]
pub struct SlotInfo {
    pub id: ByteBuf,
    pub kind: SlotKind,
    pub credential_id: Option<ByteBuf>,
    pub prf_salt: Option<ByteBuf>,
    pub kdf: Option<Kdf>,
    pub salt: ByteBuf,
    pub nonce: u64,
    pub created: u64,
    pub last_used: Option<u64>,
    pub locked_until: Option<u64>,
    pub needs_clearing: bool,
    pub label: ByteBuf,
}

#[derive(CandidType, Deserialize)]
pub enum Health {
    Ok,
    Low,
    Urgent,
    Unknown,
}

#[derive(CandidType, Deserialize)]
pub struct SlotsPublic {
    pub setup_done: bool,
    pub slots: Vec<SlotInfo>,
    pub vk_epoch: u32,
    pub kit_number: u32,
    pub kit_created: u64,
    pub pending_recovery: Option<u64>,
    pub health: Health,
}

#[derive(CandidType, Deserialize)]
pub struct UnlockArgs {
    pub slot_id: ByteBuf,
    pub nonce: u64,
    pub session: Principal,
    pub proof_f: Option<ByteBuf>,
    pub proof: ByteBuf,
}

#[derive(CandidType, Deserialize)]
pub struct AuditEvent {
    pub seq: u64,
    pub time: u64,
    pub kind: String,
    pub caller: Principal,
    pub slot_id: Option<ByteBuf>,
}

#[derive(CandidType, Deserialize)]
pub struct UnlockOk {
    pub wrapped: ByteBuf,
    pub session_expires: u64,
    pub previous_unlock: Option<u64>,
    pub failed_since: u32,
    pub uncleared: Vec<(ByteBuf, u32)>,
    pub alerts: Vec<AuditEvent>,
}

#[derive(CandidType, Deserialize)]
pub struct SetupArgs {
    pub device_slot: Slot,
    pub kit_slot: Slot,
    pub kit_principal: Principal,
    pub export_header: ByteBuf,
    pub vault_auth_pub: ByteBuf,
    pub session: Principal,
}

#[derive(CandidType, Deserialize)]
pub struct KitReplace {
    pub kit_slot: Slot,
    pub kit_principal: Principal,
    pub export_header: ByteBuf,
}

#[derive(CandidType, Deserialize)]
pub struct Handoff {
    pub id: ByteBuf,
    pub token_hash: ByteBuf,
    pub wrapped: ByteBuf,
}

#[derive(CandidType, Deserialize)]
pub struct EnrollArgs {
    pub token: ByteBuf,
    pub session: Principal,
}

#[derive(CandidType, Deserialize)]
pub struct EnrollOk {
    pub handoff_id: ByteBuf,
    pub wrapped: ByteBuf,
    pub vk_epoch: u32,
}

#[derive(CandidType, Deserialize)]
pub struct ItemPut {
    pub id: ByteBuf,
    pub expected_version: u64,
    pub wrapped_dek: ByteBuf,
    pub header: ByteBuf,
    pub body: ByteBuf,
}

#[derive(CandidType, Deserialize)]
pub struct ItemPutOk {
    pub version: u64,
    pub seq: u64,
}

#[derive(CandidType, Deserialize, Clone)]
pub struct ItemRecord {
    pub id: ByteBuf,
    pub version: u64,
    pub seq: u64,
    pub updated: u64,
    pub deleted: Option<u64>,
    pub purged: bool,
    pub wrapped_dek: ByteBuf,
    pub header: ByteBuf,
}

#[derive(CandidType, Deserialize)]
pub struct SyncPage {
    pub items: Vec<ItemRecord>,
    pub next: Option<u64>,
    pub latest_seq: u64,
    pub quota_used: u64,
    pub quota: u64,
}

#[derive(CandidType, Deserialize)]
pub struct Body {
    pub version: u64,
    pub body: ByteBuf,
}

#[derive(CandidType, Deserialize)]
pub struct RekeyArgs {
    pub expected_seq: u64,
    pub device_slot: Slot,
    pub kit_slot: Slot,
    pub kit_principal: Principal,
    pub export_header: ByteBuf,
    pub vault_auth_pub: ByteBuf,
    pub wrapped_deks: Vec<(ByteBuf, u64, ByteBuf)>,
}

#[derive(CandidType, Deserialize)]
pub struct BackupManifest {
    pub vault_id: ByteBuf,
    pub vk_epoch: u32,
    pub kit_number: u32,
    pub kit_created: u64,
    pub kit_slot: Slot,
    pub kit_principal: Principal,
    pub export_header: ByteBuf,
    pub vault_auth_pub: ByteBuf,
    pub latest_seq: u64,
    pub item_count: u32,
}

#[derive(CandidType, Deserialize)]
pub struct Status {
    pub version: String,
    pub environment: String,
    pub vault_id: ByteBuf,
    pub health: Health,
    pub kit_number: u32,
    pub kit_created: u64,
    pub recovery_pending: bool,
}

#[derive(CandidType, Deserialize)]
pub struct TrustInfo {
    pub balance: Nat,
    pub liquid: Nat,
    pub burn_per_day: Option<Nat>,
    pub quota_used: u64,
    pub quota: u64,
    pub item_count: u32,
    pub recovery_delay: u64,
    pub sessions: u8,
}

pub type HeaderField = (String, String);

#[derive(CandidType, Deserialize)]
pub struct HttpRequest {
    pub method: String,
    pub url: String,
    pub headers: Vec<HeaderField>,
    pub body: ByteBuf,
    pub certificate_version: Option<u16>,
}

#[derive(CandidType, Deserialize)]
pub struct HttpResponse {
    pub status_code: u16,
    pub headers: Vec<HeaderField>,
    pub body: ByteBuf,
    pub upgrade: Option<bool>,
}

// ---------- State, on the heap only ----------

const MINUTE: u64 = 60_000_000_000;
const SESSION_IDLE: u64 = 30 * MINUTE;
const SESSION_MAX: u64 = 8 * 60 * MINUTE;
const QUOTA: u64 = 1 << 30;
const DAY: u64 = 24 * 60 * MINUTE;
/// ADR 0012: the label of an unlock key. Nonce 12, padded text 64, tag 16.
const LABEL_CT_LEN: usize = 92;
/// ADR 0012: the claim code is 32 random bytes.
const CLAIM_CODE_LEN: usize = 32;
const MAX_FULL_SESSIONS: usize = 5;

struct StoredSlot {
    pub slot: Slot,
    pub nonce: u64,
    pub created: u64,
    pub last_used: Option<u64>,
    pub failed_row: u32,
    pub uncleared: u32,
    pub locked_until: Option<u64>,
}

struct Session {
    /// The last activity: when it was opened, or its last `renew`. ADR 0012.
    pub last_active: u64,
    pub idle_until: u64,
    pub ends: u64,
}

struct StoredItem {
    pub record: ItemRecord,
    pub body: ByteBuf,
}

#[derive(Default)]
struct State {
    pub claim_hash: Option<Vec<u8>>,
    pub environment: String,
    pub vault_id: Vec<u8>,
    pub owner: Option<Principal>,
    pub kit_principal: Option<Principal>,
    pub setup_done: bool,
    pub kit_created: u64,
    pub slots: BTreeMap<Vec<u8>, StoredSlot>,
    pub sessions: BTreeMap<Principal, Session>,
    pub items: BTreeMap<Vec<u8>, StoredItem>,
    pub seq: u64,
    pub quota_used: u64,
    pub last_unlock: Option<u64>,
    pub failed_since: u32,
    /// The times of the last ten counted wrong tries in the vault, section 3.3.
    pub wrong_times: Vec<u64>,
}

thread_local! {
    static STATE: RefCell<State> = RefCell::new(State::default());
}

// ---------- Encodings of spec 6.3 ----------

fn lp(x: &[u8]) -> Vec<u8> {
    let mut v = (x.len() as u32).to_be_bytes().to_vec();
    v.extend_from_slice(x);
    v
}

fn kind_byte(k: SlotKind) -> u8 {
    match k {
        SlotKind::PasskeyPin => 2,
        SlotKind::Kit => 4,
    }
}

fn opt(b: &Option<ByteBuf>) -> &[u8] {
    b.as_ref().map(|x| x.as_slice()).unwrap_or(&[])
}

/// ADR 0004. Never covers a proof key or the blob.
fn settings_hash(vid: &[u8], s: &Slot) -> Vec<u8> {
    let mut m = lp(b"ic-vault/v1/slot-settings");
    m.extend(lp(vid));
    m.extend_from_slice(&s.id);
    m.push(kind_byte(s.kind));
    m.extend(lp(opt(&s.credential_id)));
    m.extend(lp(opt(&s.prf_salt)));
    let (salt, mem, passes, lanes): (&[u8], u32, u32, u32) = match &s.kdf {
        Some(k) => (k.salt.as_slice(), k.memory_kib, k.passes, k.lanes),
        None => (&[], 0, 0, 0),
    };
    m.extend(lp(salt));
    m.extend_from_slice(&mem.to_be_bytes());
    m.extend_from_slice(&passes.to_be_bytes());
    m.extend_from_slice(&lanes.to_be_bytes());
    m.extend_from_slice(&s.salt);
    Sha256::digest(&m).to_vec()
}

fn unlock_msg(vid: &[u8], caller: &Principal, a: &UnlockArgs) -> Vec<u8> {
    let mut m = lp(b"ic-vault/v1/unlock");
    m.extend(lp(vid));
    m.extend(lp(canister_self().as_slice()));
    m.extend(lp(caller.as_slice()));
    m.extend_from_slice(&a.slot_id);
    m.extend_from_slice(&a.nonce.to_be_bytes());
    m.extend(lp(a.session.as_slice()));
    m
}

fn verify(pubkey: &[u8], msg: &[u8], sig: &[u8]) -> bool {
    let Ok(pk) = <[u8; 32]>::try_from(pubkey) else {
        return false;
    };
    let Ok(sig) = <[u8; 64]>::try_from(sig) else {
        return false;
    };
    let Ok(vk) = VerifyingKey::from_bytes(&pk) else {
        return false;
    };
    vk.verify_strict(msg, &Signature::from_bytes(&sig)).is_ok()
}

fn valid_point(pubkey: &[u8]) -> bool {
    <[u8; 32]>::try_from(pubkey)
        .ok()
        .and_then(|b| VerifyingKey::from_bytes(&b).ok())
        .map(|k| !k.is_weak())
        .unwrap_or(false)
}

// ---------- Gates ----------

fn is_session(st: &mut State, who: &Principal, now: u64) -> bool {
    match st.sessions.get(who) {
        Some(s) if now < s.idle_until && now < s.ends => true,
        Some(_) => {
            st.sessions.remove(who);
            false
        }
        None => false,
    }
}

/// The one decision function for `unlock`, called by the hook and by the method.
/// It checks everything that costs nothing to check, and never the proof that contains the PIN.
fn unlock_precheck(st: &State, caller: &Principal, a: &UnlockArgs, now: u64) -> Result<(), Error> {
    if !st.setup_done {
        return Err(Error::WrongState("setup not done".into()));
    }
    let is_owner = st.owner.as_ref() == Some(caller);
    let is_kit = st.kit_principal.as_ref() == Some(caller);
    if !is_owner && !is_kit {
        return Err(Error::NotAllowed);
    }
    let s = st.slots.get(a.slot_id.as_slice()).ok_or(Error::NotFound)?;
    if is_kit {
        // The waiting period is not part of the spike.
        return Err(Error::NotAllowed);
    }
    if a.nonce != s.nonce {
        return Err(Error::StaleNonce(s.nonce));
    }
    if s.uncleared >= 30 {
        return Err(Error::Locked(None));
    }
    if let Some(t) = s.locked_until {
        if now < t {
            return Err(Error::Locked(Some(t)));
        }
    }
    let msg = unlock_msg(&st.vault_id, caller, a);
    let ok = match s.slot.kind {
        SlotKind::PasskeyPin => verify(opt(&s.slot.auth_f_pub), &msg, opt(&a.proof_f)),
        SlotKind::Kit => verify(&s.slot.auth_pub, &msg, &a.proof),
    };
    if ok {
        Ok(())
    } else {
        Err(Error::NotAllowed)
    }
}

#[ic_cdk::inspect_message]
fn inspect_message() {
    let method = msg_method_name();
    let caller = msg_caller();
    let now = time();
    let ok = STATE.with(|s| {
        let mut st = s.borrow_mut();
        match method.as_str() {
            // Queries called as updates would cost the vault cycles.
            "http_request" | "status" | "slots_public" | "item_body" | "audit" | "trust" => false,
            // ADR 0012: the code has 256 bits, so the hook may compare it.
            "claim" => match candid::decode_one::<ByteBuf>(&msg_arg_data()) {
                Ok(code) => claim_precheck(&st, &caller, &code).is_ok(),
                Err(_) => false,
            },
            "setup_finish" => st.owner == Some(caller) && !st.setup_done,
            "unlock" => match candid::decode_one::<UnlockArgs>(&msg_arg_data()) {
                Ok(a) => unlock_precheck(&st, &caller, &a, now).is_ok(),
                Err(_) => false,
            },
            "enroll_begin" => st.owner == Some(caller) && st.setup_done,
            _ => is_session(&mut st, &caller, now),
        }
    });
    if ok {
        accept_message();
    }
}

fn session_gate() -> Result<(), Error> {
    let caller = msg_caller();
    let now = time();
    STATE.with(|s| {
        if is_session(&mut s.borrow_mut(), &caller, now) {
            Ok(())
        } else {
            Err(Error::NotAllowed)
        }
    })
}

fn not_in_spike() -> Error {
    Error::Invalid("not in the spike".into())
}

// ---------- Methods ----------

#[ic_cdk::init]
fn init(args: InstallArgs) {
    let InstallArgs::Init {
        claim_hash,
        host_suffix,
        environment,
        local_root_key,
    } = args
    else {
        ic_cdk::trap("init needs Init");
    };
    // ADR 0003: the three refusals.
    let local = environment == "local";
    if local_root_key.is_some() && !local {
        ic_cdk::trap("a local root key needs the environment local");
    }
    if local && local_root_key.is_none() {
        ic_cdk::trap("the environment local needs a root key");
    }
    let host = host_suffix.split(':').next().unwrap_or("");
    if local && host != "localhost" {
        ic_cdk::trap("the environment local needs the host suffix localhost");
    }
    if claim_hash.len() != 32 {
        ic_cdk::trap("claim hash must be 32 bytes");
    }
    STATE.with(|s| {
        let mut st = s.borrow_mut();
        st.claim_hash = Some(claim_hash.into_vec());
        st.environment = environment;
        st.vault_id = canister_self().as_slice().to_vec();
    });
}

#[ic_cdk::query]
fn http_request(_req: HttpRequest) -> HttpResponse {
    HttpResponse {
        status_code: 404,
        headers: vec![],
        body: ByteBuf::new(),
        upgrade: None,
    }
}

#[ic_cdk::query]
fn status() -> Status {
    STATE.with(|s| {
        let st = s.borrow();
        Status {
            version: "spike".into(),
            environment: st.environment.clone(),
            vault_id: ByteBuf::from(st.vault_id.clone()),
            health: Health::Unknown,
            kit_number: 1,
            kit_created: st.kit_created,
            recovery_pending: false,
        }
    })
}

/// The one decision function for `claim`, called by the hook and by the method. ADR 0012.
fn claim_precheck(st: &State, caller: &Principal, code: &[u8]) -> Result<(), Error> {
    if *caller == Principal::anonymous() || st.owner.is_some() || code.len() != CLAIM_CODE_LEN {
        return Err(Error::NotAllowed);
    }
    match &st.claim_hash {
        Some(h) if Sha256::digest(code).as_slice() == h.as_slice() => Ok(()),
        _ => Err(Error::NotAllowed),
    }
}

#[ic_cdk::update]
fn claim(code: ByteBuf) -> VResult {
    let caller = msg_caller();
    STATE.with(|s| {
        let mut st = s.borrow_mut();
        // The gate: the same function the hook ran. A call from another canister skipped the hook.
        if let Err(e) = claim_precheck(&st, &caller, &code) {
            return VResult::Err(e);
        }
        st.owner = Some(caller);
        st.claim_hash = None;
        VResult::Ok
    })
}

fn check_slot(s: &Slot, want: SlotKind) -> Result<(), Error> {
    let shape = s.kind == want
        && s.id.len() == 16
        && s.salt.len() == 32
        && s.label.len() == LABEL_CT_LEN
        && valid_point(&s.auth_pub)
        && match want {
            SlotKind::PasskeyPin => {
                s.credential_id.is_some()
                    && s.prf_salt.as_ref().map(|x| x.len()) == Some(32)
                    && s.kdf.is_some()
                    && s.auth_f_pub
                        .as_ref()
                        .map(|k| valid_point(k))
                        .unwrap_or(false)
            }
            SlotKind::Kit => {
                s.credential_id.is_none()
                    && s.prf_salt.is_none()
                    && s.kdf.is_none()
                    && s.auth_f_pub.is_none()
            }
        };
    if shape {
        Ok(())
    } else {
        Err(Error::Invalid("slot does not fit its kind".into()))
    }
}

fn open_session(st: &mut State, who: Principal, now: u64) -> Result<u64, Error> {
    st.sessions
        .retain(|_, s| now < s.idle_until && now < s.ends);
    // ADR 0012: a sixth full session is not refused. It ends the one with the oldest activity.
    // On a tie the one that was opened first, then the principal that sorts first.
    while st.sessions.len() >= MAX_FULL_SESSIONS && !st.sessions.contains_key(&who) {
        let oldest = st
            .sessions
            .iter()
            .min_by_key(|(p, s)| (s.last_active, s.ends, **p))
            .map(|(p, _)| *p);
        match oldest {
            Some(p) => st.sessions.remove(&p),
            None => break,
        };
    }
    st.sessions.insert(
        who,
        Session {
            last_active: now,
            idle_until: now + SESSION_IDLE,
            ends: now + SESSION_MAX,
        },
    );
    Ok(now + SESSION_IDLE)
}

#[ic_cdk::update]
fn setup_finish(a: SetupArgs) -> VResult {
    let caller = msg_caller();
    let now = time();
    STATE.with(|s| {
        let mut st = s.borrow_mut();
        if st.owner != Some(caller) {
            return VResult::Err(Error::NotAllowed);
        }
        if st.setup_done {
            return VResult::Err(Error::WrongState("setup done".into()));
        }
        if let Err(e) = check_slot(&a.device_slot, SlotKind::PasskeyPin)
            .and(check_slot(&a.kit_slot, SlotKind::Kit))
        {
            return VResult::Err(e);
        }
        if !valid_point(&a.vault_auth_pub) || a.device_slot.id == a.kit_slot.id {
            return VResult::Err(Error::Invalid("bad setup".into()));
        }
        for slot in [a.device_slot, a.kit_slot] {
            let id = slot.id.to_vec();
            st.slots.insert(
                id,
                StoredSlot {
                    slot,
                    nonce: 0,
                    created: now,
                    last_used: None,
                    failed_row: 0,
                    uncleared: 0,
                    locked_until: None,
                },
            );
        }
        st.kit_principal = Some(a.kit_principal);
        st.kit_created = now;
        st.setup_done = true;
        match open_session(&mut st, a.session, now) {
            Ok(_) => VResult::Ok,
            Err(e) => VResult::Err(e),
        }
    })
}

#[ic_cdk::query]
fn slots_public() -> SlotsPublic {
    let caller = msg_caller();
    STATE.with(|s| {
        let st = s.borrow();
        if st.owner != Some(caller) && st.kit_principal != Some(caller) {
            ic_cdk::trap("not allowed");
        }
        SlotsPublic {
            setup_done: st.setup_done,
            slots: st
                .slots
                .values()
                .map(|x| SlotInfo {
                    id: x.slot.id.clone(),
                    kind: x.slot.kind,
                    credential_id: x.slot.credential_id.clone(),
                    prf_salt: x.slot.prf_salt.clone(),
                    kdf: x.slot.kdf.clone(),
                    salt: x.slot.salt.clone(),
                    nonce: x.nonce,
                    created: x.created,
                    last_used: x.last_used,
                    locked_until: x.locked_until,
                    needs_clearing: x.uncleared >= 30,
                    label: x.slot.label.clone(),
                })
                .collect(),
            vk_epoch: 1,
            kit_number: 1,
            kit_created: st.kit_created,
            pending_recovery: None,
            health: Health::Unknown,
        }
    })
}

#[ic_cdk::update]
fn unlock(a: UnlockArgs) -> Result<UnlockOk, Error> {
    let caller = msg_caller();
    let now = time();
    STATE.with(|s| {
        let mut guard = s.borrow_mut();
        let st = &mut *guard;
        // The gate: the same function the hook ran. A call from another canister skipped the hook.
        unlock_precheck(st, &caller, &a, now)?;
        let vid = st.vault_id.clone();
        let msg = unlock_msg(&vid, &caller, &a);
        let id = a.slot_id.to_vec();
        let slot = st.slots.get_mut(&id).ok_or(Error::NotFound)?;
        // Write the failed try first, then check, then take it back on success.
        slot.nonce += 1;
        slot.failed_row += 1;
        slot.uncleared += 1;
        let counted = slot.slot.kind == SlotKind::PasskeyPin;
        let good = !counted || verify(&slot.slot.auth_pub, &msg, &a.proof);
        if !good {
            let row = slot.failed_row;
            if row >= 5 {
                let level = (row - 5).min(11);
                slot.locked_until = Some(now + (MINUTE << level).min(DAY));
            }
            // The second limit: the wrong tries of the whole vault within thirty days. The ten
            // stored times are counted before this try is written, and this try is added.
            let in_vault = 1 + st
                .wrong_times
                .iter()
                .filter(|t| now - **t < 30 * DAY)
                .count() as u32;
            st.wrong_times.push(now);
            if st.wrong_times.len() > 10 {
                st.wrong_times.remove(0);
            }
            if in_vault > 10 {
                slot.locked_until = Some(now + DAY);
            }
            // ADR 0012: the wrong tries this key still takes until it is locked, by whichever
            // of the three limits is closest. Never below zero, and zero exactly when the key is locked.
            let left = 5u32
                .saturating_sub(row)
                .min(11u32.saturating_sub(in_vault))
                .min(30u32.saturating_sub(slot.uncleared));
            let err = Error::BadProof {
                tries_left: left as u8,
                settings_hash: ByteBuf::from(settings_hash(&vid, &slot.slot)),
            };
            st.failed_since += 1;
            return Err(err);
        }
        slot.failed_row = 0;
        slot.uncleared -= 1;
        slot.locked_until = None;
        slot.last_used = Some(now);
        let wrapped = slot.slot.wrapped.clone();
        let session_expires = open_session(st, a.session, now)?;
        let previous_unlock = st.last_unlock.replace(now);
        let failed_since = std::mem::take(&mut st.failed_since);
        let uncleared = st
            .slots
            .values()
            .filter(|x| x.uncleared > 0)
            .map(|x| (x.slot.id.clone(), x.uncleared))
            .collect();
        Ok(UnlockOk {
            wrapped,
            session_expires,
            previous_unlock,
            failed_since,
            uncleared,
            alerts: vec![],
        })
    })
}

#[ic_cdk::update]
fn enroll_begin(_a: EnrollArgs) -> Result<EnrollOk, Error> {
    Err(not_in_spike())
}

#[ic_cdk::update]
fn enroll_finish(_slot: Slot, _proof: ByteBuf) -> VResult {
    VResult::Err(not_in_spike())
}

#[ic_cdk::update]
fn renew() -> u64 {
    let caller = msg_caller();
    let now = time();
    STATE.with(|s| {
        let mut st = s.borrow_mut();
        if !is_session(&mut st, &caller, now) {
            ic_cdk::trap("not allowed");
        }
        let sess = st.sessions.get_mut(&caller).unwrap();
        sess.last_active = now;
        sess.idle_until = (now + SESSION_IDLE).min(sess.ends);
        sess.idle_until
    })
}

#[ic_cdk::update]
fn lock() {
    let caller = msg_caller();
    STATE.with(|s| {
        s.borrow_mut().sessions.remove(&caller);
    })
}

#[ic_cdk::update]
fn sync(from: u64) -> SyncPage {
    if session_gate().is_err() {
        ic_cdk::trap("not allowed");
    }
    STATE.with(|s| {
        let st = s.borrow();
        let mut items: Vec<ItemRecord> = st
            .items
            .values()
            .filter(|i| i.record.seq > from)
            .map(|i| i.record.clone())
            .collect();
        items.sort_by_key(|r| r.seq);
        SyncPage {
            items,
            next: None,
            latest_seq: st.seq,
            quota_used: st.quota_used,
            quota: QUOTA,
        }
    })
}

#[ic_cdk::update]
fn item_put(p: ItemPut) -> Result<ItemPutOk, Error> {
    session_gate()?;
    let now = time();
    if p.id.len() != 16 || p.wrapped_dek.len() != 60 {
        return Err(Error::Invalid("bad item".into()));
    }
    let hl = p.header.len();
    if hl < 28 + 256 || (hl - 28) % 256 != 0 || hl - 28 > 3840 {
        return Err(Error::TooLarge);
    }
    let bl = p.body.len().saturating_sub(28);
    if !(10..=18).contains(&bl.trailing_zeros()) || !bl.is_power_of_two() {
        return Err(Error::TooLarge);
    }
    STATE.with(|s| {
        let mut st = s.borrow_mut();
        let id = p.id.to_vec();
        let current = st.items.get(&id).map(|i| i.record.version).unwrap_or(0);
        if current != p.expected_version {
            return Err(Error::VersionConflict(current));
        }
        let size = (p.header.len() + p.body.len() + p.wrapped_dek.len()) as u64;
        if st.quota_used + size > QUOTA {
            return Err(Error::QuotaExceeded);
        }
        st.seq += 1;
        st.quota_used += size;
        let (version, seq) = (current + 1, st.seq);
        let record = ItemRecord {
            id: p.id,
            version,
            seq,
            updated: now,
            deleted: None,
            purged: false,
            wrapped_dek: p.wrapped_dek,
            header: p.header,
        };
        st.items.insert(
            id,
            StoredItem {
                record,
                body: p.body,
            },
        );
        Ok(ItemPutOk { version, seq })
    })
}

#[ic_cdk::update]
fn item_delete(_id: ByteBuf, _version: u64) -> VResult {
    VResult::Err(session_gate().err().unwrap_or_else(not_in_spike))
}

#[ic_cdk::update]
fn item_restore(_id: ByteBuf, _version: u64) -> VResult {
    VResult::Err(session_gate().err().unwrap_or_else(not_in_spike))
}

#[ic_cdk::update]
fn item_purge(_id: ByteBuf) -> VResult {
    VResult::Err(session_gate().err().unwrap_or_else(not_in_spike))
}

#[ic_cdk::query]
fn item_body(id: ByteBuf) -> Option<Body> {
    if session_gate().is_err() {
        ic_cdk::trap("not allowed");
    }
    STATE.with(|s| {
        s.borrow().items.get(id.as_slice()).map(|i| Body {
            version: i.record.version,
            body: i.body.clone(),
        })
    })
}

#[ic_cdk::update]
fn slot_add(_slot: Slot, _replace: Option<ByteBuf>) -> VResult {
    VResult::Err(session_gate().err().unwrap_or_else(not_in_spike))
}

#[ic_cdk::update]
fn slot_remove(_id: ByteBuf) -> VResult {
    VResult::Err(session_gate().err().unwrap_or_else(not_in_spike))
}

#[ic_cdk::update]
fn slot_clear_lock(id: ByteBuf) -> VResult {
    if let Err(e) = session_gate() {
        return VResult::Err(e);
    }
    STATE.with(|s| match s.borrow_mut().slots.get_mut(id.as_slice()) {
        Some(x) => {
            x.uncleared = 0;
            x.failed_row = 0;
            x.locked_until = None;
            VResult::Ok
        }
        None => VResult::Err(Error::NotFound),
    })
}

#[ic_cdk::update]
fn handoff_put(_h: Handoff) -> VResult {
    VResult::Err(session_gate().err().unwrap_or_else(not_in_spike))
}

#[ic_cdk::update]
fn kit_replace(_k: KitReplace) -> VResult {
    VResult::Err(session_gate().err().unwrap_or_else(not_in_spike))
}

#[ic_cdk::update]
fn recovery_cancel() -> VResult {
    VResult::Err(session_gate().err().unwrap_or_else(not_in_spike))
}

#[ic_cdk::update]
fn owner_set(_p: Principal) -> VResult {
    VResult::Err(session_gate().err().unwrap_or_else(not_in_spike))
}

#[ic_cdk::update]
fn rekey(_a: RekeyArgs) -> VResult {
    VResult::Err(session_gate().err().unwrap_or_else(not_in_spike))
}

#[ic_cdk::update]
fn backup_manifest() -> BackupManifest {
    ic_cdk::trap("not in the spike")
}

#[ic_cdk::update]
fn alerts_ack(_seq: u64) -> VResult {
    VResult::Err(session_gate().err().unwrap_or_else(not_in_spike))
}

#[ic_cdk::update]
fn limits_set(_delay: u64) -> VResult {
    VResult::Err(session_gate().err().unwrap_or_else(not_in_spike))
}

#[ic_cdk::query]
fn audit(_from: u64, _count: u32) -> Vec<AuditEvent> {
    if session_gate().is_err() {
        ic_cdk::trap("not allowed");
    }
    vec![]
}

#[ic_cdk::query]
fn trust() -> TrustInfo {
    if session_gate().is_err() {
        ic_cdk::trap("not allowed");
    }
    STATE.with(|s| {
        let st = s.borrow();
        TrustInfo {
            balance: Nat::from(ic_cdk::api::canister_cycle_balance()),
            liquid: Nat::from(ic_cdk::api::canister_liquid_cycle_balance()),
            burn_per_day: None,
            quota_used: st.quota_used,
            quota: QUOTA,
            item_count: st.items.len() as u32,
            recovery_delay: 48 * 60 * MINUTE,
            sessions: st.sessions.len() as u8,
        }
    })
}

ic_cdk::export_candid!();

#[cfg(test)]
mod tests {
    use candid_parser::utils::{service_equal, CandidSource};
    use std::path::Path;

    /// The interface this canister exports is the interface of section 3.2, method for method and field for field.
    #[test]
    fn exported_interface_equals_vault_did() {
        let ours = super::__export_service();
        let did = Path::new(env!("CARGO_MANIFEST_DIR")).join("../vault.did");
        service_equal(CandidSource::Text(&ours), CandidSource::File(&did))
            .expect("the canister and vault.did differ");
    }
}
