# Cloud Execution Environment and TPS FMEA

**Document type:** security-safe runtime census, execution-boundary specification, and TPS-oriented Failure Mode and Effects Analysis  
**Repository:** `seanchatmangpt/dteam`  
**Branch at census start:** `agent/innovation-80-20-closure`  
**Parent head observed before this document:** `089b7d59e58092ce650a8f9a28cd3bb4ad07f669`  
**Runtime census:** `2026-08-02T20:16:50Z` / `2026-08-02T13:16:50-07:00`  
**Model/orchestrator:** GPT-5.6 Thinking  
**Standing:** `OBSERVED_SNAPSHOT`; this is not a claim that the provider host, every future session, or every connector has identical configuration.

## Purpose

This document makes the cloud execution environment explicit enough to support:

1. reproducible engineering decisions;
2. receipt-bound validation;
3. Toyota Production System controls such as jidoka, andon, poka-yoke, standard work, heijunka, genchi genbutsu, and WIP limits;
4. a bounded Failure Mode and Effects Analysis for the tool/transaction-processing path used to inspect, modify, validate, and publish this repository.

`TPS FMEA` is used here in both compatible senses:

- **transaction/tool-processing system:** the end-to-end path from user intent through model planning, tools, storage, connectors, CI, receipts, and Git;
- **Toyota Production System:** the operating controls used to expose abnormal conditions, stop propagation, remove waste, and make quality intrinsic to the process.

## Disclosure and evidence boundary

This is the most complete **security-safe, observable guest/runtime inventory** available from the execution session and repository configuration. It intentionally excludes:

- credential values, access tokens, cookies, secret environment-variable values, private keys, and internal service endpoints;
- hidden prompt text, private reasoning, provider control-plane implementation, physical host identity, tenancy placement, and unobservable network topology;
- claims about package or service availability that were not probed or declared by the platform contract.

Root inside the isolated guest is **not** provider-host root. Configuration may change between sessions. Every operational use must refresh the census and bind it to the exact Git commit being validated.

---

## 1. Execution topology

```text
User intent
   |
   v
GPT-5.6 Thinking orchestration
   |
   +--> GitHub connector --------> GitHub repository / PR / Actions API
   +--> Web retrieval -----------> public internet retrieval plane
   +--> Isolated container ------> shell, filesystem, document tooling
   +--> Private Python kernel ---> bounded private computation
   +--> User-visible Python -----> tables, plots, generated files
   +--> File/connectors ---------> Gmail, Calendar, Contacts, Drive
   +--> Image generation --------> generated or edited images
   +--> Automations -------------> scheduled or conditional future runs
   |
   v
Commit, PR state, artifacts, receipts, and user-visible response
```

### Plane separation

The network and persistence properties of these planes are not interchangeable:

| Plane | Primary use | Network behavior | Persistence |
|---|---|---|---|
| Model/orchestrator | reasoning, tool routing, response synthesis | no direct socket access | conversation-scoped context |
| Isolated container | shell commands, builds, file transformation | DNS/HTTPS egress unavailable in this census | guest filesystem is ephemeral; `/mnt/data` is the artifact handoff path |
| Private Python | analysis and private computation | internet disabled | kernel/session scoped |
| User-visible Python | generated tables, plots, and files | internet disabled | files must be explicitly linked from `/mnt/data` |
| Web retrieval | current public information | internet-enabled through a separate managed plane | results are response scoped |
| GitHub connector | repository, PR, issue, content, and Actions operations | managed connector network | writes persist in GitHub |
| Google connectors | user-authorized mail, calendar, contacts, and Drive | managed connector network | writes persist in the selected service |
| Automations | future scheduled/conditional execution | managed future-run plane | persisted task definition |
| GitHub Actions | repository CI and evidence manufacture | GitHub-hosted runner network | logs/artifacts follow repository retention settings |

No operation may infer container egress merely because web retrieval or a connector succeeds.

---

## 2. Observed isolated-container inventory

### 2.1 Compute and operating system

| Property | Observed value | Engineering interpretation |
|---|---|---|
| Guest OS | Debian GNU/Linux 13.3 (`trixie`) | mutable session image; do not assume parity with GitHub Actions |
| Kernel | Linux `6.12.13`, x86-64 | guest kernel interface presented by the isolation layer |
| Virtualization | KVM, full virtualization | guest/root boundary applies |
| CPU model | Intel Xeon Platinum 8573C | AVX/AVX2/AVX-512 and related flags visible |
| Logical CPUs visible | 5 | not the enforceable scheduling budget |
| cgroup CPU quota | `400000 100000` | effective quota is 4 CPU cores |
| NUMA | 1 node | no NUMA-aware scaling assumption required |
| Memory reported by `free` | 5.9 GiB total | host/namespace-visible figure |
| cgroup memory limit | 4,294,967,296 bytes | enforceable memory ceiling is 4 GiB |
| Swap | 0 bytes | memory exhaustion can terminate processes abruptly |
| Root disk | 63 GiB; 39 GiB available at census | ephemeral capacity, not durable evidence storage |
| Shared memory | approximately 988 MiB | relevant to browsers, multiprocessing, and large array exchange |
| Guest user | `uid=0(root)` | privileged only within the guest boundary |
| PID cgroup limit | `max` | process creation is instead bounded by memory and user/process limits |
| Max user processes | 7,851 | broad but finite |
| Open-file limit | 1,024 | parallel I/O and test fan-out must be bounded |
| Stack size | 8 MiB | deep recursion can fail |
| Core dumps | disabled (`0`) | post-mortem native crash analysis may lack a core |
| Locked memory | 8 MiB | unsuitable for large mlock-dependent workloads |
| Runtime clock | UTC | user timezone is America/Los_Angeles |
| Filesystem | ext4 root; read-only `/caas_toolbox`; cgroup v2 | writes must target writable guest paths |
| `/mnt/data` | writable shared artifact path | verify exact path before exposing a sandbox link |
| `/home/oai` | platform skills and working resources | not a user-deliverable location |
| `/tmp` | writable sticky temporary storage | disposable intermediate data only |

