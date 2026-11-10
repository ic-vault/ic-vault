//! Spike SP9: one thin path, end to end, against a local test network (PocketIC).
//! Build the canister first: cargo build -p thin-slice-canister --target wasm32-unknown-unknown --release

use candid::{decode_one, encode_args, encode_one, Principal};
use ed25519_dalek::SigningKey;
use pocket_ic::PocketIc;
use serde_bytes::ByteBuf;
use thin_slice_canister::*;
use thin_slice_driver::*;

const KIND_PASSKEY_PIN: u8 = 2;
const KIND_KIT: u8 = 4;
const ZERO32: [u8; 32] = [0u8; 32];

/// Generated test bytes. Nothing here is a real secret.
fn bytes<const N: usize>(tag: u8) -> [u8; N] {
    let mut out = [0u8; N];
    let mut block = sha256(&[b's', b'p', b'9', tag]);
    for (i, b) in out.iter_mut().enumerate() {
        if i % 32 == 0 && i > 0 {
            block = sha256(&block);
        }
        *b = block[i % 32];
    }
    out
}

fn principal_of(seed_tag: u8) -> Principal {
    let key = SigningKey::from_bytes(&bytes::<32>(seed_tag));
    Principal::self_authenticating(key.verifying_key().as_bytes())
}

fn bb(x: &[u8]) -> ByteBuf {
    ByteBuf::from(x.to_vec())
}

struct Device {
    f: [u8; 32],
    slot: Slot,
}

/// ADR 0012: the label of an unlock key pads to 64 bytes, so the ciphertext has 92.
fn label_ct(vid: &[u8], vk: &[u8; 32], slot_id: &[u8], nonce_tag: u8, name: &str) -> Vec<u8> {
    assert!(name.len() <= 60, "a label holds at most 60 bytes of text");
    let k_meta = hkdf32(vk, vid, b"ic-vault/v1/meta");
    let aad = cat(&[&lp(b"ic-vault/v1/label"), &lp(vid), slot_id]);
    let ct = seal(
        &k_meta,
        &bytes::<12>(nonce_tag),
        &pad(name.as_bytes(), 64),
        &aad,
    );
    assert_eq!(ct.len(), 92);
    ct
}

fn build_device_slot(vid: &[u8], vk: &[u8; 32], pin: &str) -> Device {
    let f = bytes::<32>(10); // stands in for the PRF output of the passkey
    let id = bytes::<16>(11);
    let salt = bytes::<32>(12);
    let kdf = Kdf {
        memory_kib: 64 * 1024,
        passes: 3,
        lanes: 1,
        salt: bb(&bytes::<16>(13)),
    };
    let p = argon2id(
        pin.as_bytes(),
        &kdf.salt,
        kdf.memory_kib,
        kdf.passes,
        kdf.lanes,
    );
    let keys = slot_keys(&f, &p, &salt, vid, &id, KIND_PASSKEY_PIN);
    let wrapped = seal(
        &keys.kek,
        &bytes::<12>(14),
        vk,
        &slot_inner_aad(vid, &id, 1),
    );
    let slot = Slot {
        id: bb(&id),
        kind: SlotKind::PasskeyPin,
        credential_id: Some(bb(&bytes::<20>(15))),
        prf_salt: Some(bb(&bytes::<32>(16))),
        kdf: Some(kdf),
        salt: bb(&salt),
        auth_f_pub: Some(bb(keys.auth_f.verifying_key().as_bytes())),
        auth_pub: bb(keys.auth.verifying_key().as_bytes()),
        wrap: 1,
        wrapped: bb(&wrapped),
        label: bb(&label_ct(vid, vk, &id, 17, "Test phone")),
    };
    Device { f, slot }
}

