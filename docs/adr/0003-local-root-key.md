# ADR 0003: How the app learns the root key of a local network

Status: Accepted. Signed by the owner on 2026-11-10
Date: 2026-11-10
Task: the review of plan and architecture of 2026-11-10, finding D2. Blocks M0-05
Decided by: the owner

## Question

A local test network has its own root key. How does the web app learn it, when the tests must load the page the canister serves, from the same Wasm that ships?

## Facts, checked on 2026-11-10

1. Rule 8 of `AGENTS.md` forbids `fetchRootKey`. The pinned canister-security skill says the same, even behind an environment check.
2. Spec 8.2 says: "Production bundles contain no root key override and no call to `fetchRootKey`."
3. Section 5.1 of the plan says the end-to-end tests "always load the page that the canister serves" and "never use the development server". Spec 7.6 says the release Wasm is identical for every vault.
4. The pinned icp-cli skill says an agent without `rootKey` uses the mainnet key, "and every call against a local network fails verification". The usual source of the key is the `ic_env` cookie. That cookie is set by the standard frontend canister or by the Vite development server. The vault uses neither.
5. `config.json` of section 3.9 has a fixed `"api_host": "https://icp-api.io"`. On a local network the API is at the page's own origin.
6. The same root key is needed for the local Internet Identity, pinned internet-identity skill, mistake 18.

So M0-05, M0-06, SP6 and SP8 cannot be built as written. Something has to give.

## Options

| | What it means | Good | Bad |
|---|---|---|---|
| A | The canister puts the local root key and the API host into its certified `config.json`, from the install arguments. The app uses them only on a loopback host with the environment `local` | One Wasm for tests and release. The rule is two short checks that a test can prove | The bundle contains a root key override, guarded. Spec 8.2 has to say so |
| B | The canister sets the `ic_env` cookie itself, and the app reads it as the skills do | Matches the SDK examples | The standard pattern also takes the key from the cookie on mainnet. A cookie is not part of the certified page. The app would trust it everywhere unless guarded like A |
| C | A separate test build with the key baked in | The release bundle stays free of any override | The tests no longer run the Wasm that ships. That gives up fact 3 |
| D | `fetchRootKey` behind a check | Least work | Forbidden by rule 8 and by the skill |

## Decision

Chosen: option A.

- `InstallArgs.Init` gets one more field, `local_root_key : opt blob`.
- **The canister** refuses to install when `local_root_key` is set and the environment is not `local`, when the environment is `local` and it is missing, and when the environment is `local` and `host_suffix` is not `localhost` with an optional port.
- **`config.json`** carries `api_host` and `root_key`. On `production` and `staging`, `api_host` is `https://icp-api.io` and `root_key` is `null`. On `local`, `api_host` is `null`, which means the page's own origin, and `root_key` is the key in hex.
- **The app** uses `root_key` only when all three hold: the environment in the config is `local`, the host name is `localhost` or ends in `.localhost`, and the key is present. If a key is present and one of the other two fails, the app refuses to run. In every other case it passes no root key, and the SDK uses its built-in mainnet key.
- **The local Internet Identity** follows the same rule. Only when the three conditions hold, the app signs in at `http://id.ai.<host suffix>/authorize` and hands the client the same root key. The pinned skill says a local Internet Identity has the canister ids of mainnet. Everywhere else the app uses the default, mainnet Internet Identity.
- The banner of owner mistake U13 already marks every environment that is not `production`.

Why this is safe enough: a browser resolves `localhost` names to the machine itself, so no mainnet page can pass the host check. Someone who can change the page that is delivered can do worse than change a key, see R1.

Signed: Manuel Rüedi, the owner, on 2026-11-10, in the chat with the agent that wrote this record. The answer, in the owner's words: "Drop 0011 and 0012 for now. Rest is ok". The agent recorded it.

## Consequences

- `docs/implementation-plan.md`: `InstallArgs` in 3.2, the `Config` line in 3.3, section 3.9, step 6 of the gate in 5.1, and the proof of M0-05.
- `VAULT-PLAN.md`: one bullet in 7.6 and one in 8.2.
- Tests. PocketIC: the three refusals at install. Web app: a key on a host that is not loopback stops the app, and no key means the built-in key. Bundle: still no `fetchRootKey`.
