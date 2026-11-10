# Operations

> Copied from sections 11.4 to 11.6 of `VAULT-PLAN.md`, revision 8, by task M0-02 on 2026-11-09. From then on this file is the source of truth. It changes only through an ADR in `docs/adr/`.
>
> Section numbers are those of `VAULT-PLAN.md`. Sections 2, 6 and 11.4 to 11.6 live in `docs/threat-model.md`, `docs/crypto-spec.md` and `docs/operations.md`. Any other section number means `VAULT-PLAN.md`.

### 11.4 Setup and upkeep risks

Section 2 covers attackers and owner mistakes. This section covers what is left: bringing a vault to life, and keeping it alive for years. Nothing is deployed yet, so the check ran against this plan and the current documentation. Ratings use the scale of section 2. "Before" rates revision 2.

**What the check against the current docs found**

| Topic | Revision 2 said | The docs say | Change |
|---|---|---|---|
| Vault address | `<id>.icp0.io`, to be verified at M4 | Every canister is reachable at `<id>.icp.net`. The pages read name no other domain | Default is `icp.net`. Confirmed at M0, before any passkey exists |
| Mainnet install | `icp deploy -e ic` | Deploy builds, creates, applies settings and installs in one step and picks the mode itself. A reinstall mode exists, and no confirmation is documented | Scripts call each step with a named mode |
| Canister ids | Not mentioned | Stored under `.icp/data/`, which projects usually commit | A private deployment directory. The public repository ignores `.icp/` |
| Freezing threshold | 180 days | The default is 30 days | Declared in the deployment config and checked by `doctor.sh` |
| Public state | Command unknown | `icp canister status --public` shows module hash and controllers to anyone | `verify.sh` and the watchdog use it |
| Staging key | `test_key_1` | `test_key_1` is for development, `key_1` for production | Moot while the network key is deferred |
| Snapshots | One before each upgrade | At most 10 per canister, the canister must be stopped, each is billed as storage | Kept 30 days, then removed |
| API host | To be verified at M4 | `https://icp-api.io` | Written into the page policy |
| Tool version | `icp-cli` 1.6 | The docs index already carries a newer version label | Exact pin at M0, tracked monthly |

Two questions the docs did not answer. The staging rehearsal settles both:

- whether `icp` installs a Wasm above 2 MiB without extra steps
- whether a keyring exists under WSL

**Setup risks**

| # | Risk | Cost | Before | After | Safeguard |
|---|---|---|---|---|---|
| S1 | The vault address rests on a gateway domain that changes, or that was chosen wrongly | Passkeys and the Internet Identity principal are bound to it. Every device has to enrol again | High | Medium | Default follows the docs and is confirmed at M0. The canister redirects every other host. The kit does not depend on the address, and a flow for changing it comes after the first release. Stays a platform risk, R7. |
| S2 | The subnet proves to be the wrong choice and cannot be changed | The vault has to move | Medium | Medium | The vault id is separate from the canister id, so "Move a vault" can be added after the first release without changing a format. Until then: a backup, the offline tool, and entering the data again. |
| S3 | The production key is first used by the real vault, or its name is retired later | Setup fails, or no new device can enrol | Medium | Low | Does not apply to the first version, which uses no threshold key. It applies again when the network key returns. |
| S4 | A deploy applies more than intended: an install mode it picked itself, or settings that changed in the config | Wrong settings or controllers go live unnoticed. At worst a reinstall wipes the vault | Medium | Low | Named steps and modes. The config sits in the private directory. `doctor.sh` compares the live canister with it after every run. |
| S5 | Canister ids and deployment state land in the public repository | The address of your vault is published. Agents and CI see production targets | Medium | Low | Private deployment directory. `.icp/` ignored. CI without credentials. |
| S6 | The first page load is trusted blindly, in a browser full of extensions | The code that creates the vault key and the kit could be altered or watched | Medium | Medium | `verify.sh` right before the claim link is opened. A browser profile without extensions. The gateway stays in the path, R1 and R18. |
| S7 | Funding fails on the day: exchange delays, tokens sent to a wrong account, too few cycles | Days of delay, or lost money | Medium | Low | Funding a week ahead with the documented commands and a small test amount first. `deploy.sh` checks the balance before it creates anything. |
| S8 | The procedure runs for the first time on the real vault, with tools that have moved on | A half-created vault and improvised commands | Medium | Low | Full rehearsal on staging within 7 days before, with the same release, scripts and tool versions. |
| S9 | The Internet Identity has one passkey and no recovery | One lost phone means the kit and a 48-hour wait | Medium | Low | The setup-day checklist asks for a second passkey or a recovery phrase before the claim. |
| S10 | The device cannot do what setup needs, and this shows halfway | A printed kit from an aborted attempt, or a weaker profile chosen in a hurry | Medium | Low | Device check before anything is created. |