fn build_kit_slot(vid: &[u8], vk: &[u8; 32], kit_secret: &[u8; 32]) -> (Slot, Principal, Vec<u8>) {
    let kit_factor = hkdf32(kit_secret, vid, b"ic-vault/v1/kit-factor");
    let kit_id = SigningKey::from_bytes(&hkdf32(kit_secret, vid, b"ic-vault/v1/kit-identity"));
    let kit_export = hkdf32(kit_secret, vid, b"ic-vault/v1/kit-export");
    let id = bytes::<16>(21);
    let salt = bytes::<32>(22);
    let keys = slot_keys(&kit_factor, &ZERO32, &salt, vid, &id, KIND_KIT);
    let wrapped = seal(
        &keys.kek,
        &bytes::<12>(23),
        vk,
        &slot_inner_aad(vid, &id, 1),
    );
    let slot = Slot {
        id: bb(&id),
        kind: SlotKind::Kit,
        credential_id: None,
        prf_salt: None,
        kdf: None,
        salt: bb(&salt),
        auth_f_pub: None,
        auth_pub: bb(keys.auth.verifying_key().as_bytes()),
        wrap: 1,
        wrapped: bb(&wrapped),
        label: bb(&label_ct(vid, vk, &id, 25, "")),
    };
    let export_header = seal(
        &kit_export,
        &bytes::<12>(24),
        vk,
        &cat(&[&lp(b"ic-vault/v1/export"), &lp(vid)]),
    );
    (
        slot,
        Principal::self_authenticating(kit_id.verifying_key().as_bytes()),
        export_header,
    )
}

struct Proofs {
    proof_f: Vec<u8>,
    proof: Vec<u8>,
    kek: [u8; 32],
}

/// The slow step: the PIN through Argon2id. The tests do it once per PIN.
fn pin_key(info: &SlotInfo, pin: &str) -> [u8; 32] {
    let kdf = info.kdf.as_ref().unwrap();
    argon2id(
        pin.as_bytes(),
        &kdf.salt,
        kdf.memory_kib,
        kdf.passes,
        kdf.lanes,
    )
}

fn prove(
    vid: &[u8],
    canister: Principal,
    caller: Principal,
    info: &SlotInfo,
    f: &[u8; 32],
    pin: &str,
    session: Principal,
) -> Proofs {
    prove_with(vid, canister, caller, info, f, &pin_key(info, pin), session)
}

fn prove_with(
    vid: &[u8],
    canister: Principal,
    caller: Principal,
    info: &SlotInfo,
    f: &[u8; 32],
    p: &[u8; 32],
    session: Principal,
) -> Proofs {
    let keys = slot_keys(f, p, &info.salt, vid, &info.id, KIND_PASSKEY_PIN);
    let msg = unlock_msg(
        vid,
        canister.as_slice(),
        caller.as_slice(),
        &info.id,
        info.nonce,
        session.as_slice(),
    );
    Proofs {
        proof_f: sign(&keys.auth_f, &msg),
        proof: sign(&keys.auth, &msg),
        kek: keys.kek,
    }
}

fn client_settings_hash(vid: &[u8], i: &SlotInfo) -> Vec<u8> {
    let k = i.kdf.as_ref().unwrap();
    sha256(&cat(&[
        &lp(b"ic-vault/v1/slot-settings"),
        &lp(vid),
        &i.id,
        &[KIND_PASSKEY_PIN],
        &lp(i.credential_id.as_ref().unwrap()),
        &lp(i.prf_salt.as_ref().unwrap()),
        &lp(&k.salt),
        &k.memory_kib.to_be_bytes(),
        &k.passes.to_be_bytes(),
        &k.lanes.to_be_bytes(),
        &i.salt,
    ]))
}

fn wasm() -> Vec<u8> {
    let p = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../target/wasm32-unknown-unknown/release/thin_slice_canister.wasm");
    std::fs::read(&p).unwrap_or_else(|_| {
        panic!(
            "build the canister first, see the top of this file: {}",
            p.display()
        )
    })
}

fn install(pic: &PocketIc, args: InstallArgs) -> Principal {
    let id = pic.create_canister();
    pic.add_cycles(id, 2_000_000_000_000);
    pic.install_canister(id, wasm(), encode_one(args).unwrap(), None);
    id
}