### 2.2 Network

Observed from the isolated container:

```text
github.com      -> temporary DNS failure
api.github.com  -> temporary DNS failure
pypi.org        -> temporary DNS failure
HTTPS probes    -> HTTP 000 / name-resolution timeout
```

Therefore:

- `git push`, `gh`, `cargo install`, `pip install`, `npm install`, direct API calls, and source downloads cannot be assumed to work from the container;
- managed connectors and the web retrieval plane remain separate alternatives;
- dependency acquisition must be pre-baked, vendored, cached in a reachable plane, or executed in GitHub Actions.

### 2.3 Security posture visible inside the guest

| Property | Observed value | Note |
|---|---|---|
| Effective/bounding capabilities | `00000000a00425fb` | capability mask is guest-scoped; values must not be interpreted as host authority |
| `NoNewPrivs` | `0` | no-new-privileges is not set for the current process |
| Seccomp | `0` | no seccomp filter reported for the current process |
| `/proc/sys` | read-only mount | kernel tuning is restricted |
| `/sys` | read-only mount | host/device mutation is restricted |
| `/caas_toolbox` | read-only virtiofs | platform-provided tools are immutable from the guest |

This is not a penetration-test result. It is only the process-visible state needed for engineering risk analysis.

---

## 3. Installed toolchain census

### 3.1 Languages and command-line tools

| Tool | Version/status |
|---|---|
| Python | 3.13.5 |
| pip | 25.1.1 |
| Node.js | 22.16.0 |
| npm / npx | 10.9.2 |
| Java / javac | OpenJDK 21.0.10 |
| Go | 1.23.2 |
| GCC / G++ | 14.2.0 |
| Clang | 17.0.0 |
| Swift | 6.2.1 |
| Ruby | 3.3.8 |
| Perl | 5.40.1 |
| PHP | 8.4.16 |
| Git | 2.47.3 |
| CMake | 3.31.6 |
| GNU Make | 4.4.1 |
| jq | 1.7 |
| ripgrep | 14.1.1 |
| curl | 8.10.1 |
| wget | 1.25.0 |
| Graphviz `dot` | 2.42.4 |
| LibreOffice | 25.2.3.2 |
| FFmpeg | 7.1.3 |
| Pandoc | 3.1.11.1 |
| Poppler `pdftoppm` | 25.06.0 |
| `rustc` | **missing in isolated container** |
| `cargo` | **missing in isolated container** |
| GitHub CLI `gh` | **missing in isolated container** |
| `fd` | **missing in isolated container** |

The repository itself pins:

```toml
[toolchain]
channel = "nightly-2026-06-02"
profile = "minimal"
components = ["rustfmt", "clippy"]
targets = ["wasm32-unknown-unknown"]
```

The current innovation workflow separately installs `dtolnay/rust-toolchain@stable`. That is a material toolchain-divergence risk and appears in the FMEA.

### 3.2 Python artifact and analysis libraries

| Library | Version/status |
|---|---|
| `artifact_tool` | import succeeds; module version `0.0.0` |
| NumPy | 2.3.5 |
| pandas | 2.2.3 |
| Matplotlib | 3.10.8 |
| openpyxl | 3.1.5 |
| Pillow | 12.2.0 |
| ReportLab | 4.4.9 |
| pypdf | 5.9.0 |
| PyMuPDF / `fitz` | 1.26.7 |
| python-docx | 1.2.0 |
| python-pptx | 1.0.2 |
| lxml | 6.1.1 |
| requests | 2.32.5 |
| httpx | 0.28.1 |
| SciPy | 1.17.0 |
| NetworkX | 3.6.1 |
| Pydantic | 2.13.4 |
| `caas_jupyter_tools` | import succeeds; distribution version unreported |
| IPython kernel | 7.2.0 |

### 3.3 Managed capabilities

The orchestrator can invoke managed tools for:

- current web retrieval, page opening, PDF screenshots, finance, weather, sports, calculation, and time;
- GitHub repositories, branches, commits, PRs, issues, file contents, workflow runs, jobs, logs, and artifacts;
- Gmail, Google Calendar, Google Contacts, and Google Drive;
- document, PDF, spreadsheet, and presentation creation through platform skills and container libraries;
- image generation and image editing;
- scheduled and conditional automations, with one hour as the highest supported recurrence frequency;
- user settings and local hotline lookup where applicable.

Managed-tool availability is permission-, schema-, quota-, and service-health-dependent. A tool declaration is not proof that a specific resource is authorized or that a call will succeed.

---

## 4. Repository and CI environment

### 4.1 Current stacked PR context

