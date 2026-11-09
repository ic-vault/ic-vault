# ic-vault

> **Experimental. Not independently reviewed. Keep here only what you can replace.**

An open-source personal vault on the Internet Computer. Every user runs one canister of their own. It stores ciphertext only and serves the web app. All encryption happens in the browser.

## Status

Nothing is built yet. The repository holds the design and the plan.

- [VAULT-PLAN.md](VAULT-PLAN.md) is the design, with its security claim and its risks.
- [IMPLEMENTATION-PLAN.md](IMPLEMENTATION-PLAN.md) lists the tasks in order, with their tests.
- [DESIGN-REVIEW-FINDINGS.md](DESIGN-REVIEW-FINDINGS.md) is a review of the design by its own author.
- [DESIGN-REVIEW-BRIEF.md](DESIGN-REVIEW-BRIEF.md) is the brief for anyone who wants to review it.

## Not independently reviewed

No specialist and no auditor has read the design or the code, and there is no budget for an audit. The design and most of the code are written with AI agents. Section 1 of VAULT-PLAN.md says what the vault claims and what it does not.

Until a named, independent person has reviewed the design and published the result:

- Store nothing you cannot rotate, such as identity documents or seed phrases.
- Keep your current password manager as the master copy.

Reviews are welcome. Start with the brief.

## Reporting a vulnerability

Report it privately through GitHub. See [SECURITY.md](SECURITY.md).

## Layout

Section 10 of VAULT-PLAN.md gives the layout. Each directory appears with the task that fills it.

## Toolchain

| Tool | Version | Pinned in |
|---|---|---|
| Node.js | 26.13.0 | `.node-version`, and `engines` in `package.json` |
| pnpm | 12.17.0 | `packageManager` in `package.json` |
| Rust | 1.99.1 | `rust-toolchain.toml` |
| icp-cli | 1.8.0 | `devDependencies` in `package.json` |

Install without running any package script:

```bash
pnpm install --frozen-lockfile --ignore-scripts
pnpm exec icp --version
```

On Linux, icp-cli needs the D-Bus library for the system keyring. If `icp --version` reports that `libdbus-1.so.3` is missing, install it:

```bash
sudo apt-get install libdbus-1-3
```

A new package version enters the lockfile only after it has been published for seven days. The rule is in `pnpm-workspace.yaml`.

## License

Apache-2.0. See [LICENSE](LICENSE).
