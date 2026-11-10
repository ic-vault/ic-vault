# Spike SP9: one thin path, end to end

Throwaway code. It answers one question before task M1-01 freezes the interface: does section 3 of `docs/implementation-plan.md` compile, and does one path run? The answer and the gaps it found are in `docs/adr/0011-thin-slice-spike.md`. The four gaps are decided in `docs/adr/0012-four-gaps-before-freeze.md`, and the code here follows that record: the label of 92 bytes, `tries_left` by the closest of the three limits, a sixth session that ends the one running out first, and the claim code checked in the message hook.

Nothing here is the product. State lives on the heap, 17 of the 29 methods are stubs, and nobody reviewed it. Do not copy from it without reading the spec again.

## What is here

| Path | What |
|---|---|
| `vault.did` | The interface of section 3.2, copied unchanged, plus the two HTTP types |
| `canister/` | A Rust canister with the types of 3.2. Real: `claim`, `setup_finish`, `slots_public`, `unlock` with its message hook, `renew`, `lock`, `sync`, `item_put`, `item_body`, `slot_clear_lock`, `status`, `trust` |
| `driver/` | The client side in plain Rust: the derivations of spec 6.3 that the path needs, and the test that walks the path on PocketIC |

## Run it

```bash
cd spikes/thin-slice
cargo build -p thin-slice-canister --target wasm32-unknown-unknown --release
cargo test
```

The first test run downloads the PocketIC server 16.1.0 from GitHub into the temp directory. The product tasks should pin that binary by hash and set `POCKET_IC_BIN`.

All keys and secrets in the test are generated from fixed tags. None is real.