| Field | Value at census start |
|---|---|
| Repository | `seanchatmangpt/dteam` |
| PR | `#10` |
| State | open, draft, mergeable at observation time |
| Head branch | `agent/innovation-80-20-closure` |
| Head SHA | `089b7d59e58092ce650a8f9a28cd3bb4ad07f669` |
| Base branch | `agent/ggen-alive-closure` |
| Base SHA | `8354e411ca333df0dddc02a0eb4eadff4591c3a8` |
| Changed files before this document | 39 |
| Commits before this document | 20 |

This is a stacked PR. Diff and acceptance claims must be evaluated against the recorded base, not the repository default branch.

### 4.2 Innovation workflow

The branch contains a GitHub Actions workflow with these material properties:

| Property | Current value |
|---|---|
| Runner | `ubuntu-24.04` |
| Timeout | 20 minutes |
| Checkout | full history, explicit PR head/ref |
| Concurrency | one group per PR/ref; in-progress run cancelled |
| Permission | `contents: write` |
| Rust setup | `dtolnay/rust-toolchain@stable` with rustfmt and clippy |
| Mutation before validation | Python patch script, then `cargo fmt` |
| Validation | check, test, clippy |
| Acceptance parsing | `grep` against textual output |
| Evidence | audit, snapshot, support, and telco text files |
| Write-back | commits all changes and pushes to the same head branch |

The last three properties—mutation during validation, textual acceptance, and self-push—are primary TPS stop-the-line findings. A validation workflow must prove the reviewed subject, not manufacture a new unreviewed subject while proving it.

---

## 5. TPS FMEA method

### 5.1 Rating scale

| Rating | Severity `S` | Occurrence `O` | Detection difficulty `D` |
|---|---|---|---|
| 1–2 | negligible/local inconvenience | rare under exceptional conditions | almost certain immediate detection |
| 3–4 | recoverable workflow degradation | occasional | strong automated detection |
| 5–6 | material delay, partial corruption, or wrong decision | recurring | mixed automated/manual detection |
| 7–8 | invalid evidence, substantial rework, release or security impact | likely in normal operation | difficult to detect before propagation |
| 9–10 | integrity breach, secret exposure, unrecoverable loss, or false production standing | frequent/systemic or unacceptable even if rare | latent or likely to escape |

`RPN = S × O × D`.

### 5.2 Action thresholds

| Condition | Required response |
|---|---|
| Severity 10 | stop the line regardless of RPN |
| RPN ≥ 300 | P0: block readiness and release |
| RPN 200–299 | P1: corrective action required before normal operation |
| RPN 120–199 | P2: scheduled control with owner and verification |
| RPN < 120 | monitor, standardize, and prevent regression |

Detection difficulty is scored high when the failure can produce plausible-looking output. A visible hard failure generally has a lower `D` than a silent integrity failure.

---

## 6. FMEA register