fn device_info(pic: &PocketIc, can: Principal, owner: Principal, slot_id: &[u8]) -> SlotInfo {
    let sp: SlotsPublic = decode_one(
        &pic.query_call(can, owner, "slots_public", encode_args(()).unwrap())
            .unwrap(),
    )
    .unwrap();
    assert!(sp.setup_done);
    sp.slots
        .into_iter()
        .find(|s| s.id.as_slice() == slot_id)
        .expect("device slot listed")
}

fn unlock(
    pic: &PocketIc,
    can: Principal,
    caller: Principal,
    info: &SlotInfo,
    session: Principal,
    pr: &Proofs,
) -> Result<Result<UnlockOk, Error>, pocket_ic::RejectResponse> {
    let args = UnlockArgs {
        slot_id: info.id.clone(),
        nonce: info.nonce,
        session,
        proof_f: Some(bb(&pr.proof_f)),
        proof: bb(&pr.proof),
    };
    pic.update_call(can, caller, "unlock", encode_one(args).unwrap())
        .map(|r| decode_one(&r).unwrap())
}

#[test]
fn install_refusals_of_adr_0003() {
    let pic = PocketIc::new();
    let try_install = |environment: &str, host: &str, key: Option<ByteBuf>| {
        let id = pic.create_canister();
        pic.add_cycles(id, 2_000_000_000_000);
        let args = InstallArgs::Init {
            claim_hash: bb(&[0u8; 32]),
            host_suffix: host.into(),
            environment: environment.into(),
            local_root_key: key,
        };
        std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            pic.install_canister(id, wasm(), encode_one(args).unwrap(), None)
        }))
        .is_ok()
    };
    assert!(try_install("production", "icp.net", None));
    assert!(try_install("local", "localhost:8000", Some(bb(&[1u8; 96]))));
    assert!(
        !try_install("production", "icp.net", Some(bb(&[1u8; 96]))),
        "a root key outside local must be refused"
    );
    assert!(
        !try_install("local", "localhost:8000", None),
        "local without a root key must be refused"
    );
    assert!(
        !try_install("local", "icp.net", Some(bb(&[1u8; 96]))),
        "local on another host must be refused"
    );
}

