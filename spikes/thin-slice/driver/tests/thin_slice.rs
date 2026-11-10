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
        label: bb(&label_ct(vid, vk, &id, 25, "Recovery kit")),
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

    // ----- ADR 0012 -----
    let good_p = pin_key(&info, pin);
    let wrong_p = pin_key(&info, "482914");
    let second = std::time::Duration::from_secs(1);
    let alive = |who: Principal| {
        pic.update_call(can, who, "sync", encode_one(0u64).unwrap())
            .is_ok()
    };
    let open_session = |who: Principal| {
        pic.advance_time(second);
        let info = device_info(&pic, can, owner, &dev.slot.id);
        let pr = prove_with(&vid, can, owner, &info, &dev.f, &good_p, who);
        unlock(&pic, can, owner, &info, who, &pr)
            .unwrap()
            .expect("unlock")
    };
    let wrong_try = || -> u8 {
        pic.advance_time(second);
        let info = device_info(&pic, can, owner, &dev.slot.id);
        let who = principal_of(99);
        let pr = prove_with(&vid, can, owner, &info, &dev.f, &wrong_p, who);
        match unlock(&pic, can, owner, &info, who, &pr).expect("a wrong PIN reaches the method") {
            Err(Error::BadProof { tries_left, .. }) => tries_left,
            other => panic!("expected BadProof, got {:?}", other.map(|_| ()).err()),
        }
    };

    // A sixth full session is not refused. It ends the one whose time runs out first.
    let s: Vec<Principal> = (70..76).map(principal_of).collect();
    for who in &s[..5] {
        open_session(*who);
    }
    assert!(
        s[..5].iter().all(|who| alive(*who)),
        "five sessions are live"
    );
    // The first one renews, so the second is now the one that runs out first.
    pic.advance_time(second);
    pic.update_call(can, s[0], "renew", encode_args(()).unwrap())
        .unwrap();
    open_session(s[5]);
    assert!(!alive(s[1]), "the session that would end first is gone");
    for who in [s[0], s[2], s[3], s[4], s[5]] {
        assert!(alive(who), "the other sessions keep working");
    }

    // `tries_left` follows whichever limit is closest. The vault has one wrong try so far.
    let left: Vec<u8> = (0..4).map(|_| wrong_try()).collect();
    assert_eq!(left, [4, 3, 2, 1], "five in a row is the closest limit");
    open_session(principal_of(80)); // a good unlock ends the row, not the count of the vault
    let left: Vec<u8> = (0..4).map(|_| wrong_try()).collect();
    assert_eq!(left, [4, 3, 2, 1]);
    open_session(principal_of(81));
    // Nine wrong tries in the vault. The tenth is the last one before the 24 hours.
    assert_eq!(wrong_try(), 1, "the limit of the vault is now the closest");
    assert_eq!(wrong_try(), 0, "the eleventh wrong try locks the key");
    let info = device_info(&pic, can, owner, &dev.slot.id);
    let now = pic.get_time().as_nanos_since_unix_epoch();
    let until = info.locked_until.expect("locked");
    let day = 24 * 3600 * 1_000_000_000u64;
    assert!(until > now + day - 60_000_000_000 && until <= now + day);
    let pr = prove_with(&vid, can, owner, &info, &dev.f, &good_p, principal_of(82));
    assert!(
        unlock(&pic, can, owner, &info, principal_of(82), &pr).is_err(),
        "a locked key is refused in the hook, even with the right PIN"
    );
}

/// ADR 0012: the canister refuses a slot whose label is not 92 bytes.
#[test]
fn a_label_of_another_length_is_refused() {
    let pic = PocketIc::new();
    let claim_code = bytes::<32>(1);
    let can = install(
        &pic,
        InstallArgs::Init {
            claim_hash: bb(&sha256(&claim_code)),
            host_suffix: "icp.net".into(),
            environment: "production".into(),
            local_root_key: None,
        },
    );
    let owner = principal_of(2);
    let r: VResult = decode_one(
        &pic.update_call(can, owner, "claim", encode_one(bb(&claim_code)).unwrap())
            .unwrap(),
    )
    .unwrap();
    assert!(matches!(r, VResult::Ok));
    let vid = can.as_slice().to_vec();
    let vk = bytes::<32>(30);
    let mut dev = build_device_slot(&vid, &vk, "271828");
    dev.slot.label = bb(&[0u8; 44]);
    let (kit_slot, kit_principal, export_header) = build_kit_slot(&vid, &vk, &bytes::<32>(31));
    let vault_auth = SigningKey::from_bytes(&hkdf32(&vk, &vid, b"ic-vault/v1/vault-auth"));
    let setup = SetupArgs {
        device_slot: dev.slot,
        kit_slot,
        kit_principal,
        export_header: bb(&export_header),
        vault_auth_pub: bb(vault_auth.verifying_key().as_bytes()),
        session: principal_of(40),
    };
    let r: VResult = decode_one(
        &pic.update_call(can, owner, "setup_finish", encode_one(setup).unwrap())
            .unwrap(),
    )
    .unwrap();
    assert!(matches!(r, VResult::Err(Error::Invalid(_))));
}