| ID | Status | Layer | Failure mode | S | O | D | RPN |
|---|---|---|---|---:|---:|---:|---:|
| FM-01 | Observed | Assurance | Completion or ALIVE is asserted without exact-head execution evidence | 10 | 5 | 8 | **400** |
| FM-02 | Observed | CI / Git | Validation workflow mutates and pushes the same branch it is validating | 9 | 6 | 7 | **378** |
| FM-06 | Observed | Runtime | Visible memory reports 5.9 GiB while cgroup enforcement is 4 GiB | 9 | 7 | 6 | **378** |
| FM-07 | Observed | CI / Git | Branch head changes between inspection, file update, workflow start, and final claim | 9 | 7 | 6 | **378** |
| FM-03 | Foreseeable | Evidence | Receipt or support-bundle digest is not cryptographically bound to the Git commit and environment manifest | 9 | 5 | 8 | **360** |
| FM-08 | Foreseeable | Environment | Tool/package image changes without a committed environment manifest | 8 | 6 | 7 | **336** |
| FM-05 | Observed | Evidence | Text grep is used as semantic acceptance for JSON or standing | 9 | 5 | 7 | **315** |
| FM-09 | Observed | Permissions | Validation job has contents:write although validation does not require repository mutation | 10 | 3 | 9 | **270** |
| FM-12 | Foreseeable | Security | Secrets, tokens, internal endpoints, or credentials are copied into logs, artifacts, or documentation | 10 | 3 | 9 | **270** |
| FM-10 | Foreseeable | Connectors | Paged or truncated connector output is mistaken for the complete repository state | 8 | 5 | 7 | **280** |
| FM-04 | Observed | CI / Toolchain | Workflow installs Rust stable while repository pins nightly-2026-06-02 | 8 | 6 | 6 | **288** |
| FM-11 | Observed | Network | Container network is assumed to match web or connector network | 8 | 7 | 5 | **280** |
| FM-13 | Observed | Documents | PDF parsing is treated as complete without rendering pages containing charts, images, or layout-dependent evidence | 7 | 5 | 7 | **245** |
| FM-20 | Observed | Isolation | Root inside the guest is mistaken for host-level authority or security | 10 | 3 | 8 | **240** |
| FM-15 | Observed | Artifacts | Ephemeral root filesystem is treated as durable storage | 9 | 5 | 5 | **225** |
| FM-19 | Observed | Time | UTC runtime time is confused with the user's America/Los_Angeles timezone | 6 | 6 | 6 | **216** |
| FM-14 | Observed | Resources | Five visible CPUs are scheduled although cgroup quota equals four cores | 7 | 6 | 5 | **210** |
| FM-18 | Foreseeable | Tool output | Long logs, diffs, or file reads are truncated and the missing tail contains the root cause | 7 | 5 | 6 | **210** |
| FM-17 | Observed | Branching | A stacked PR is compared to or retargeted onto the wrong base | 8 | 5 | 5 | **200** |
| FM-26 | Observed | CI | Formatting or migration scripts modify source during a validation job | 9 | 5 | 6 | **270** |
| FM-21 | Observed | Dependency | A missing compiler or CLI is assumed installable even though the container has no DNS/egress | 7 | 7 | 3 | **147** |
| FM-31 | Foreseeable | Artifacts | Digest varies across platforms due to newline, path, ordering, locale, or timestamp nondeterminism | 7 | 4 | 6 | **168** |
| FM-23 | Observed | Runtime | Open-file limit of 1024 is exceeded by parallel tests, crawlers, or artifact fan-out | 7 | 4 | 6 | **168** |
| FM-22 | Foreseeable | Runtime | No swap plus memory-intensive compile/render workload causes abrupt OOM termination | 8 | 5 | 4 | **160** |
| FM-28 | Foreseeable | Access | Connector authorization, installation scope, or repository permissions are incomplete or stale | 8 | 4 | 5 | **160** |
| FM-30 | Observed | Documents | OCR is used on unsupported language or as a first-line parser | 6 | 4 | 6 | **144** |
| FM-25 | Observed | CI | Lint runs without -D warnings or differs between developer and CI policy | 6 | 6 | 4 | **144** |
| FM-38 | Observed | Model boundary | Internal reasoning, hidden instructions, or provider internals are treated as auditable runtime facts | 8 | 3 | 7 | **168** |
| FM-24 | Observed | Execution | Private Python execution exceeds its bounded runtime or a container command times out | 7 | 6 | 3 | **126** |
| FM-16 | Observed | GitHub API | A file update uses a stale blob SHA or concurrent writers update the same path | 7 | 6 | 3 | **126** |
| FM-32 | Foreseeable | Evidence | Support bundle contains reproduction commands that depend on unavailable local tools | 7 | 5 | 5 | **175** |
| FM-34 | Foreseeable | Git | Mergeability is assumed stable after new commits or base movement | 7 | 4 | 4 | **112** |
| FM-29 | Observed | Files | Library/connector file reference is assumed to be a local sandbox path | 6 | 5 | 4 | **120** |
| FM-37 | Foreseeable | Human interface | User interruption or new instructions arrive while mutations are in flight | 6 | 5 | 5 | **150** |
| FM-35 | Foreseeable | Ownership | Files created as root are later consumed by a non-root process or mounted workspace | 6 | 4 | 4 | **96** |
| FM-27 | Observed | Connectors | Search or connector upstream returns transient 5xx and is treated as authoritative absence | 5 | 6 | 4 | **120** |
| FM-33 | Observed | Workflow | Large workflow fan-out queues many redundant jobs for documentation-sized changes | 5 | 7 | 3 | **105** |
| FM-36 | Observed | Automation | A request requires sub-hour monitoring or event-triggered webhooks unavailable to the scheduler | 4 | 3 | 2 | **24** |

### Detailed controls and verification

#### FM-01 — Completion or ALIVE is asserted without exact-head execution evidence

- **Classification:** Observed · Assurance · `S=10` `O=5` `D=8` `RPN=400`
- **Effect:** False release confidence; defects promoted as proven.
- **TPS countermeasure:** Jidoka: stop the line unless commit-bound receipts prove every required gate.
- **Required verification:** Receipt names commit SHA, commands, exit codes, artifact digests, and standing.

#### FM-02 — Validation workflow mutates and pushes the same branch it is validating

- **Classification:** Observed · CI / Git · `S=9` `O=6` `D=7` `RPN=378`
- **Effect:** Moving target, recursive runs, race conditions, evidence bound to the wrong head.
- **TPS countermeasure:** Poka-yoke: validation is read-only; isolate write-back behind explicit workflow_dispatch.
- **Required verification:** Validation token has contents:read and head SHA is unchanged from start to finish.

#### FM-06 — Visible memory reports 5.9 GiB while cgroup enforcement is 4 GiB

- **Classification:** Observed · Runtime · `S=9` `O=7` `D=6` `RPN=378`
- **Effect:** Sizing decisions exceed the real limit and processes are OOM-killed.
- **TPS countermeasure:** Genchi genbutsu: size against cgroup limits, not host-visible free output.
- **Required verification:** Preflight records memory.max and enforces an application budget below it.

#### FM-07 — Branch head changes between inspection, file update, workflow start, and final claim

- **Classification:** Observed · CI / Git · `S=9` `O=7` `D=6` `RPN=378`
- **Effect:** Patch, test result, or review references stale source.
- **TPS countermeasure:** Poka-yoke: optimistic concurrency with expected head SHA at every mutation and claim.
- **Required verification:** Final report re-fetches PR head and matches workflow head_sha.

#### FM-03 — Receipt or support-bundle digest is not cryptographically bound to the Git commit and environment manifest