#[test]
fn thin_slice_end_to_end() {
    let pic = PocketIc::new();
    let claim_code = bytes::<32>(1);
    let can = install(
        &pic,
        InstallArgs::Init {
            claim_hash: bb(&sha256(&claim_code)),
            host_suffix: "localhost:8000".into(),
            environment: "local".into(),
            local_root_key: Some(bb(&[1u8; 96])),
        },
    );
    let vid = can.as_slice().to_vec();
    let owner = principal_of(2);
    let stranger = principal_of(3);
    let pin = "482913";

    // Status is public.
    let st: Status = decode_one(
        &pic.query_call(
            can,
            Principal::anonymous(),
            "status",
            encode_args(()).unwrap(),
        )
        .unwrap(),
    )
    .unwrap();
    assert_eq!(st.environment, "local");
    assert_eq!(st.vault_id.as_slice(), vid.as_slice());
    assert!(!st.recovery_pending);

    // Claim: wrong code, right code, second claim. ADR 0012: the hook checks the code.
    let claim = |who: Principal, code: &[u8]| -> VResult {
        decode_one(
            &pic.update_call(can, who, "claim", encode_one(bb(code)).unwrap())
                .unwrap(),
        )
        .unwrap()
    };
    assert!(
        pic.update_call(can, stranger, "claim", encode_one(bb(&[9u8; 32])).unwrap())
            .is_err(),
        "the hook turns a wrong claim code away"
    );
    assert!(
        pic.update_call(
            can,
            owner,
            "claim",
            encode_one(bb(&claim_code[..31])).unwrap()
        )
        .is_err(),
        "and a code of another length"
    );
    assert!(matches!(claim(owner, &claim_code), VResult::Ok));
    assert!(
        pic.update_call(can, stranger, "claim", encode_one(bb(&claim_code)).unwrap())
            .is_err(),
        "the hook turns a second claim away"
    );

    // Setup: vault key, kit slot, device slot.
    let vk = bytes::<32>(30);
    let kit_secret = bytes::<32>(31);
    let dev = build_device_slot(&vid, &vk, pin);
    let (kit_slot, kit_principal, export_header) = build_kit_slot(&vid, &vk, &kit_secret);
    let vault_auth = SigningKey::from_bytes(&hkdf32(&vk, &vid, b"ic-vault/v1/vault-auth"));
    let session0 = principal_of(40);
    let setup = SetupArgs {
        device_slot: dev.slot.clone(),
        kit_slot,
        kit_principal,
        export_header: bb(&export_header),
        vault_auth_pub: bb(vault_auth.verifying_key().as_bytes()),
        session: session0,
    };
    let r: VResult = decode_one(
        &pic.update_call(can, owner, "setup_finish", encode_one(setup).unwrap())
            .unwrap(),
    )
    .unwrap();
    assert!(matches!(r, VResult::Ok));

    // A stranger learns nothing about the slots.
    assert!(pic
        .query_call(can, stranger, "slots_public", encode_args(()).unwrap())
        .is_err());

    // Practice unlock, with one wrong PIN first.
    let session = principal_of(41);
    let info = device_info(&pic, can, owner, &dev.slot.id);
    assert_eq!(info.nonce, 0);
    let wrong = prove(&vid, can, owner, &info, &dev.f, "482914", session);
    match unlock(&pic, can, owner, &info, session, &wrong).expect("a wrong PIN reaches the method")
    {
        Err(Error::BadProof {
            tries_left,
            settings_hash,
        }) => {
            assert_eq!(tries_left, 4);
            assert_eq!(
                settings_hash.as_slice(),
                client_settings_hash(&vid, &info).as_slice(),
                "ADR 0004: both sides compute the same settings hash"
            );
        }
        other => panic!("expected BadProof, got {:?}", other.map(|_| ()).err()),
    }

    // The wrong try moved the nonce. A replay of the same call is turned away by the hook.
    assert!(
        unlock(&pic, can, owner, &info, session, &wrong).is_err(),
        "stale nonce is refused in the hook"
    );
    let info = device_info(&pic, can, owner, &dev.slot.id);
    assert_eq!(info.nonce, 1);

    // Without the passkey no try is counted: a wrong passkey proof never reaches the method.
    let other_f = bytes::<32>(50);
    let no_passkey = prove(&vid, can, owner, &info, &other_f, pin, session);
    assert!(
        unlock(&pic, can, owner, &info, session, &no_passkey).is_err(),
        "wrong passkey proof is refused in the hook"
    );
    assert_eq!(
        device_info(&pic, can, owner, &dev.slot.id).nonce,
        1,
        "and nothing was counted"
    );

    // A stranger with perfect proofs is not the owner.
    let as_stranger = prove(&vid, can, stranger, &info, &dev.f, pin, session);
    assert!(unlock(&pic, can, stranger, &info, session, &as_stranger).is_err());

    // The right passkey and the right PIN.
    let good = prove(&vid, can, owner, &info, &dev.f, pin, session);
    let ok = unlock(&pic, can, owner, &info, session, &good)
        .unwrap()
        .expect("unlock");
    assert_eq!(ok.failed_since, 1);
    assert_eq!(
        ok.uncleared.len(),
        1,
        "ADR 0010: the wrong try stays counted after a success"
    );
    assert_eq!(ok.uncleared[0].1, 1);
    let vk_back = open(&good.kek, &ok.wrapped, &slot_inner_aad(&vid, &info.id, 1))
        .expect("the slot blob opens");
    assert_eq!(vk_back, vk, "the vault key came back");
    let vk: [u8; 32] = vk_back.try_into().unwrap();

    // ADR 0013: the key list carries the label. With the vault key it opens to the name.
    let sp: SlotsPublic = decode_one(
        &pic.query_call(can, owner, "slots_public", encode_args(()).unwrap())
            .unwrap(),
    )
    .unwrap();
    let k_meta = hkdf32(&vk, &vid, b"ic-vault/v1/meta");
    let label_aad = |id: &[u8]| cat(&[&lp(b"ic-vault/v1/label"), &lp(&vid), id]);
    let names: Vec<Vec<u8>> = sp
        .slots
        .iter()
        .map(|s| {
            assert_eq!(s.label.len(), 92);
            unpad(&open(&k_meta, &s.label, &label_aad(&s.id)).expect("the label opens"))
        })
        .collect();
    let mut sorted = names.clone();
    sorted.sort();
    // The kit slot has the empty name.
    assert_eq!(sorted, [b"".to_vec(), b"Test phone".to_vec()]);
    assert!(
        open(&k_meta, &sp.slots[0].label, &label_aad(&sp.slots[1].id)).is_none(),
        "a label does not open under the id of another key"
    );

    // Own typo: clear it, as ADR 0010 lets the app do.
    let r: VResult = decode_one(
        &pic.update_call(
            can,
            session,
            "slot_clear_lock",
            encode_one(info.id.clone()).unwrap(),
        )
        .unwrap(),
    )
    .unwrap();
    assert!(matches!(r, VResult::Ok));

    // Save one item as the session.
    let k_wrap = hkdf32(&vk, &vid, b"ic-vault/v1/dek-wrap");
    let item_id = bytes::<16>(60);
    let dek = bytes::<32>(61);
    let header = br#"{"v":1,"t":"login","title":"Example","sub":"","tags":[],"fav":false}"#;
    let body = br#"{"v":1,"fields":[{"k":"password","v":"generated-test-value","secret":true}],"notes":"","urls":[],"history":[]}"#;
    let put = ItemPut {
        id: bb(&item_id),
        expected_version: 0,
        wrapped_dek: bb(&seal(
            &k_wrap,
            &bytes::<12>(62),
            &dek,
            &item_aad("ic-vault/v1/dek", &vid, &item_id, 1, None),
        )),
        header: bb(&seal(
            &dek,
            &bytes::<12>(63),
            &pad(header, 256),
            &item_aad("ic-vault/v1/item", &vid, &item_id, 1, Some(1)),
        )),
        body: bb(&seal(
            &dek,
            &bytes::<12>(64),
            &pad(body, 1024),
            &item_aad("ic-vault/v1/item", &vid, &item_id, 1, Some(2)),
        )),
    };
    let put_bytes = encode_one(put).unwrap();
    assert!(
        pic.update_call(can, owner, "item_put", put_bytes.clone())
            .is_err(),
        "the owner principal is not a session"
    );
    let r: Result<ItemPutOk, Error> = decode_one(
        &pic.update_call(can, session, "item_put", put_bytes.clone())
            .unwrap(),
    )
    .unwrap();
    let r = r.expect("item_put");
    assert_eq!((r.version, r.seq), (1, 1));
    let again: Result<ItemPutOk, Error> = decode_one(
        &pic.update_call(can, session, "item_put", put_bytes)
            .unwrap(),
    )
    .unwrap();
    assert!(matches!(again, Err(Error::VersionConflict(1))));

    // Read it back: sync as an update, the body as a query, both as the session.
    let page: SyncPage = decode_one(
        &pic.update_call(can, session, "sync", encode_one(0u64).unwrap())
            .unwrap(),
    )
    .unwrap();
    assert_eq!(page.items.len(), 1);
    let rec = &page.items[0];
    let dek_back: [u8; 32] = open(
        &k_wrap,
        &rec.wrapped_dek,
        &item_aad("ic-vault/v1/dek", &vid, &rec.id, rec.version, None),
    )
    .expect("data key")
    .try_into()
    .unwrap();
    let header_back = unpad(
        &open(
            &dek_back,
            &rec.header,
            &item_aad("ic-vault/v1/item", &vid, &rec.id, rec.version, Some(1)),
        )
        .expect("header"),
    );
    assert_eq!(header_back, header);
    let b: Option<Body> = decode_one(
        &pic.query_call(can, session, "item_body", encode_one(bb(&item_id)).unwrap())
            .unwrap(),
    )
    .unwrap();
    let b = b.expect("body");
    let body_back = unpad(
        &open(
            &dek_back,
            &b.body,
            &item_aad("ic-vault/v1/item", &vid, &rec.id, b.version, Some(2)),
        )
        .expect("body"),
    );
    assert_eq!(body_back, body);

    // A ciphertext moved to another version does not open.
    assert!(open(
        &dek_back,
        &b.body,
        &item_aad("ic-vault/v1/item", &vid, &rec.id, 2, Some(2))
    )
    .is_none());

    // Lock ends the session.
    pic.update_call(can, session, "lock", encode_args(()).unwrap())
        .unwrap();
    assert!(pic
        .update_call(can, session, "sync", encode_one(0u64).unwrap())
        .is_err());
}