**Upkeep risks**

| # | Risk | Cost | Before | After | Safeguard |
|---|---|---|---|---|---|
| K1 | Nobody is told when something goes wrong: cycles low, code or controllers changed, vault unreachable | The problem is found at the next unlock, or too late | Medium | Low | Daily watchdog outside the vault. It reads public data, holds no key and alerts through the scheduler it runs on. |
| K2 | An upgrade breaks the vault or its data | Downtime, or loss of what was written since the snapshot | Medium | Low | Staging first. Named upgrade mode. Snapshot kept 30 days, backup file before. No downgrades. Upgrade test against a full vault. |
| K3 | A dependency or a tool is poisoned, or a published flaw goes unnoticed | Malicious or vulnerable code in a release | Medium | Low | Exact pins. Seven-day release age. Monthly update task with a fixed list of sources. Second build in CI. |
| K4 | An old release can no longer be rebuilt | A deployed hash cannot be verified any more | Medium | Low | Build inputs archived with each release. Monthly rebuild of the latest release. |
| K5 | The platform changes underneath: gateway domain, API version, Internet Identity, subnet, PRF support in browsers | The embedded web app stops working, or a setup choice becomes invalid | Medium | Medium | Watch list checked monthly. Sign-in and unlock on staging every month. A rebuild and upgrade every year. Kit, backup file and offline tool need none of it. |
| K6 | The vault has to move to another canister or subnet | Without preparation, a migration under pressure | Medium | Medium | Until "Move a vault" exists, after the first release: a current backup, the offline tool, and entering the data again. The yearly drill decrypts a backup offline. |
| K7 | The switch to self-controlled upgrades goes wrong | A vault that can never be upgraded again | Medium | Low | Two-step switch. The backup controller stays until one self-upgrade worked. Upgrades can be approved with the command-line tool and the kit. |
| K8 | Upkeep depends on one person and on chat sessions that are gone | Runbooks rot, routines stop | Medium | Low | Every routine is a script or a prompt in the repository. Each run is logged. Runbooks are executed on a calendar. |
| K9 | A flaw needs a fix within days | A slow, improvised release | Medium | Low | `SECURITY.md` points to GitHub's private vulnerability reporting. A hot-fix runbook that keeps the staging rehearsal. In stage B the cooldown still applies, R19. |
| K10 | The tag signing key is lost or stolen | No releases, or a false one | Medium | Low | Key on a hardware token. Rotation runbook. The second build in CI and, in stage B, the cooldown limit the damage. |

**What the assessment shows**

- Setup is where the choices are made that cannot be taken back. The table in 11.5 names each one and its way out. The address domain rated High because revision 2 defaulted to the older domain and checked it only at the milestone that creates the first vault.
- Upkeep fails through silence. The vault cannot call for help, so a watchdog outside it and a calendar do.
- Five risks stay Medium. Three belong to the platform: the address domain, the gateway in front of the first page load, and change underneath. Two wait for "Move a vault", which the cut line puts after the first release.
- The minimum that protects the data is small: enough cycles, a current backup file and the kit. Everything else in this section buys early warning and convenience.

### 11.5 Setup procedure

A vault is set up twice: once on staging as a rehearsal, then for real. Both runs use the same release, the same scripts and the same tool versions.

**Choices that are fixed at setup**

| Fixed | Why | Way out |
|---|---|---|
| Subnet | Chosen when the canister is created | A backup, the offline tool and a new vault. After the first release: "Move a vault" |
| Canister id | Given at creation, part of the address | The same |
| Gateway domain | Passkeys and the Internet Identity principal are bound to the full address | The kit under the new address, with its waiting period. After the first release: "Change the address" |
| Vault id | Bound into every ciphertext | None needed. It moves with the vault |
| First controllers | Set at creation | Changed by script in stage A |

**Steps**