- **Classification:** Foreseeable · Evidence · `S=9` `O=5` `D=8` `RPN=360`
- **Effect:** Evidence can be replayed against a different source or runtime.
- **TPS countermeasure:** Jidoka + traceability: include commit/tree/toolchain/environment digests in the receipt subject.
- **Required verification:** Independent verifier recomputes all digests from the exact commit.

#### FM-08 — Tool/package image changes without a committed environment manifest

- **Classification:** Foreseeable · Environment · `S=8` `O=6` `D=7` `RPN=336`
- **Effect:** Previously green procedures become irreproducible.
- **TPS countermeasure:** Standard work: emit a versioned machine-readable environment manifest on every run.
- **Required verification:** Manifest diff is reviewed and receipt contains its digest.

#### FM-05 — Text grep is used as semantic acceptance for JSON or standing

- **Classification:** Observed · Evidence · `S=9` `O=5` `D=7` `RPN=315`
- **Effect:** Malformed or contradictory output can satisfy the gate.
- **TPS countermeasure:** Poka-yoke: strict JSON/schema parsing and explicit boolean predicates.
- **Required verification:** jq/schema validator rejects malformed, duplicate, or contradictory fields.

#### FM-04 — Workflow installs Rust stable while repository pins nightly-2026-06-02

- **Classification:** Observed · CI / Toolchain · `S=8` `O=6` `D=6` `RPN=288`
- **Effect:** Local/release behavior diverges; nightly-only code or lints fail unpredictably.
- **TPS countermeasure:** Standard work: consume rust-toolchain.toml without an overriding channel.
- **Required verification:** rustc -Vv and cargo -V captured in artifacts and match the lock.

#### FM-09 — Validation job has contents:write although validation does not require repository mutation

- **Classification:** Observed · Permissions · `S=10` `O=3` `D=9` `RPN=270`
- **Effect:** Compromised dependency or script can alter source.
- **TPS countermeasure:** Least privilege + jidoka: default contents:read; separate signed write path.
- **Required verification:** Workflow permission audit fails on unexpected write scopes.

#### FM-12 — Secrets, tokens, internal endpoints, or credentials are copied into logs, artifacts, or documentation

- **Classification:** Foreseeable · Security · `S=10` `O=3` `D=9` `RPN=270`
- **Effect:** Credential compromise and repository exposure.
- **TPS countermeasure:** Poka-yoke: allowlist fields, redact values, secret scan before commit/upload.
- **Required verification:** Secret-scanning gate and manual review show no credential material.

#### FM-26 — Formatting or migration scripts modify source during a validation job

- **Classification:** Observed · CI · `S=9` `O=5` `D=6` `RPN=270`
- **Effect:** Tests execute code different from the reviewed commit.
- **TPS countermeasure:** Jidoka: validation must fail on dirty tree; mutation happens in a separate reviewed commit.
- **Required verification:** git diff --exit-code before and after every gate.

#### FM-10 — Paged or truncated connector output is mistaken for the complete repository state

- **Classification:** Foreseeable · Connectors · `S=8` `O=5` `D=7` `RPN=280`
- **Effect:** Files, failures, comments, or workflow jobs are omitted from decisions.
- **TPS countermeasure:** Standard work: follow cursors/pages and record completeness markers.
- **Required verification:** Receipt records page count, item count, and terminal cursor absence.

#### FM-11 — Container network is assumed to match web or connector network

- **Classification:** Observed · Network · `S=8` `O=7` `D=5` `RPN=280`
- **Effect:** Commands attempt package/API access that cannot resolve DNS while other planes still work.
- **TPS countermeasure:** Visual management: declare network plane per operation; no implicit egress assumptions.
- **Required verification:** Preflight probes DNS/HTTPS in the exact execution plane.

#### FM-13 — PDF parsing is treated as complete without rendering pages containing charts, images, or layout-dependent evidence

- **Classification:** Observed · Documents · `S=7` `O=5` `D=7` `RPN=245`
- **Effect:** Material evidence is omitted or misinterpreted.
- **TPS countermeasure:** Genchi genbutsu: screenshot/render every relevant visual page.
- **Required verification:** Page-image review is cited alongside parsed text.

#### FM-20 — Root inside the guest is mistaken for host-level authority or security

- **Classification:** Observed · Isolation · `S=10` `O=3` `D=8` `RPN=240`
- **Effect:** Unsafe assumptions about mounts, devices, capabilities, or persistence.
- **TPS countermeasure:** Visual control: label guest-root boundary; prohibit host-control claims.
- **Required verification:** Environment report distinguishes guest UID from provider/host privilege.

#### FM-15 — Ephemeral root filesystem is treated as durable storage

- **Classification:** Observed · Artifacts · `S=9` `O=5` `D=5` `RPN=225`
- **Effect:** Generated evidence or patches disappear when the session ends.
- **TPS countermeasure:** Standard work: persist deliverables to Git, approved connectors, or /mnt/data before declaring completion.
- **Required verification:** Artifact existence is re-opened from the durable target.

#### FM-19 — UTC runtime time is confused with the user's America/Los_Angeles timezone

- **Classification:** Observed · Time · `S=6` `O=6` `D=6` `RPN=216`
- **Effect:** Wrong dates, schedules, deadlines, or incident ordering.
- **TPS countermeasure:** Standard work: always record UTC and user-local timestamps together.
- **Required verification:** Both ISO-8601 timestamps appear in receipts and automation definitions.