// ---------- ADR 0012 ----------

const PIN: &str = "271828";
const WRONG_PIN: &str = "482914";
const SECOND: std::time::Duration = std::time::Duration::from_secs(1);

struct Vault {
    can: Principal,
    vid: Vec<u8>,
    owner: Principal,
    vk: [u8; 32],
}

fn installed(pic: &PocketIc, claim_hash: &[u8]) -> Principal {
    install(
        pic,
        InstallArgs::Init {
            claim_hash: bb(claim_hash),
            host_suffix: "icp.net".into(),
            environment: "production".into(),
            local_root_key: None,
        },
    )
}

fn claimed(pic: &PocketIc) -> Vault {
    let claim_code = bytes::<32>(1);
    let can = installed(pic, &sha256(&claim_code));
    let owner = principal_of(2);
    let r: VResult = decode_one(
        &pic.update_call(can, owner, "claim", encode_one(bb(&claim_code)).unwrap())
            .unwrap(),
    )
    .unwrap();
    assert!(matches!(r, VResult::Ok));
    Vault {
        can,
        vid: can.as_slice().to_vec(),
        owner,
        vk: bytes::<32>(30),
    }
}

fn setup_finish(
    pic: &PocketIc,
    v: &Vault,
    device_slot: Slot,
    kit_label: Option<Vec<u8>>,
) -> VResult {
    let (mut kit_slot, kit_principal, export_header) =
        build_kit_slot(&v.vid, &v.vk, &bytes::<32>(31));
    if let Some(l) = kit_label {
        kit_slot.label = bb(&l);
    }
    let vault_auth = SigningKey::from_bytes(&hkdf32(&v.vk, &v.vid, b"ic-vault/v1/vault-auth"));
    let setup = SetupArgs {
        device_slot,
        kit_slot,
        kit_principal,
        export_header: bb(&export_header),
        vault_auth_pub: bb(vault_auth.verifying_key().as_bytes()),
        session: principal_of(40),
    };
    decode_one(
        &pic.update_call(v.can, v.owner, "setup_finish", encode_one(setup).unwrap())
            .unwrap(),
    )
    .unwrap()
}

