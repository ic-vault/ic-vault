# ADR 0001: Toolchain and pinned skills

Status: Accepted
Date: 2026-11-09
Task: M0-01 for the tools, M0-02 for this record, the skills and the checks of spec 8.4
Decided by: the pull requests of tasks M0-01 and M0-02, under the rule of spec 8.4

## Question

Which versions of the toolchain, the libraries and the official ICP skills does the project start with, and how do they move later? Spec 8.4 and Appendix B ask M0 to pin Rust, Node, pnpm and icp-cli, to check every version that spec 8.4 names, and to pin the official ICP skills of spec 11.1.

## Decision

### The 7-day rule

Everything that is pinned is the newest release that was at least 7 days old on the day it was pinned. That covers tools, npm packages, crates, container images and skill commits. Exact versions only.

- pnpm enforces the rule for npm packages through `minimumReleaseAge` in `pnpm-workspace.yaml`.
- For everything else the agent applies it by hand and records the release date.
- A security fix that cannot wait 7 days goes through `minimumReleaseAgeExclude` and an ADR of its own, as spec 8.4 says.

### Tools, pinned in M0-01

| Tool | Version | Released | Pinned in | Newer, refused under the rule |
|---|---|---|---|---|
| Node.js | 26.13.0, Active LTS "Lithium" | 2026-10-28 | `.node-version`, `engines` in `package.json` | none |
| pnpm | 12.17.0 | 2026-10-30 | `packageManager` in `package.json`, and the lockfile | 12.18.0 of 2026-11-05 |
| Rust | 1.99.1, with the `wasm32-unknown-unknown` target, rustfmt and clippy | 2026-10-15 | `rust-toolchain.toml` | none |
| icp-cli | 1.8.0, npm package `@icp-sdk/icp-cli` | 2026-10-30 | `devDependencies` in `package.json`, and the lockfile | none |

On Linux, icp-cli needs the system packages `libdbus-1-3`, `libssl3` and `ca-certificates`. The README lists them.

### Libraries named in spec 8.4, checked on 2026-11-09

None of them is in the repository yet. Each enters with the first task that needs it, at the newest version that is 7 days old on that day. The table shows what that would be today.

| Crate | Spec says | Newest | Newest 7 days old | Result |
|---|---|---|---|---|
| `ic-cdk` | 0.20 | 0.20.3, 2026-09-15 | 0.20.3 | Current |
| `ic-stable-structures` | no version | 0.7.2, 2025-09-22 | 0.7.2 | Current |
| `ic-asset-certification` | 4 | 4.0.1, 2026-10-28 | 4.0.1 | Current |
| `ic-http-certification` | 4 | 4.0.1, 2026-10-28 | 4.0.1 | Current |
| `candid` | 0.10 | 0.10.40, 2026-11-05 | 0.10.39, 2026-10-21 | Current |
| `ed25519-dalek` | no version | 3.0.0, 2026-07-06 | 3.0.0 | A new major version. The task that adds it checks the verify API |
| `sha2` | no version | 0.11.1, 2026-10-12 | 0.11.1 | Current |
| `serde` | no version | 1.0.229, 2026-07-18 | 1.0.229 | Current |

| npm package | Newest | Newest 7 days old |
|---|---|---|
| `react`, `react-dom` | 19.3.1, 2026-10-14 | 19.3.1 |
| `@icp-sdk/core` | 6.1.1, 2026-10-12 | 6.1.1 |
| `@icp-sdk/auth` | 11.2.0, 2026-11-09 | 11.1.0, 2026-10-27 |
| `@icp-sdk/bindgen`, from spec 11.1 | 0.4.1, 2026-09-23 | 0.4.1 |
| `@noble/hashes` | 2.4.0, 2026-08-27 | 2.4.0 |
| `@noble/curves` | 2.4.0, 2026-08-27 | 2.4.0 |

Spec 8.4 also allows "one small QR generator" without naming one. The task that needs it names it in an ADR.

**Still open: the `getrandom` check of spec 8.4.** Among the official skills, only the vetKeys skill describes the Wasm workaround: the crate `ic-dummy-getrandom-for-wasm` or a `getrandom` shim, otherwise the canister traps. That skill is not pinned, because the network key is deferred. No crate is in the repository yet, so nothing can be checked. Task M0-05 runs `cargo tree -i getrandom` on the first Cargo workspace and records the result.

### Agent skills, pinned in M0-02