#### FM-14 — Five visible CPUs are scheduled although cgroup quota equals four cores

- **Classification:** Observed · Resources · `S=7` `O=6` `D=5` `RPN=210`
- **Effect:** Oversubscription, timeouts, unstable benchmark and test duration.
- **TPS countermeasure:** Heijunka: schedule workers from cpu.max, not nproc.
- **Required verification:** Worker count derives from quota and is recorded.

#### FM-18 — Long logs, diffs, or file reads are truncated and the missing tail contains the root cause

- **Classification:** Foreseeable · Tool output · `S=7` `O=5` `D=6` `RPN=210`
- **Effect:** Incorrect repair or false clean bill of health.
- **TPS countermeasure:** Andon: detect truncation and retrieve focused ranges/pages/jobs.
- **Required verification:** Completeness marker or final line is observed.

#### FM-17 — A stacked PR is compared to or retargeted onto the wrong base

- **Classification:** Observed · Branching · `S=8` `O=5` `D=5` `RPN=200`
- **Effect:** Diff includes unrelated migrations or omits required dependencies.
- **TPS countermeasure:** Standard work: record base/head SHAs and compare_commits before review.
- **Required verification:** PR metadata and merge-base are captured in the receipt.

#### FM-32 — Support bundle contains reproduction commands that depend on unavailable local tools

- **Classification:** Foreseeable · Evidence · `S=7` `O=5` `D=5` `RPN=175`
- **Effect:** Handoff recipient cannot reproduce the claimed result.
- **TPS countermeasure:** Genchi genbutsu: test reproduction in a clean declared environment.
- **Required verification:** Fresh-run transcript is included.

#### FM-31 — Digest varies across platforms due to newline, path, ordering, locale, or timestamp nondeterminism

- **Classification:** Foreseeable · Artifacts · `S=7` `O=4` `D=6` `RPN=168`
- **Effect:** False drift or unverifiable support bundles.
- **TPS countermeasure:** Standard work: canonical serialization, sorted keys, normalized LF, UTC, relative paths.
- **Required verification:** Cross-platform golden fixture produces identical digest.

#### FM-23 — Open-file limit of 1024 is exceeded by parallel tests, crawlers, or artifact fan-out

- **Classification:** Observed · Runtime · `S=7` `O=4` `D=6` `RPN=168`
- **Effect:** EMFILE errors and cascading I/O failures.
- **TPS countermeasure:** Heijunka: cap fan-out and close resources deterministically.
- **Required verification:** FD count is monitored during stress tests.

#### FM-38 — Internal reasoning, hidden instructions, or provider internals are treated as auditable runtime facts

- **Classification:** Observed · Model boundary · `S=8` `O=3` `D=7` `RPN=168`
- **Effect:** False documentation and disclosure risk.
- **TPS countermeasure:** Poka-yoke: document only observable runtime facts and public capability contracts.
- **Required verification:** Every environment claim has a probe/config source or is labeled unknown.

#### FM-22 — No swap plus memory-intensive compile/render workload causes abrupt OOM termination

- **Classification:** Foreseeable · Runtime · `S=8` `O=5` `D=4` `RPN=160`
- **Effect:** Partial files, killed jobs, and missing diagnostics.
- **TPS countermeasure:** Muri reduction: bounded concurrency, streaming, chunking, and memory budgets.
- **Required verification:** Peak RSS and exit cause are recorded.

#### FM-28 — Connector authorization, installation scope, or repository permissions are incomplete or stale

- **Classification:** Foreseeable · Access · `S=8` `O=4` `D=5` `RPN=160`
- **Effect:** Reads/writes silently omit protected resources or fail late.
- **TPS countermeasure:** Poka-yoke: preflight identity and permission scope before work.
- **Required verification:** Authenticated principal and repository permission level are receipted.

#### FM-37 — User interruption or new instructions arrive while mutations are in flight

- **Classification:** Foreseeable · Human interface · `S=6` `O=5` `D=5` `RPN=150`
- **Effect:** Scope drift, duplicate writes, or stale final report.
- **TPS countermeasure:** Kanban/WIP limit: finish atomic mutation, acknowledge delta, re-resolve head and scope.
- **Required verification:** Post-interruption state check is recorded.

#### FM-21 — A missing compiler or CLI is assumed installable even though the container has no DNS/egress

- **Classification:** Observed · Dependency · `S=7` `O=7` `D=3` `RPN=147`
- **Effect:** Validation stops after implementation; work is stranded.
- **TPS countermeasure:** Poka-yoke: capability preflight before selecting the execution path.
- **Required verification:** Tool presence and network availability are checked before mutation.

#### FM-30 — OCR is used on unsupported language or as a first-line parser

- **Classification:** Observed · Documents · `S=6` `O=4` `D=6` `RPN=144`
- **Effect:** Text corruption and incorrect conclusions.
- **TPS countermeasure:** Poka-yoke: native parsing/vision first; OCR only as bounded last resort.
- **Required verification:** OCR usage, language, pages, and confidence limitations are declared.

#### FM-25 — Lint runs without -D warnings or differs between developer and CI policy

- **Classification:** Observed · CI · `S=6` `O=6` `D=4` `RPN=144`
- **Effect:** Warnings accumulate or become surprise release blockers.
- **TPS countermeasure:** Standard work: one lint command and warning policy in Makefile/workflow/docs.
- **Required verification:** Exact command and clippy version are receipted.