/// A vault after setup, with the device and the two PIN keys the tests need.
struct Ready {
    v: Vault,
    dev: Device,
    good_p: [u8; 32],
    wrong_p: [u8; 32],
}

fn ready(pic: &PocketIc) -> Ready {
    let v = claimed(pic);
    let dev = build_device_slot(&v.vid, &v.vk, PIN);
    assert!(matches!(
        setup_finish(pic, &v, dev.slot.clone(), None),
        VResult::Ok
    ));
    let info = device_info(pic, v.can, v.owner, &dev.slot.id);
    let good_p = pin_key(&info, PIN);
    let wrong_p = pin_key(&info, WRONG_PIN);
    Ready {
        v,
        dev,
        good_p,
        wrong_p,
    }
}

impl Ready {
    fn info(&self, pic: &PocketIc) -> SlotInfo {
        device_info(pic, self.v.can, self.v.owner, &self.dev.slot.id)
    }

    fn try_unlock(
        &self,
        pic: &PocketIc,
        p: &[u8; 32],
        who: Principal,
    ) -> Result<Result<UnlockOk, Error>, pocket_ic::RejectResponse> {
        pic.advance_time(SECOND);
        let info = self.info(pic);
        let pr = prove_with(
            &self.v.vid,
            self.v.can,
            self.v.owner,
            &info,
            &self.dev.f,
            p,
            who,
        );
        unlock(pic, self.v.can, self.v.owner, &info, who, &pr)
    }

