# ADR 0005: Worker messages for recovery with the kit

Status: Accepted, by the merge of the pull request that carries it
Date: 2026-11-10
Task: the review of plan and architecture of 2026-11-10, finding D3
Decided by: the pull request that merges it. It changes section 3.5 of the plan before task M1-01 freezes it

## Question

Recovery needs the kit secret in two ways. How do they reach the worker, when the message table of section 3.5 "is the whole surface of `crypto/`"?

## Facts, checked on 2026-11-10

1. Rule 1 of `AGENTS.md`: vault keys exist only in `apps/web/src/crypto`.
2. Spec 6.6, "Recover": with Internet Identity you "enter the 24 words" and the kit slot opens. Without it, "the client signs in as the kit principal `R`".
3. `R` is the principal of `kit_id`, an Ed25519 key derived from the kit secret, spec 6.3. To call as `R`, something must sign every request with that key.
4. Section 3.5 has `kit.confirm`, which checks typed words during setup and kit replacement. It has no message that takes the words for recovery, and none that signs as `R`. `unlock.prove` takes `F` from outside, but for the kit `F` is derived from the kit secret.
5. Section 3.5 and spec 8.1 ask for "a fresh passkey answer that opens the current slot" before three requests. A passkey answer is `F` alone. A slot with a PIN does not open without `P`. The text does not say what the worker compares.

## Decision

Two new messages, one changed message, and one definition.

| Message | In | Out |
|---|---|---|
| `kit.open` | The 24 words as typed | The kit principal `R` and its public key, or the position of the first wrong word. The worker keeps the kit secret until `lock` |
| `kit.sign` | The bytes of one request, as the SDK hands them to an identity | An Ed25519 signature made with `kit_id`. Only after `kit.open`. Only for bytes that start with the request domain separator of the IC |

- `unlock.prove` takes no `F` for the kit key. The worker uses the kit factor it holds since `kit.open`.
- `platform/` wraps `kit.sign` as an identity for the SDK. The key never leaves the worker.
- **A fresh passkey answer** means: at unlock the worker keeps the `auth_f_pub` it derived. Later it derives `auth_f_pub` from the new `F` and accepts the answer when the two are equal. `F` is overwritten at once in both cases.
- Spike SP6 gets one more question: can an identity whose signature is made in the worker drive an agent, and what exactly are the bytes it is asked to sign? If the answer does not fit `kit.sign` as written, a new ADR changes it before the freeze.

`kit.sign` lets a script in the page sign calls as `R` while the worker holds the kit. That is no more than the page can already do, spec 8.1: "It does not stop a script in the page from asking it to use the keys." `R` can try the kit key and nothing else, and it waits.

## Consequences

- `docs/implementation-plan.md`: section 3.5, the SP6 row, and the proofs of M2-06 and M5-03.
- `VAULT-PLAN.md` 8.1: the bullet about the three requests says what a fresh passkey answer is.
- Tests. Every new message has a worker test, as M2-06 requires. `kit.sign` refuses before `kit.open` and refuses bytes without the separator. A lint test shows that `platform/` holds no key of `R`.