#### FM-24 — Private Python execution exceeds its bounded runtime or a container command times out

- **Classification:** Observed · Execution · `S=7` `O=6` `D=3` `RPN=126`
- **Effect:** Partial computation and no final artifact.
- **TPS countermeasure:** Standard work: checkpoint, chunk, and use the correct execution plane.
- **Required verification:** Each phase produces an intermediate receipt and can resume idempotently.

#### FM-16 — A file update uses a stale blob SHA or concurrent writers update the same path

- **Classification:** Observed · GitHub API · `S=7` `O=6` `D=3` `RPN=126`
- **Effect:** 409 conflict, lost update, or partial branch state.
- **TPS countermeasure:** Kanban/WIP limit: one writer per path; fetch-current-SHA immediately before update.
- **Required verification:** Update response commit SHA descends from expected head.

#### FM-29 — Library/connector file reference is assumed to be a local sandbox path

- **Classification:** Observed · Files · `S=6` `O=5` `D=4` `RPN=120`
- **Effect:** Programmatic edit fails or an invalid download link is presented.
- **TPS countermeasure:** Standard work: materialize only when bytes are needed and verify exact path.
- **Required verification:** Filesystem existence check precedes every sandbox link.

#### FM-27 — Search or connector upstream returns transient 5xx and is treated as authoritative absence

- **Classification:** Observed · Connectors · `S=5` `O=6` `D=4` `RPN=120`
- **Effect:** Existing files or terminology are missed.
- **TPS countermeasure:** Andon: classify transport failure separately from zero results; bounded retry/fallback.
- **Required verification:** Response status and fallback source are recorded.

#### FM-34 — Mergeability is assumed stable after new commits or base movement

- **Classification:** Foreseeable · Git · `S=7` `O=4` `D=4` `RPN=112`
- **Effect:** Ready claim becomes invalid; conflicts appear at merge time.
- **TPS countermeasure:** Andon: refresh mergeability and base SHA immediately before readiness/merge.
- **Required verification:** Final PR snapshot is bound to head and base SHAs.

#### FM-33 — Large workflow fan-out queues many redundant jobs for documentation-sized changes

- **Classification:** Observed · Workflow · `S=5` `O=7` `D=3` `RPN=105`
- **Effect:** Long feedback loops, wasted capacity, stale results.
- **TPS countermeasure:** Heijunka + muda reduction: path filters, reusable gates, cancellation, and WIP limits.
- **Required verification:** Queue time and duplicated compute fall below target.

#### FM-35 — Files created as root are later consumed by a non-root process or mounted workspace

- **Classification:** Foreseeable · Ownership · `S=6` `O=4` `D=4` `RPN=96`
- **Effect:** Permission failures and non-reproducible local behavior.
- **TPS countermeasure:** Standard work: normalize ownership/modes in exported artifacts.
- **Required verification:** Artifact permissions are enumerated and tested under target UID.

#### FM-36 — A request requires sub-hour monitoring or event-triggered webhooks unavailable to the scheduler

- **Classification:** Observed · Automation · `S=4` `O=3` `D=2` `RPN=24`
- **Effect:** Delayed detection or false promise of continuous monitoring.
- **TPS countermeasure:** Visual control: declare scheduler granularity and unsupported triggers.
- **Required verification:** Automation definition shows supported cadence and timing mode.

---

## 7. Current stop-the-line findings

The following findings are active against the branch configuration observed during this census:

> **Direct observation:** while this document was being prepared, PR #10 moved from `948e872318c4f37d837f00d4bf42b4aa23ed35db` to `1800cd3f63873ed236d176f1f86787ec49a403f6` (`chore(kernel): synchronize capability module surface`) and then to `089b7d59e58092ce650a8f9a28cd3bb4ad07f669` (`fix(kernel): apply cargo-authored repairs`). This is concrete evidence for FM-02, FM-07, and FM-26: the subject moved twice during one audit and validation automation committed formatter/repair output to the branch.

1. **Validation mutates and pushes its own subject.**  
   The innovation workflow applies a repair script, formats files, stages all changes, commits, and pushes. Evidence generated earlier in the same job may no longer describe the final branch head.

2. **Workflow permission is broader than validation requires.**  
   `contents: write` is available to every step and third-party action in the job.

3. **Repository and workflow Rust channels diverge.**  
   The repository pins `nightly-2026-06-02`; the workflow explicitly requests `stable`.

4. **Acceptance is string-based rather than semantic.**  
   `grep` can confirm the presence of text without proving valid JSON, unique fields, coherent standing, or digest integrity.

5. **The local cloud guest cannot compile the Rust kernel.**  
   `rustc` and `cargo` are absent, and the guest has no DNS/HTTPS egress to install them. Exact Rust evidence must therefore come from a prepared runner or GitHub Actions.

6. **Resource discovery can overstate usable capacity.**  
   `free` reports 5.9 GiB while the cgroup enforces 4 GiB; `nproc` reports five CPUs while the cgroup quota is four.

7. **Connector transport errors are distinguishable from repository absence.**  
   GitHub code search returned upstream `502` responses during this work. The absence of search results was not accepted as evidence that TPS terminology was absent.

8. **The branch is stacked.**  
   PR #10 depends on `agent/ggen-alive-closure`; default-branch comparisons would misstate the change surface.