    fn open_session(&self, pic: &PocketIc, who: Principal) {
        self.try_unlock(pic, &self.good_p, who)
            .unwrap()
            .expect("unlock");
    }

    /// One wrong PIN. Returns `tries_left`.
    fn wrong_try(&self, pic: &PocketIc) -> u8 {
        match self
            .try_unlock(pic, &self.wrong_p, principal_of(99))
            .expect("a wrong PIN reaches the method")
        {
            Err(Error::BadProof { tries_left, .. }) => tries_left,
            other => panic!("expected BadProof, got {:?}", other.map(|_| ()).err()),
        }
    }

    fn alive(&self, pic: &PocketIc, who: Principal) -> bool {
        pic.update_call(self.v.can, who, "sync", encode_one(0u64).unwrap())
            .is_ok()
    }

    fn renew(&self, pic: &PocketIc, who: Principal) {
        pic.update_call(self.v.can, who, "renew", encode_args(()).unwrap())
            .expect("renew");
    }
}

/// The claim code is 32 bytes. A code of another length is refused even when its hash is the stored one.
#[test]
fn a_claim_code_of_another_length_is_refused() {
    let pic = PocketIc::new();
    let short = bytes::<31>(1);
    let can = installed(&pic, &sha256(&short));
    assert!(
        pic.update_call(
            can,
            principal_of(2),
            "claim",
            encode_one(bb(&short)).unwrap()
        )
        .is_err(),
        "31 bytes with the right hash never reach the method"
    );
}

/// The canister refuses a slot whose label is not 92 bytes, on the device slot and on the kit slot.
#[test]
fn a_label_of_another_length_is_refused() {
    let pic = PocketIc::new();
    let v = claimed(&pic);
    let dev = build_device_slot(&v.vid, &v.vk, PIN);
    for len in [0usize, 44, 91, 93] {
        let mut slot = dev.slot.clone();
        slot.label = bb(&vec![0u8; len]);
        assert!(
            matches!(
                setup_finish(&pic, &v, slot, None),
                VResult::Err(Error::Invalid(_))
            ),
            "a device label of {len} bytes"
        );
        assert!(
            matches!(
                setup_finish(&pic, &v, dev.slot.clone(), Some(vec![0u8; len])),
                VResult::Err(Error::Invalid(_))
            ),
            "a kit label of {len} bytes"
        );
    }
    // The same call with both labels at 92 bytes is accepted.
    assert_eq!(dev.slot.label.len(), 92);
    assert!(matches!(
        setup_finish(&pic, &v, dev.slot.clone(), None),
        VResult::Ok
    ));
}

/// A sixth full session is not refused. It ends the one with the oldest activity.
#[test]
fn a_sixth_session_ends_the_one_with_the_oldest_activity() {
    let pic = PocketIc::new();
    let r = ready(&pic);
    let first = principal_of(40); // opened by setup
    let s: Vec<Principal> = (70..76).map(principal_of).collect();

    // Keep the first session alive for 7 hours and 45 minutes. It is now close to its eight hours.
    for _ in 0..31 {
        pic.advance_time(std::time::Duration::from_secs(15 * 60));
        r.renew(&pic, first);
    }
    for who in &s[..4] {
        r.open_session(&pic, *who);
    }
    assert!(r.alive(&pic, first) && s[..4].iter().all(|who| r.alive(&pic, *who)));

    // The first session is the oldest and the first to run out, but it was used last.
    pic.advance_time(SECOND);
    r.renew(&pic, first);
    // s[1] is used after s[0], s[2] and s[3] too. So s[0] has the oldest activity.
    for who in &s[1..4] {
        pic.advance_time(SECOND);
        r.renew(&pic, *who);
    }
    r.open_session(&pic, s[4]);
    assert!(
        !r.alive(&pic, s[0]),
        "the session with the oldest activity is gone"
    );
    for who in [first, s[1], s[2], s[3], s[4]] {
        assert!(r.alive(&pic, who), "the other sessions keep working");
    }

    // Once more. The first session renewed before s[1], s[2] and s[3] did, so it is next.
    r.open_session(&pic, s[5]);
    assert!(!r.alive(&pic, first));
    for who in [s[1], s[2], s[3], s[4], s[5]] {
        assert!(r.alive(&pic, who));
    }
}

