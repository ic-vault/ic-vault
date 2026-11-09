# ADR 0002: Canonical gateway domain

Status: Proposed. Waiting for the owner's signature.
Date: 2026-11-09
Task: M0-02, lane H. Decision D8 of `VAULT-PLAN.md`
Decided by: the owner, once signed below

## Question

At which address does every vault live: `https://<canister-id>.icp.net` or `https://<canister-id>.icp0.io`?

## Why it matters

- The vault passkey uses the full host as its relying party id, spec 8.3 and correction C35. A passkey made at one address does not work at the other. Moving an existing vault to another address means enrolling every device again, decision D8.
- The kit sheet prints the address, owner mistake U18, and the owner keeps using it for years.
- The host suffix is an install argument. It goes into the certified `config.json` as the accepted host, spec 7.6.
- Nothing exists yet. Until the staging vault of milestone M4 gets its first passkey, a change costs nothing.

## Facts, checked on 2026-11-09

1. **The docs.** The custom domains guide says that by default every canister is reachable at `https://<canister-id>.icp.net`. The pinned skills use `icp.net` in their examples. No notice that `icp0.io` will be retired was found.
2. **What answers today.** An old canister answered on `icp.net`, `icp0.io`, `ic0.app` and `icp1.io`. Two newer canisters answered on `icp.net` and `icp0.io`, and got status 400 on `ic0.app`. `ic0.app` was the default before `icp0.io`. So an old default domain has been closed to new canisters once, and the canisters that already used it kept it.
3. **The public suffix list.** It lists `icp0.io`, so `<canister-id>.icp0.io` is an ordinary site, the usual case for a passkey. It lists `*.icp.net`, so every `<canister-id>.icp.net` is itself a public suffix. The web rules allow a passkey whose relying party id equals the host even then, but this case is rare and nobody has tested it for this project. Spike SP1 already names the risk.
4. **Gate 1 is not affected.** Internet Identity treats `icp.net`, `icp0.io` and `ic0.app` as one origin, so the sign-in principal is the same at either address. Source: the pinned internet-identity skill.
5. **API calls.** On `icp0.io` the SDK sends API calls through the page's own origin. On `icp.net` and custom domains it sends them to `https://icp-api.io`, which the page policy of spec 7.6 already allows. Source: the pinned icp-cli skill.

Sources: the [custom domains guide](https://docs.internetcomputer.org/guides/frontends/custom-domains.md), the [public suffix list](https://publicsuffix.org/list/public_suffix_list.dat), and the skills in `.claude/skills/`, pinned by ADR 0001.

## Options

| | `icp.net` | `icp0.io` |
|---|---|---|
| Docs | The default address today | Still served. Not named in the guide |
| Passkey | The host is a public suffix. Untested | An ordinary site. The usual case |
| API calls | Go to `https://icp-api.io` | Go to the page's own origin |
| Lifetime | The newest default | The previous default. If it closes to new canisters, an existing vault keeps it, judging by `ic0.app` |

A custom domain is not offered. It would add a registrar, a DNS provider and a certificate as parties that can serve a different page, it costs money every year, and a lapsed domain locks the owner out. The spec leaves it out.

## Recommendation

`icp.net`, the spec default, on one condition: spike SP1 passes at `<canister-id>.icp.net` on every device you use.

- It is the address the docs give every canister today, and the newest default has the longest future for new canisters.
- Its one open risk is the passkey on a public-suffix host. SP1 finds out before any passkey that counts exists.
- SP1 therefore runs every passkey test at both addresses. If `icp.net` fails where `icp0.io` passes, a new ADR switches to `icp0.io` before the staging vault is set up.

## Decision

To be filled in by the owner. Write the chosen domain, your name and the date. Or tell the agent, who records your answer here with your words.

- Chosen domain:
- Signed:

## Consequences, once signed

- `deploy.sh` passes the chosen suffix as the install argument, and `config.json` accepts only that host on mainnet.
- The kit sheet prints `https://<canister-id>.<chosen domain>`.
- The SP1 row in `docs/implementation-plan.md` now asks for every passkey test at both addresses. This ADR made that change.
- Spec D8 and section 7.2 of `docs/implementation-plan.md` record the decision.
- If the choice is `icp0.io`, task M0-05 checks whether the page policy can drop `https://icp-api.io` from `connect-src`.