| Step | When | What | Checkpoint |
|---|---|---|---|
| 1. Fund | A week before | `docs/runbooks/top-up.md`: account id, small test transfer, full transfer, mint cycles | `icp cycles balance` covers both vaults |
| 2. Prepare identities | A week before | A second passkey or a recovery phrase on your Internet Identity. Create `vault-staging` in the development environment. Create `vault-prod` and the backup controller in the operating environment of ADR 0012, which holds only the release scripts, Docker and icp-cli and where no agent runs | You can sign in to Internet Identity from two devices. `vault-prod` does not exist in the development environment |
| 3. Rehearse | Within 7 days before | Steps 4 to 7 against staging with fake data, then one upgrade and one kit recovery | Every checkpoint below passed on staging. Problems are fixed in the scripts, not worked around |
| 4. Create | Setup day | `deploy.sh production`: balance check, create on the subnet with named cycles and controllers, apply settings, install in install mode | The script stops at the first failed check |
| 5. Verify | Setup day | `doctor.sh` and `verify.sh`: subnet, environment name, freezing threshold, controllers, module hash against the release | Both report healthy. Only then is the claim link shown |
| 6. First run | Setup day, right after | The setup-day checklist, then the flow of section 6.6 | Kit entered in full. Practice unlock succeeded |
| 7. Close | Same day | Second unlock key. Bookmark. Kit copies in their two places. Watchdog switched on. First line in the log | The safety check is green except for the 30-day kit check |

**Setup-day checklist**, also kept as `docs/runbooks/setup-day.md`:

- Thirty quiet minutes, and nobody else can see the screen.
- You work in the operating environment of ADR 0012. No agent session is linked to this machine or this user account.
- A printer you trust, or pen and paper.
- A browser profile without extensions. A private window will do.
- The second device at hand.
- The two places for the kit copies are decided.
- The staging rehearsal is less than 7 days old.

### 11.6 Upkeep routine

**The watchdog** is `scripts/watch.sh`, run once a day by a scheduler of your choice. It needs the canister id and nothing else.

- It reads the module hash and the controllers with `icp canister status --public`, calls `status`, and loads the start page.
- It fails when the vault does not answer, when the module hash is not on the list of released hashes, when the controllers differ from the recorded ones, when the health value is not `ok`, or when `status` shows a pending recovery, ADR 0007.
- A pending recovery that you did not start means that someone holds your kit. Open the vault on a device you trust, read the blocking warning, and choose kit replacement or re-key.
- A failure reaches you through the scheduler: a failed workflow run that is mailed to you, or a local task that shows a notification. `ops-template/` ships a scheduled workflow for a private repository. **DECIDE** (D17)
- It holds no key and can change nothing. Whoever runs it learns the address of the vault and nothing more.
- After a failure the rule is: do not unlock before `doctor.sh` and `verify.sh` have run.

**Calendar**

| When | What | Who | Time |
|---|---|---|---|
| Daily | Watchdog | Scheduler | None |
| Monthly | Update task from Appendix E: advisories, pins, watch list. Sign in and unlock on staging. Confirm that the watchdog still runs | Agent prepares, you review | 20 minutes |
| Every 3 months, from M6 | Backup and "Verify a backup". `doctor.sh`. Review the list of unlock keys | You | 15 minutes |
| Yearly | Kit check. Backup drill: decrypt a backup with the offline tool. Rebuild and upgrade even without changes. Run every runbook once and date it. Look again for a funded or volunteer review | You | 2 hours |
| On each release | Upgrade runbook, staging first | You | 30 minutes |

**Watch list**, checked in the monthly task:

| What can change | Where it shows | What to do |
|---|---|---|
| Gateway domain | Custom domains guide, developer forum | The kit under the new address. A flow for it comes after the first release |
| HTTP API versions | SDK release notes | Upgrade within the announced window |
| Internet Identity | Its release notes, and the monthly sign-in on staging | Fix and release before production breaks |
| Subnet: members, confidential setup, public access | The subnet page on the dashboard | A backup, the offline tool and a new vault. "Move a vault" comes after the first release |
| Cycle prices | Cycle costs reference | Adjust the funding |
| PRF in browsers and authenticators | An unlock on each of your devices after a major browser update | Another device, or the kit. A passphrase key comes after the first release |
| `icp-cli` and SDK versions | Release notes | Bump the pins in one pull request |

**Urgent fixes.** `SECURITY.md` gives a private way to report a flaw: GitHub's private vulnerability reporting, which needs no mailbox. `docs/runbooks/hotfix.md` is the short path: fix, review, release, staging, production, on the same day if needed. It may skip the 7-day release age through the documented exclusion. It never skips staging.

**The log.** Every script appends one line to `log.md` in the private directory: date, script, environment, release, result. The yearly routine reads it.

**What upkeep is for.** A funded vault that nobody tends keeps working until its cycles run out or a platform change breaks the embedded web app. Even then the data stays readable with the kit, a backup file and the offline tool.