/// `tries_left` by the first limit: five in a row, and the one try after a lock has run out.
#[test]
fn tries_left_five_in_a_row() {
    let pic = PocketIc::new();
    let r = ready(&pic);
    let left: Vec<u8> = (0..5).map(|_| r.wrong_try(&pic)).collect();
    assert_eq!(left, [4, 3, 2, 1, 0]);
    let minute = 60_000_000_000u64;
    let now = pic.get_time().as_nanos_since_unix_epoch();
    let until = r.info(&pic).locked_until.expect("locked for a minute");
    assert!(until > now && until <= now + minute);
    assert!(
        r.try_unlock(&pic, &r.good_p, principal_of(80)).is_err(),
        "a locked key is refused in the hook, even with the right PIN"
    );

    // After the lock there is one try. A wrong one is counted, answered with zero, and locks again.
    // The tick lets the test network take over the new time before the hook reads it.
    pic.advance_time(std::time::Duration::from_secs(61));
    pic.tick();
    let nonce = r.info(&pic).nonce;
    assert_eq!(r.wrong_try(&pic), 0);
    let info = r.info(&pic);
    assert_eq!(info.nonce, nonce + 1, "the try was counted");
    let now = pic.get_time().as_nanos_since_unix_epoch();
    let until = info.locked_until.expect("locked again");
    assert!(
        until > now + minute && until <= now + 2 * minute,
        "for twice as long"
    );
}

/// `tries_left` by the second limit: ten wrong tries of the vault within thirty days.
#[test]
fn tries_left_at_the_limit_of_the_vault() {
    let pic = PocketIc::new();
    let r = ready(&pic);
    let left: Vec<u8> = (0..4).map(|_| r.wrong_try(&pic)).collect();
    assert_eq!(left, [4, 3, 2, 1], "five in a row is the closest limit");
    r.open_session(&pic, principal_of(80)); // a good unlock ends the row, not the count of the vault
    let left: Vec<u8> = (0..4).map(|_| r.wrong_try(&pic)).collect();
    assert_eq!(left, [4, 3, 2, 1]);
    r.open_session(&pic, principal_of(81));
    // Eight wrong tries in the vault so far.
    assert_eq!(
        r.wrong_try(&pic),
        2,
        "the ninth: the limit of the vault is now the closest"
    );
    assert_eq!(
        r.wrong_try(&pic),
        1,
        "the tenth is the last one before the 24 hours"
    );
    assert_eq!(r.wrong_try(&pic), 0, "the eleventh locks the key");
    let day = 24 * 3600 * 1_000_000_000u64;
    let now = pic.get_time().as_nanos_since_unix_epoch();
    let until = r.info(&pic).locked_until.expect("locked");
    assert!(until > now + day - 60_000_000_000 && until <= now + day);
    assert!(r.try_unlock(&pic, &r.good_p, principal_of(82)).is_err());
}

/// `tries_left` by the third limit: 30 wrong tries that nobody cleared.
#[test]
fn tries_left_at_thirty_uncleared() {
    let pic = PocketIc::new();
    let r = ready(&pic);
    let month = std::time::Duration::from_secs(31 * 24 * 3600);
    let mut left = Vec::new();
    for n in 1..=30u8 {
        // One typo a month, each followed by a good unlock. Only the third limit comes closer.
        pic.advance_time(month);
        left.push(r.wrong_try(&pic));
        if n < 30 {
            r.open_session(&pic, principal_of(100 + n));
        }
    }
    assert!(left[..26].iter().all(|x| *x == 4), "{left:?}");
    assert_eq!(left[26..], [3, 2, 1, 0]);
    let info = r.info(&pic);
    assert!(info.needs_clearing);
    pic.advance_time(month);
    assert!(
        r.try_unlock(&pic, &r.good_p, principal_of(200)).is_err(),
        "the key stays locked until a full session clears it"
    );
}
