# ic-vault

> **Experimental. Nothing is released yet. Do not store real secrets.**

An open-source personal vault on the Internet Computer. Every user runs one canister of their own. It stores ciphertext only and serves the web app. All encryption happens in the browser.

## Status

Nothing is built yet. The repository holds the design and the plan.

- [VAULT-PLAN.md](VAULT-PLAN.md) is the design, with its security claim and its risks.
- [docs/](docs/) holds the parts the work follows: the [threat model](docs/threat-model.md), the [crypto spec](docs/crypto-spec.md), [operations](docs/operations.md), and the [implementation plan](docs/implementation-plan.md), which lists the tasks in order and tracks them. Since task M0-02 these copies win over VAULT-PLAN.md.
- [docs/adr/](docs/adr/) holds the decision records.
- [IMPLEMENTATION-PLAN.md](IMPLEMENTATION-PLAN.md) is the plan as first written. It is frozen.
- [DESIGN-REVIEW-FINDINGS.md](DESIGN-REVIEW-FINDINGS.md) is a review of the design by its own author.
- [docs/review/](docs/review/) is the review package for anyone who wants to review the design: the brief, and a map from each security claim to its mechanism. [DESIGN-REVIEW-BRIEF.md](DESIGN-REVIEW-BRIEF.md) is the first version of the brief, frozen.
- [AGENTS.md](AGENTS.md) holds the rules for the AI agents that build it.

## Not independently reviewed

No specialist and no auditor has read the design or the code, and there is no budget for an audit. The design and most of the code are written with AI agents. Section 1 of VAULT-PLAN.md says what the vault claims and what it does not.

Trust therefore grows in stages. Nothing has been released, so no stage is open yet.

| Stage | What may go in | Opens when |
|---|---|---|
| 1 | Test data only | A staging vault exists |
| 2 | Secrets you can rotate, such as passwords. Your current password manager stays the master copy | The release gate of VAULT-PLAN.md section 12 has passed |
| 3 | What you cannot rotate, such as identity documents and seed phrases | A named, independent person has reviewed the design and published the result, and a client exists that does not arrive through the gateway |

Until stage 3 opens, every release says "Not independently reviewed. Keep here only what you can replace".

Reviews are welcome. Start with the brief in [docs/review/](docs/review/).

## Reporting a vulnerability

Report it privately through GitHub. See [SECURITY.md](SECURITY.md).

## Development setup

Work on Linux or macOS. On Windows, work inside WSL Ubuntu. A native Windows install fails the Node version check, and the local network expects Linux.

1. Install Node.js 26.13.0, the version in `.node-version`. A version manager that reads that file works.
2. Install pnpm without running its scripts. pnpm then switches itself to the version in `package.json`, which the lockfile records with its hashes.

   ```bash
   npm install -g pnpm@12.17.0 --ignore-scripts
   ```

3. On Linux, install the system libraries that icp-cli needs.

   ```bash
   sudo apt-get install -y libdbus-1-3 libssl3 ca-certificates
   ```

4. Install the dependencies without running any package script.

   ```bash
   pnpm install --frozen-lockfile --ignore-scripts
   pnpm exec icp --version
   ```

icp-cli ships binaries for Linux and macOS on x64 and arm64, and for Windows on x64. Alpine and other systems without glibc are not expected to work.

A new package version enters the lockfile only after it has been published for seven days. The rule is in `pnpm-workspace.yaml`.

## Toolchain

| Tool | Version | Pinned in |
|---|---|---|
| Node.js | 26.13.0 | `.node-version`. The `engines` field accepts later 26.x releases too |
| pnpm | 12.17.0 | `packageManager` in `package.json`, and the lockfile |
| Rust | 1.99.1 | `rust-toolchain.toml` |
| icp-cli | 1.8.0 | `devDependencies` in `package.json`, and the lockfile |
| Official ICP skills | `dfinity/icskills` at commit `1c28dd2` | `.claude/skills/` and `.agents/skills/` |

[ADR 0001](docs/adr/0001-toolchain.md) records the release dates, the checks and how to update.

## Layout

Section 10 of VAULT-PLAN.md gives the layout. Each directory appears with the task that fills it. `icp.yaml`, `Cargo.toml` and `deny.toml` arrive with the first canister, task M0-05.

## License

Apache-2.0. See [LICENSE](LICENSE).
