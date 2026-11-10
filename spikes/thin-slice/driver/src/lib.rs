//! Client side of spike SP9: the derivations of spec 6.3 that the thin path needs, in plain Rust.
//! Throwaway. The product does this in the browser worker.

use aes_gcm::aead::{Aead, KeyInit, Payload};
use aes_gcm::{Aes256Gcm, Nonce};
use ed25519_dalek::{Signer, SigningKey};
use hkdf::Hkdf;
use sha2::{Digest, Sha256};

pub fn lp(x: &[u8]) -> Vec<u8> {
    let mut v = (x.len() as u32).to_be_bytes().to_vec();
    v.extend_from_slice(x);
    v
}

pub fn cat(parts: &[&[u8]]) -> Vec<u8> {
    parts.concat()
}

pub fn hkdf32(ikm: &[u8], salt: &[u8], info: &[u8]) -> [u8; 32] {
    let mut out = [0u8; 32];
    Hkdf::<Sha256>::new(Some(salt), ikm)
        .expand(info, &mut out)
        .expect("hkdf");
    out
}

/// Blob layout of spec 6.1: nonce (12) || ciphertext || tag (16).
pub fn seal(key: &[u8; 32], nonce: &[u8; 12], plain: &[u8], aad: &[u8]) -> Vec<u8> {
    let ct = Aes256Gcm::new_from_slice(key)
        .unwrap()
        .encrypt(Nonce::from_slice(nonce), Payload { msg: plain, aad })
        .expect("seal");
    cat(&[nonce, &ct])
}

pub fn open(key: &[u8; 32], blob: &[u8], aad: &[u8]) -> Option<Vec<u8>> {
    let (nonce, ct) = blob.split_at(12);
    Aes256Gcm::new_from_slice(key)
        .unwrap()
        .decrypt(Nonce::from_slice(nonce), Payload { msg: ct, aad })
        .ok()
}

pub fn argon2id(pin: &[u8], salt: &[u8], memory_kib: u32, passes: u32, lanes: u32) -> [u8; 32] {
    let params = argon2::Params::new(memory_kib, passes, lanes, Some(32)).expect("params");
    let mut out = [0u8; 32];
    argon2::Argon2::new(argon2::Algorithm::Argon2id, argon2::Version::V0x13, params)
        .hash_password_into(pin, salt, &mut out)
        .expect("argon2");
    out
}

pub fn pad(x: &[u8], bucket: usize) -> Vec<u8> {
    let mut v = lp(x);
    assert!(v.len() <= bucket);
    v.resize(bucket, 0);
    v
}

pub fn unpad(x: &[u8]) -> Vec<u8> {
    let n = u32::from_be_bytes(x[..4].try_into().unwrap()) as usize;
    x[4..4 + n].to_vec()
}

pub struct SlotKeys {
    pub kek: [u8; 32],
    pub auth: SigningKey,
    pub auth_f: SigningKey,
}

/// Slot keys of spec 6.3. `f` and `p` are 32 bytes each, ZERO32 where the kind has none.
pub fn slot_keys(
    f: &[u8; 32],
    p: &[u8; 32],
    slot_salt: &[u8],
    vid: &[u8],
    slot_id: &[u8],
    kind: u8,
) -> SlotKeys {
    let ikm = cat(&[f, p]);
    let info = |label: &str| cat(&[&lp(label.as_bytes()), &lp(vid), slot_id, &[kind]]);
    SlotKeys {
        kek: hkdf32(&ikm, slot_salt, &info("ic-vault/v1/slot-kek")),
        auth: SigningKey::from_bytes(&hkdf32(&ikm, slot_salt, &info("ic-vault/v1/slot-auth"))),
        auth_f: SigningKey::from_bytes(&hkdf32(f, slot_salt, &info("ic-vault/v1/slot-auth-f"))),
    }
}

pub fn slot_inner_aad(vid: &[u8], slot_id: &[u8], epoch: u32) -> Vec<u8> {
    cat(&[
        &lp(b"ic-vault/v1/slot-inner"),
        &lp(vid),
        slot_id,
        &epoch.to_be_bytes(),
    ])
}

pub fn unlock_msg(
    vid: &[u8],
    canister: &[u8],
    caller: &[u8],
    slot_id: &[u8],
    nonce: u64,
    session: &[u8],
) -> Vec<u8> {
    cat(&[
        &lp(b"ic-vault/v1/unlock"),
        &lp(vid),
        &lp(canister),
        &lp(caller),
        slot_id,
        &nonce.to_be_bytes(),
        &lp(session),
    ])
}

pub fn sign(key: &SigningKey, msg: &[u8]) -> Vec<u8> {
    key.sign(msg).to_bytes().to_vec()
}

pub fn sha256(x: &[u8]) -> Vec<u8> {
    Sha256::digest(x).to_vec()
}

pub fn item_aad(
    label: &str,
    vid: &[u8],
    item_id: &[u8],
    version: u64,
    part: Option<u8>,
) -> Vec<u8> {
    let mut v = cat(&[
        &lp(label.as_bytes()),
        &lp(vid),
        item_id,
        &version.to_be_bytes(),
    ]);
    if let Some(p) = part {
        v.push(p);
    }
    v
}
