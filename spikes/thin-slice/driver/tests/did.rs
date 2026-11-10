use std::path::Path;
#[test]
fn vault_did_parses() {
    let p = Path::new(env!("CARGO_MANIFEST_DIR")).join("../vault.did");
    let (_env, actor, _) =
        candid_parser::typing::pretty_check_file(&p).expect("vault.did does not type-check");
    assert!(actor.is_some(), "no service in vault.did");
}