| Field | Value |
|---|---|
| Source | https://github.com/dfinity/icskills, Apache-2.0 |
| Commit | `1c28dd2035fd9890ac0953661eb78ae7ee603e4d`, 2026-11-02, signature verified by GitHub |
| Skills | icp-cli, canister-security, internet-identity, stable-memory, certified-variables, cycles-management |
| Installed with | `skills` 1.7.5, the official CLI from npm. 1.7.6 of 2026-11-03 was refused under the rule |
| Claude Code reads | `.claude/skills/` |
| Cursor and Codex read | `.agents/skills/` |

- The six skills are the ones spec 11.1 names. Not pinned: vetkeys, deferred with the network key. Also custom-domains, because the vault uses a gateway domain, see ADR 0002. And the skills for Motoko, tokens and other chains, which the vault does not use.
- Two newer commits change a pinned skill. On 2026-11-06 the internet-identity skill was split into five skills, and on 2026-11-09 its description was shortened. The monthly upkeep task takes them once they are 7 days old.
- The two folders hold copies, not symlinks. Symlinks break on a Windows checkout. Both folders are identical.
- The installed files equal GitHub's archive of the commit, byte for byte.
- The CLI's lock file `skills-lock.json` is not committed. It records the temporary path the skills were installed from, not the commit. The hashes it computed are listed here instead:

| Skill | Hash computed by the CLI |
|---|---|
| canister-security | `8d00ec4ffd76f1d9347032e85899e829acd172d66d9c7d23c89d21b0451d2bf2` |
| certified-variables | `bf18b5475b7a98f30bb7a9d8c783450d8fc62f34dfb1ef145e8236d285e85595` |
| cycles-management | `126166ed1b9ec058f109b98b67d3b4759e8f1a544b55566d1c47b8a8c30a2351` |
| icp-cli | `22935bd2721cf3feafa202648038e0d582b7b6fc1997b05add4a7fedb9ac7c71` |
| internet-identity | `5df7465da671cd0311862dbd096bdc1dc2d8e83ad5541be052a5b0d6c87c7a4e` |
| stable-memory | `b4233626420d3d473cab2cf96bbe70204ded9387c80077487b9fe6a7e413c2d3` |

Two instructions in the pinned skills conflict with `AGENTS.md`, and `AGENTS.md` wins:

- The icp-cli skill installs icp-cli globally with npm. The project uses its pinned copy through `pnpm exec icp`.
- The icp-cli skill shows `icp deploy --mode reinstall`. No script may contain a reinstall mode, hard rule 12.

## How to update the skills

The monthly upkeep task of spec Appendix E does this.

1. Pick the newest commit of `dfinity/icskills` that is at least 7 days old, and the newest `skills` release that is at least 7 days old.
2. Read the diff of the pinned skills since the last commit. Skills run with full agent permissions, so read them like code. Look for instructions that conflict with `AGENTS.md`, install tools, or send data anywhere.
3. In an empty folder, run the command below with the new commit, the new CLI version, and a `--before` date 7 days before the run. The CLI's telemetry stays off.
4. Replace `.claude/skills/` and `.agents/skills/` with the folders it created. Delete its `skills-lock.json`.
5. Record the new commit, the CLI version and the hashes in a new ADR that supersedes this section.

```bash
docker run --rm -e DISABLE_TELEMETRY=1 -e DO_NOT_TRACK=1 -v "$PWD:/w" -w /w node:26.13.0-slim sh -c '
  apt-get update && apt-get install -y --no-install-recommends git ca-certificates &&
  git clone https://github.com/dfinity/icskills.git /src &&
  git -C /src checkout 1c28dd2035fd9890ac0953661eb78ae7ee603e4d &&
  npx --yes --before=2026-11-02T21:00:00Z skills@1.7.5 add /src/skills \
    --skill icp-cli --skill canister-security --skill internet-identity \
    --skill stable-memory --skill certified-variables --skill cycles-management \
    -a claude-code -a cursor --copy -y'
```

The CLI clones by branch name, so it cannot take a commit id directly. The command therefore clones the repository, checks out the commit, and installs from that local copy. `npx --before` applies the 7-day rule to the CLI's own dependencies.

## Consequences

- The pins lag behind the newest releases by design. The monthly upkeep task moves them.
- A tool or library that is not listed here needs an ADR before it enters, hard rules 5 and 17.
- Task M0-05 closes the `getrandom` check.
