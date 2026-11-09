# ADR 0001: Toolchain and pinned skills

Status: Accepted, by the merge of the pull request of task M0-02
Date: 2026-11-09
Task: M0-01 for the tools, M0-02 for this record, the skills and the checks of spec 8.4
Decided by: the pull requests of tasks M0-01 and M0-02, under the rule of spec 8.4

## Question

Which versions of the toolchain, the libraries and the official ICP skills does the project start with, and how do they move later? Spec 8.4 and Appendix B ask M0 to pin Rust, Node, pnpm and icp-cli, to check every version that spec 8.4 names, and to pin the official ICP skills of spec 11.1.

## Decision

### The 7-day rule

Everything that is pinned is the newest release that was at least 7 days old on the day it was pinned. That covers tools, npm packages, crates, container images and skill commits. Exact versions only.

- pnpm enforces the rule for npm packages through `minimumReleaseAge` in `pnpm-workspace.yaml`.
- For everything else the agent applies it by hand. The pull request names each version and its release date.
- A security fix that cannot wait 7 days goes through `minimumReleaseAgeExclude` and an ADR of its own, as spec 8.4 says.
- Which dependencies need an ADR, and which only a line in the pull request, is in the section "Dependencies" of `AGENTS.md`.

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

### Still open, for task M0-05

- **The `getrandom` check of spec 8.4.** Among the official skills, only the vetKeys skill describes the Wasm workaround: the crate `ic-dummy-getrandom-for-wasm` or a `getrandom` shim, otherwise the canister traps. That skill is not pinned, because the network key is deferred. No crate is in the repository yet. M0-05 runs `cargo tree -i getrandom` on the first Cargo workspace and records the result.
- **`ic-wasm` and the recipes.** The pinned icp-cli skill says the official recipes, `@dfinity/rust` among them, need `ic-wasm`, npm package `@icp-sdk/ic-wasm`. icp-cli refuses a recipe without a version, but the recipe is still fetched from GitHub. M0-05 decides between the Rust recipe and its own build steps, and pins `ic-wasm` and the recipe version under the 7-day rule.

### Agent skills, pinned in M0-02

| Field | Value |
|---|---|
| Source | https://github.com/dfinity/icskills, Apache-2.0 |
| Commit | `1c28dd2035fd9890ac0953661eb78ae7ee603e4d`, 2026-11-02, signature verified by GitHub |
| Skills | icp-cli, canister-security, internet-identity, stable-memory, certified-variables, cycles-management |
| Installed with | the `skills` CLI of Vercel Labs, which the icskills README uses, version 1.7.5. 1.7.6 of 2026-11-03 was refused under the rule |
| Claude Code reads | `.claude/skills/` |
| Cursor and Codex read | `.agents/skills/` |

- The six skills are the ones spec 11.1 names. Not pinned: vetkeys, deferred with the network key. Also custom-domains, because the vault uses a gateway domain, see ADR 0002. And the skills for Motoko, tokens and other chains, which the vault does not use.
- Two newer commits change a pinned skill. On 2026-11-06 the internet-identity skill was split into five skills, and on 2026-11-09 its description was shortened. The monthly upkeep task takes them once they are 7 days old.
- The two folders hold copies, not symlinks. Symlinks break on a Windows checkout.
- The CLI's lock file `skills-lock.json` is not committed. It records the temporary path the skills were installed from, not the commit.

**How to check the copies.** Each skill folder in the repository has the same git tree id as the folder in the pinned commit. Compare `git rev-parse HEAD:.agents/skills/<name>` and `git rev-parse HEAD:.claude/skills/<name>` with the tree ids below, which `gh api repos/dfinity/icskills/contents/skills?ref=<commit>` lists. All twelve matched on 2026-11-09.

| Skill | Git tree id at the pinned commit |
|---|---|
| canister-security | `d16eff57dedcfd55f1b30071d65f6b352421bcbd` |
| certified-variables | `de3e019543e33ed2b941cb0ec2c9e096c36becd0` |
| cycles-management | `1657e718eb68040ddffca80bdacc2a0cb681060c` |
| icp-cli | `4c882092056568eff25a974c7b3232911bb71084` |
| internet-identity | `ae0f3c3491c48a02942ccdd73abe6bae94949918` |
| stable-memory | `94a121f8bf7f432736cbb6527f3b67d1eab4bf2d` |

**Where the skills disagree with the project.** The project files win, as `AGENTS.md` says. Known cases:

| Skill says | The project does |
|---|---|
| Install icp-cli and `ic-wasm` globally with npm | Uses the pinned copies through `pnpm exec`. Rule 19 of `AGENTS.md` |
| `icp deploy --mode reinstall` | No script contains a reinstall mode. Rule 12 |
| `icp deploy -e ic` for mainnet | Scripts name the install mode of every mainnet install, and `icp deploy` does not appear in them. Section 6.2 of the plan. Agents never touch mainnet |
| `icp cycles buy` with a card | Agents never buy anything and never touch mainnet. Funding follows `docs/runbooks/top-up.md`, written in M4-05 |
| `@icp-sdk/core` pinned to `^6` | Exact versions only. Rule 5 |

**The container image broke the rule once.** The install ran in `node:26.13.0-slim`, pulled by its tag. Docker Hub shows that tag was last pushed on 2026-11-05, four days before. The result does not depend on the image, because the installed files equal the pinned commit byte for byte. The procedure below pins the image by digest.

## How to update the skills

The monthly upkeep task of spec Appendix E does this. The pull request needs the owner's approval, level R2 in `AGENTS.md`.

1. Pick the newest commit of `dfinity/icskills`, the newest `skills` release and the newest `node` image digest that are each at least 7 days old.
2. Read the diff of the pinned skills since the last commit. Skills run with full agent permissions, so read them like code. Look for instructions that conflict with the project, install tools, or send data anywhere. Add new conflicts to the table above.
3. In an empty folder, run the command below with the new commit, image digest, CLI version and a `--before` date 7 days before the run. The CLI's telemetry stays off.
4. Replace `.claude/skills/` and `.agents/skills/` with the folders it created. Delete its `skills-lock.json`.
5. Compare the git tree ids, as above.
6. Write a new ADR with the commit, the CLI version, the image digest and the tree ids. This ADR then gets the status "Partly superseded", for the skills.

```bash
docker run --rm -e DISABLE_TELEMETRY=1 -e DO_NOT_TRACK=1 \
  -e HOST_UID="$(id -u)" -e HOST_GID="$(id -g)" -v "$PWD:/w" -w /w \
  node:26.13.0-slim@sha256:<digest at least 7 days old> sh -c '
  apt-get update && apt-get install -y --no-install-recommends git ca-certificates &&
  git clone https://github.com/dfinity/icskills.git /src &&
  git -C /src checkout 1c28dd2035fd9890ac0953661eb78ae7ee603e4d &&
  npx --yes --before=2026-11-02T21:00:00Z skills@1.7.5 add /src/skills \
    --skill icp-cli --skill canister-security --skill internet-identity \
    --skill stable-memory --skill certified-variables --skill cycles-management \
    -a claude-code -a cursor --copy -y &&
  chown -R "$HOST_UID:$HOST_GID" /w'
```

- The CLI clones by branch name, so it cannot take a commit id directly. The command therefore clones the repository, checks out the commit, and installs from that local copy.
- `npx --before` applies the 7-day rule to the CLI's own dependencies.
- The container runs as root. The last line gives the files back to the user who ran it.

## Consequences

- The pins lag behind the newest releases by design. The monthly upkeep task moves them.
- Task M0-05 closes the `getrandom` check and pins `ic-wasm` and the recipe.