The document commit itself does not repair these findings. It makes them visible and establishes their required controls.

---

## 8. Recommended control plan

### P0 — required before declaring the innovation surface ALIVE

1. Make validation read-only:
   - set `permissions: contents: read`;
   - remove commit/push from the validation job;
   - fail when `git diff --exit-code` detects mutation.

2. Separate manufacture from proof:
   - run migration/formatting in a developer-controlled commit or explicit write-back workflow;
   - re-run validation on the resulting immutable head.

3. Align Rust:
   - allow `rust-toolchain.toml` to select `nightly-2026-06-02`;
   - capture `rustc -Vv`, `cargo -V`, installed targets, and component versions.

4. Replace `grep` gates:
   - parse output with `jq -e`;
   - validate against a committed schema;
   - reject duplicate keys, malformed JSON, missing digest subjects, or inconsistent standings.

5. Bind receipts:
   - include repository, commit SHA, tree SHA, base SHA, toolchain digest, environment-manifest digest, exact commands, exit codes, and artifact hashes;
   - verify the bundle independently before marking `ALIVE`.

### P1 — stabilize throughput and handoff

1. Commit a machine-readable environment manifest and regenerate it in CI.
2. Add resource preflight from cgroup files.
3. Add dependency/SBOM and lockfile checks.
4. Add secret scanning for logs, receipts, and artifacts.
5. Add workflow path filters and WIP limits to reduce redundant fan-out.
6. Test support-bundle reproduction in a clean runner.
7. Record pagination/completeness metadata for connector-backed audits.

### P2 — resilience and continuous improvement

1. Add cross-platform canonical-digest fixtures.
2. Add fault injection for DNS failure, 409 conflicts, OOM, timeout, truncation, and stale-head races.
3. Record incident A3s and update occurrence/detection ratings from measured data.
4. Trend queue time, reruns, false positives, evidence drift, and mean time to detection.
5. Re-run this FMEA after material changes to the model, tool schemas, base image, CI runner, permissions, or repository architecture.

---

## 9. Standard-work preflight

Run or implement the equivalent of this checklist before repository mutation:

```text
[ ] Resolve repository, base branch/SHA, head branch/SHA, and PR number.
[ ] Confirm authenticated principal and repository permission.
[ ] Inventory required binaries and versions in the exact execution plane.
[ ] Probe network capability in that same plane.
[ ] Read cgroup CPU and memory limits.
[ ] Confirm durable output target.
[ ] Fetch current blob SHA immediately before each file update.
[ ] Prohibit parallel writes to the same path.
[ ] Make validation jobs read-only.
[ ] Capture exact commands and exit codes.
[ ] Parse evidence semantically.
[ ] Bind receipts to commit, tree, environment, toolchain, and artifacts.
[ ] Re-fetch final PR head and mergeability before the final claim.
[ ] Use PARTIAL_ALIVE or REFUSED when any required proof is missing.
```

---

## 10. Machine-readable environment subject

The following normalized subject is suitable for inclusion in a future receipt:

```json
{
  "schema": "dteam.cloud-environment.v1",
  "observed_at_utc": "2026-08-02T20:16:50Z",
  "user_timezone": "America/Los_Angeles",
  "orchestrator": "GPT-5.6 Thinking",
  "guest": {
    "os": "Debian GNU/Linux 13.3 (trixie)",
    "kernel": "6.12.13",
    "architecture": "x86_64",
    "virtualization": "KVM",
    "uid": 0,
    "cpu_visible": 5,
    "cpu_quota_cores": 4,
    "memory_visible_bytes": 6367956992,
    "memory_limit_bytes": 4294967296,
    "swap_bytes": 0,
    "open_files_soft_limit": 1024,
    "stack_bytes": 8388608,
    "timezone": "UTC",
    "container_dns_egress": false
  },
  "repository": {
    "full_name": "seanchatmangpt/dteam",
    "pr": 10,
    "base_branch": "agent/ggen-alive-closure",
    "base_sha": "8354e411ca333df0dddc02a0eb4eadff4591c3a8",
    "head_branch": "agent/innovation-80-20-closure",
    "head_sha_before_document": "089b7d59e58092ce650a8f9a28cd3bb4ad07f669"
  },
  "local_missing": ["rustc", "cargo", "gh", "fd"],
  "security_boundary": {
    "secrets_included": false,
    "provider_host_inventory_claimed": false,
    "hidden_prompt_or_reasoning_included": false
  }
}
```

This JSON is descriptive, not yet cryptographically receipted. The document commit SHA becomes the first durable anchor; a later workflow should emit and verify the canonical manifest digest.

---

## 11. Ownership and review cadence

| Trigger | Required action |
|---|---|
| Every evidence-producing CI run | emit environment manifest and bind it to the receipt |
| Base image or runner change | repeat full census and compare |
| Tool/schema/connector change | review affected failure modes and detection controls |
| Permission change | repeat least-privilege audit |
| Security event or secret exposure | stop the line; rotate credentials; preserve sanitized evidence |
| Any false ALIVE claim | severity-10 incident; perform A3/5-Whys and strengthen the gate |
| Quarterly minimum | re-score occurrence and detection from observed incidents |

The FMEA is a living control surface. Rows may be closed only when the countermeasure is implemented and the required verification is observed against an immutable subject.
