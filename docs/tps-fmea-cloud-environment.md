# Cloud Execution Environment Inventory and TPS/FMEA Catalog

**Repository:** `seanchatmangpt/dteam`  
**Document purpose:** Standard-work inventory, Toyota Production System failure-mode catalog, and FMEA control plan for AI-assisted repository manufacture.  
**Observation timestamp:** `2026-08-02T20:10:39Z`  
**Observation scope:** The live execution container and connected GitHub control plane available during this session.  
**Standing:** Environment facts below are **observed** unless marked **inferred** or **unknown**.

> This document records every operational detail that was observable and relevant to reproducibility. It does **not** disclose or guess credentials, secret values, tenant identifiers, private service endpoints, provider-internal topology, or hidden platform configuration.

## 1. TPS framing

The operating model is:

```text
admitted request
→ exact repository/base/branch resolution
→ local observation
→ bounded implementation
→ local execution
→ evidence and receipt
→ Git object publication
→ pull-request review
```

TPS controls:

- **Jidoka:** stop the line on an invariant failure; never convert failure into success.
- **Andon:** emit a typed failure with the earliest repair boundary.
- **Poka-yoke:** make wrong repository, stale SHA, missing dependency, and false standing mechanically difficult.
- **Genchi genbutsu:** inspect the actual repository and actual runtime rather than reasoning from summaries.
- **Standard work:** use repeatable commands, fixed standing vocabulary, exact identities, and replay.
- **Hansei/kaizen:** preserve observed failures as fixtures and update this FMEA after each material incident.
- **Muda:** avoid duplicate generation, repeated manual inspection, and contaminated pull requests.
- **Mura:** reduce environment/version variance through pins and receipts.
- **Muri:** avoid overloading a 5-vCPU/5.9-GiB ephemeral worker with unbounded builds.

## 2. Boundary and disclosure policy

### 2.1 Observable and recorded

- Guest operating system, kernel, architecture, CPU allocation, memory, filesystem capacity, and process limits.
- Installed executable versions and explicitly absent toolchains.
- Sanitized filesystem mounts and permissions.
- Direct network-resolution behavior.
- The distinction between local execution and the connected GitHub API control plane.
- Repository identities, commit identities, and user-requested source changes.
- Failure modes observed during this work and credible failure modes implied by the observed architecture.

### 2.2 Intentionally excluded

- Environment-variable values, passwords, tokens, cookies, certificates, proxy URLs, registry credentials, and authentication headers.
- Internal hostnames, container/VM identifiers, tenant IDs, private IP addresses, VPC/subnet data, or service ports not required for the public operating contract.
- Cloud-provider region, availability zone, physical host identity, storage backend, backup policy, or image-build pipeline because these were not observable from the guest.
- Hidden prompts, private reasoning, or platform-internal control logic.

### 2.3 Unknown and not to be inferred

| Unknown | Why it remains unknown |
|---|---|
| Cloud provider and region | No authoritative guest-visible source was available. |
| Physical host and noisy-neighbor allocation | KVM is visible; host scheduling and tenancy are not. |
| Container lifetime and snapshot policy | The session exposes an ephemeral working filesystem but no durability SLA. |
| Base-image SBOM and signature | Versions are observable; signed provenance is not. |
| Network firewall/proxy routing | Credentialed proxy variables exist, but values and infrastructure are intentionally not inspected or recorded. |
| Backup/restore and disaster recovery | No platform-level contract is exposed to the guest. |
| Bandwidth, latency, and availability SLOs | One DNS/HTTPS probe is evidence of current behavior, not an SLO. |

## 3. Live environment inventory

### 3.1 Compute and isolation

| Property | Observed value |
|---|---|
| Guest OS | Debian GNU/Linux 13.3 (`trixie`) |
| Kernel | Linux `6.12.13`, x86_64 |
| Virtualization | KVM full virtualization; AMD-V visible |
| CPU allocation | 5 online vCPUs |
| CPU model | AMD EPYC 9V74 |
| Byte order | Little endian |
| CPU features | Includes SSE4, AVX, AVX2, AVX-512 families, AES, and SHA-NI; do not assume these exist on deployment targets |
| Memory | 5.9 GiB total; approximately 5.1 GiB available at observation |
| Swap | 0 bytes |
| Root filesystem | 63 GiB total; 39 GiB available at observation |
| Cgroup | cgroup v2 visible; control filesystem mounted read-only |
| Execution identity | UID 0 (`root`), GID 0 |
| Default umask | `0022` |
| Open-file limit | 1024 |
| Stack limit | 8192 KiB |
| Locked-memory limit | 8192 KiB |
| Process limit | 7851 |
| Core dumps | Disabled (`0`) |

### 3.2 Filesystem and persistence surfaces

| Path/surface | Observed contract |
|---|---|
| `/` | Writable ext4 guest filesystem |
| `/home/oai` | Owner `oai`, group `oai_shared`, mode `2710` |
| `/mnt/data` | Owner `oai`, group `oai_shared`, setgid mode `2775`; artifact handoff surface |
| `/tmp` | World-writable sticky directory, mode `1777` |
| `/caas_toolbox` | Read-only `virtiofs` tooling mount |
| `/sys` and cgroup controls | Read-only to the guest |
| Durability | No platform durability guarantee is visible; Git commits and explicitly handed-off artifacts are the reliable externalization mechanisms |

### 3.3 Installed runtimes and build tools

| Surface | Version/status |
|---|---|
| Python / CPython | 3.13.5, virtual environment at `/opt/pyvenv` |
| Node.js | 22.16.0 |
| npm | 10.9.2 |
| Git | 2.47.3 |
| Bash | 5.2.37 |
| GNU Make | 4.4.1 |
| GCC / G++ | 14.2.0 |
| Clang / Clang++ | 17.0.0, supplied from the Swift LLVM installation |
| Go | 1.23.2 |
| OpenJDK / `javac` | 21.0.10 |
| Kotlin compiler | 1.9.0 on JRE 21 |
| Swift | 6.2.1 |
| Ruby | 3.3.8 |
| Perl | 5.40.1 |
| PHP | 8.4.16 |
| Chromium | 144.0.7559.96 |
| jq | 1.7 |
| CMake | 3.31.6 |
| Ninja | 1.12.1 |
| APT | 3.0.3 |
| Rust `rustc` / Cargo | **Absent** |
| Wasmtime / Wasmer | **Absent** |
| Docker / Podman | **Absent** |
| Google Chrome / `chromium-browser` alias | **Absent** |

### 3.4 Network and control planes

| Plane | Observed behavior |
|---|---|
| Local shell network | Resolver configuration exists, but resolving/connecting to `github.com` timed out during a bounded five-second HTTPS probe. Direct clone/download/install cannot be assumed. |
| Proxy/registry environment | Proxy and package-registry configuration is injected. Credential-bearing values are not recorded. Presence does not prove reachability. |
| GitHub connector | Separate authenticated control plane can fetch Git objects, create branches/blobs/trees/commits/files, update refs, and open/update PRs. |
| Connector reliability | Search produced transient HTTP 502 failures; large responses were truncated; exact object APIs remained usable. |
| Web/browser | Chromium is installed locally. Browser execution must record executable path, version, sandbox flags, and whether the test used a real page or a synthetic contract. |
| Local/remote relationship | GitHub connector success does not imply local network success. Local execution and remote repository mutation are separate evidence domains. |

## 4. Standing model

| Standing | Meaning |
|---|---|
| `UNKNOWN` | Required observation has not been made. |
| `BLOCKED_TRANSPORT` | Required bytes cannot be reached through the admitted transport. |
| `BLOCKED_DEPENDENCY` | Source controls pass, but a required toolchain or exact dependency is absent. |
| `BUILD_BROKEN` | Source, build, test, mutation, or replay invariant failed. |
| `PARTIAL_ALIVE` | Some bounded layers executed successfully; required crown layers remain blocked or unknown. |
| `ALIVE` | The exact admitted subject executed, passed its required negative controls, emitted a receipt, and replayed without drift. |

A remote commit, queued workflow, generated file, or passing source scan is never sufficient by itself for `ALIVE`.

## 5. FMEA scoring method

Scores use a 1–10 scale:

- **Severity (S):** 1 = cosmetic; 4 = local rework; 7 = material integrity/availability loss; 10 = false actuation, credential exposure, or systemic corruption.
- **Occurrence (O):** 1 = exceptional; 4 = occasional; 7 = recurrent under current architecture; 10 = effectively certain.
- **Detection (D):** 1 = automatic/immediate detection; 4 = visible in normal logs; 7 = subtle/manual detection; 10 = likely latent.
- **RPN:** `S × O × D`.
- **Priority:** Critical when severity is at least 9 or RPN is at least 300; High at 160–299; Medium at 80–159; Low below 80.

FMEA scores are current engineering judgments, not statistical probabilities. Re-score after controls are implemented and observed.

## 6. Highest-priority Andon board

| Rank | ID | Failure mode | S | O | D | RPN | Priority |
|---:|---|---|---:|---:|---:|---:|---|
| 1 | FM-002 | Diagnostic command fails open or suppresses stderr | 9 | 6 | 8 | 432 | Critical |
| 2 | FM-001 | Authored or queued work is promoted as `ALIVE` | 10 | 5 | 8 | 400 | Critical |
| 3 | FM-035 | Running as root hides permission and ownership defects | 7 | 7 | 8 | 392 | Critical |
| 4 | FM-003 | Agent stops at first blocked transport or absent tool | 8 | 6 | 8 | 384 | Critical |
| 5 | FM-041 | Source-only checks are mistaken for compiled/runtime behavior | 9 | 5 | 8 | 360 | Critical |
| 6 | FM-046 | Missing config silently falls back to defaults | 9 | 5 | 8 | 360 | Critical |
| 7 | FM-056 | Failure is classified by persuasive narrative instead of exact witness | 9 | 5 | 8 | 360 | Critical |
| 8 | FM-014 | Local container and GitHub connector have different network/access capabilities | 8 | 6 | 7 | 336 | Critical |
| 9 | FM-050 | Preinstalled toolchains lack SBOM/provenance receipt | 7 | 6 | 8 | 336 | Critical |
| 10 | FM-042 | Polyglot validators compare only a common digest | 8 | 5 | 8 | 320 | Critical |
| 11 | FM-055 | Tool or subprocess stderr is truncated or discarded | 7 | 6 | 7 | 294 | High |
| 12 | FM-033 | Virtualized CPU/timing variance invalidates microbenchmarks | 6 | 7 | 7 | 294 | High |
| 13 | FM-004 | Hosted CI is substituted for an explicit local-only requirement | 9 | 4 | 8 | 288 | Critical |
| 14 | FM-005 | A thin demo replaces the existing manufacturing architecture | 9 | 4 | 8 | 288 | Critical |
| 15 | FM-040 | Generated evidence is committed without source correspondence | 9 | 4 | 8 | 288 | Critical |

## 7. Full TPS/FMEA catalog

| ID | Process | Failure mode | Effect | Cause | Current control/detection | S | O | D | RPN | Priority | TPS countermeasure |
|---|---|---|---|---|---|---:|---:|---:|---:|---|---|
| FM-001 | Standing | Authored or queued work is promoted as `ALIVE` | False confidence; downstream adoption of unexecuted behavior | Status conflates existence, publication, execution, and replay | Standing vocabulary and explicit receipts | 10 | 5 | 8 | 400 | Critical | Jidoka stop: require exact execution witness and independent replay before `ALIVE`. |
| FM-002 | Diagnostics | Diagnostic command fails open or suppresses stderr | Defects are rewritten as success; repair starts from false state | Shell fallbacks such as `\|\| echo` and discarded stderr | PR #11 changes `make doctor` to propagate failure | 9 | 6 | 8 | 432 | Critical | Poka-yoke: typed exit codes `ALIVE`, `BLOCKED_DEPENDENCY`, `BUILD_BROKEN`; never rewrite failure. |
| FM-003 | Work method | Agent stops at first blocked transport or absent tool | Premature partial delivery; missed alternate paths | Single-path planning and no exhaustion ledger | Explicit user correction and multi-path attempts | 8 | 6 | 8 | 384 | Critical | Standard work: enumerate local, connector, archive, package, and language alternatives before declaring blocked. |
| FM-004 | Work method | Hosted CI is substituted for an explicit local-only requirement | Evidence violates the requested execution boundary | Convenience substitution after local network/toolchain failure | Later local-only crowns and boundary statements | 9 | 4 | 8 | 288 | Critical | Andon: reject evidence whose execution venue differs from the admitted venue. |
| FM-005 | Architecture | A thin demo replaces the existing manufacturing architecture | Capability density collapses; Chesterton's fence is violated | Implementation begins before repository doctrine and generators are recovered | Later preservation of ontology→generation→receipt chain | 9 | 4 | 8 | 288 | Critical | Genchi genbutsu: inspect doctrine, manifests, generators, and receiving contracts before writing. |
| FM-006 | Scope | Wrong repository, branch, or base is modified | Unrelated history, merge risk, or lost work | Ambiguous scope or stale branch assumptions | Exact base/head receipts and compare-commits | 8 | 4 | 7 | 224 | High | Resolve repo/base/head, assert merge base, and refuse writes until exact identity is recorded. |
| FM-007 | Git history | Existing branch name contains unrelated commits | Contaminated PR; impossible review and unsafe merge | Branch existence was not inspected before reuse | Contaminated PR #9 closed; clean PR #11 created | 8 | 4 | 8 | 256 | High | Create a unique branch from exact base; compare before PR; reject unexpected file/commit count. |
| FM-008 | Git writes | Multi-file changes are published as many sequential partial commits | Intermediate branch states are inconsistent; review noise | Contents API writes commit per file | Tree/commit API used for clean atomic assembly where possible | 6 | 7 | 5 | 210 | High | Prefer one tree plus one commit; otherwise use staging and final exact-tree commit. |
| FM-009 | Git writes | Concurrent or stale-SHA updates race on the same file | Lost update or rejected write | Sequential-update contract not preserved | Fetch blob SHA before update | 8 | 3 | 6 | 144 | Medium | Serialize same-path writes and verify resulting blob SHA. |
| FM-010 | Pull request | PR mergeability is initially unknown/false | Premature blockage or incorrect merge claim | GitHub computes mergeability asynchronously | Re-fetch PR metadata | 5 | 5 | 4 | 100 | Medium | Treat initial mergeability as `UNKNOWN`; poll after head stabilizes. |
| FM-011 | Connector | GitHub code search returns transient 502 | Audit gaps or false absence | Upstream connector/search service failure | Fallback to `fetch_file`, commit diff, and known paths | 5 | 5 | 2 | 50 | Low | Andon plus fallback matrix; never infer absence from one failed search. |
| FM-012 | Connector | Connector response is truncated | Critical code or metadata omitted from review | Large blobs/diffs exceed response budget | Range fetches, blob fetches, resource search | 7 | 7 | 5 | 245 | High | Fetch bounded ranges and verify terminal markers/counts. |
| FM-013 | Connector | Search index misses newly written or niche files | False negative during audit | Index lag or incomplete coverage | Direct path fetch and PR changed-file listing | 7 | 5 | 7 | 245 | High | Use search only for discovery; exact object APIs grant admission. |
| FM-014 | Control planes | Local container and GitHub connector have different network/access capabilities | Split-brain evidence; local cannot reproduce connector-visible state | Separate authenticated remote control plane and isolated local data plane | Explicit local-vs-remote standing | 8 | 6 | 7 | 336 | Critical | Model planes separately; receipt records where each observation/action occurred. |
| FM-015 | Network | Direct DNS/egress to GitHub times out | Clone, rustup, apt, and external downloads fail | Resolver/proxy/egress unavailable in the local container | Five-second DNS/curl probe; connector fallback | 6 | 9 | 2 | 108 | Medium | Classify `BLOCKED_TRANSPORT`; use pre-mounted files or connector object materialization. |
| FM-016 | Network | Proxy or registry configuration exists but is opaque or stale | Package installation behaves inconsistently | Credentialed proxy/registry environment is injected and not operator-controlled | Values intentionally excluded from logs | 6 | 8 | 6 | 288 | High | Doctor tests sanitized reachability per registry; never print credential-bearing values. |
| FM-017 | Toolchain | Rust compiler and Cargo are absent | Native workspace and Rust/WASM tests cannot run | Base image lacks Rust; direct installation transport blocked | Doctor reports `BLOCKED_DEPENDENCY` | 8 | 8 | 2 | 128 | Medium | Pre-bake pinned Rust or mount an offline toolchain bundle with checksum. |
| FM-018 | Dependencies | Sibling path dependencies are missing | Cargo metadata/build cannot resolve workspace | Repository expects adjacent `unibit`/`wasm4pm` paths | Path-dependency doctor inventory | 8 | 7 | 3 | 168 | High | Materialize exact sibling commits and bind them in a workspace receipt. |
| FM-019 | Toolchain | Floating nightly changes compiler behavior | Build drift and non-reproducible diagnostics | `channel = "nightly"` without date | PR #11 pins `nightly-2026-06-02` | 8 | 5 | 7 | 280 | High | Pin compiler/date/components/targets and receipt `rustc -Vv`. |
| FM-020 | Dependencies | External registries or package indexes are unavailable | Build cannot hydrate; hidden cache dependence | Direct egress blocked or cache not populated | Local source-only audit remains dependency-free | 8 | 6 | 6 | 288 | High | Vendor/lock critical dependencies; provide offline cache manifest and checksums. |
| FM-021 | Compatibility | Python 3.13 breaks packages expecting older CPython | Tool installation/import failures | Environment is ahead of ecosystem support windows | Dependency-free standard-library audit | 6 | 5 | 6 | 180 | High | Pin supported Python or test against 3.13 explicitly. |
| FM-022 | Compatibility | Node 22 or npm 10 changes dependency/runtime behavior | Build or browser harness drift | Major runtime differs from project assumptions | Exact version inventory | 6 | 4 | 6 | 144 | Medium | Declare engines and lockfile format; execute `npm ci` under admitted version. |
| FM-023 | Compatibility | Cross-language integer/hash semantics diverge | False equivalence or inconsistent receipts | Unsigned overflow and integer-width differences | Observed FNV failure in Perl/PHP; moved to SHA-256 | 7 | 4 | 6 | 168 | High | Use standardized digests and official vectors in every runtime. |
| FM-024 | Compatibility | Clang is supplied by the Swift toolchain | C/C++/WASM behavior differs from Debian GCC expectations | `clang` originates under `/usr/local/swift` | Version/path inventory | 6 | 4 | 7 | 168 | High | Receipt compiler path, target triple, and version; test GCC and Clang separately. |
| FM-025 | WASM | No Wasmtime/Wasmer runtime is installed | Native WASI execution unavailable | Base image omits standalone WASM engines | Clang WASM executed through Node in prior capsule | 8 | 5 | 7 | 280 | High | Classify Node-WebAssembly separately; attach a pinned standalone engine for WASI claims. |
| FM-026 | Browser | Only Chromium is available; Chrome aliases absent | Browser-specific tests may be skipped or misrouted | Base image provides Debian Chromium only | Exact executable/version inventory | 6 | 5 | 5 | 150 | Medium | Browser doctor resolves executable explicitly and records version. |
| FM-027 | Browser | Running browser as root changes sandbox behavior | Tests pass under unsafe flags but fail in production | Container user is root | Explicit browser launch contract | 7 | 4 | 6 | 168 | High | Run Chromium as non-root; record sandbox flags and fail on implicit fallback. |
| FM-028 | Containers | Docker and Podman are absent | Cannot reproduce nested-container workflows | Nested engine not installed/allowed | Direct local process execution | 4 | 8 | 1 | 32 | Low | Do not design acceptance around nested containers; use process/WASM isolation. |
| FM-029 | Resources | No swap is configured | Memory spikes terminate processes abruptly | 5.9 GiB RAM with zero swap | Memory inventory and staged commands | 7 | 4 | 5 | 140 | Medium | Set memory budgets; stream artifacts; monitor RSS and stop before OOM. |
| FM-030 | Resources | Open-file limit is 1024 | Large test matrices fail with EMFILE | Container ulimit is bounded | Ulimit inventory | 6 | 4 | 6 | 144 | Medium | Batch file operations, close descriptors, and add doctor threshold. |
| FM-031 | Resources | Root filesystem fills with builds/artifacts | Writes fail or corrupt partial outputs | 63 GiB volume is finite | Disk inventory; clean build directories | 7 | 3 | 4 | 84 | Medium | Preflight free-space threshold and clean-on-failure standard work. |
| FM-032 | Resources | Long-running commands exceed tool/session timeout | Partial output and ambiguous standing | Execution tools are bounded and session-scoped | Staged commands and explicit timeout handling | 6 | 5 | 5 | 150 | Medium | Split DAG into receipted stages and classify timeout distinctly. |
| FM-033 | Resources | Virtualized CPU/timing variance invalidates microbenchmarks | Performance claims are not portable | KVM guest, shared scheduling, frequency variance | CPU/hypervisor inventory | 6 | 7 | 7 | 294 | High | Separate functional from performance standing; report distributions. |
| FM-034 | Resources | Host CPU features exceed target deployment features | Generated binaries fail on older hardware | EPYC exposes AVX2/AVX-512 and other extensions | Target/version inventory | 7 | 5 | 7 | 245 | High | Compile to declared baseline and test portable/optimized profiles. |
| FM-035 | Permissions | Running as root hides permission and ownership defects | Artifacts fail when consumed by non-root users | UID 0 has broad write authority | Ownership/mode inventory | 7 | 7 | 8 | 392 | Critical | Add non-root replay; normalize ownership and modes before publication. |
| FM-036 | Persistence | Local container state is ephemeral or session-bound | Uncommitted code/evidence disappears | No durability contract for local filesystem | Git commits and exported artifacts | 8 | 4 | 5 | 160 | High | Commit source, publish receipt, and never cite unconfirmed paths. |
| FM-037 | Artifacts | A sandbox/download link is invented from a filename | User receives a broken or wrong artifact | Filename does not prove mounted path | Path existence confirmation before linking | 7 | 4 | 7 | 196 | High | Link only after container stat/materialization confirms the path. |
| FM-038 | Evidence | Timestamps, latency, or unordered maps enter receipt identity | Replay drift despite equivalent behavior | Nondeterministic fields are hashed | Canonical JSON and exclusion rules | 8 | 5 | 7 | 280 | High | Separate identity fields from observations; sort and normalize. |
| FM-039 | Evidence | Receipt includes itself | Impossible fixed point or changing root | Receipt directory is not excluded | Explicit receipt exclusion in later capsules | 8 | 3 | 8 | 192 | High | Define receipt subject before emission and test self-reference mutant. |
| FM-040 | Evidence | Generated evidence is committed without source correspondence | Stale or fabricated proof survives code change | No source hash or exact head in evidence | Exact-head and artifact-tree hashes | 9 | 4 | 8 | 288 | Critical | Bind source head, tool versions, commands, outputs, and standing. |
| FM-041 | Evidence | Source-only checks are mistaken for compiled/runtime behavior | Native defects remain hidden behind green audit | Standing scopes are collapsed | PR #11 separates source `ALIVE` from Rust blockage | 9 | 5 | 8 | 360 | Critical | Use orthogonal standing dimensions; prohibit aggregate `ALIVE` with unexecuted required layers. |
| FM-042 | Evidence | Polyglot validators compare only a common digest | Semantic bugs survive identical input hashing | Validators prove subject identity, not behavior | Later scenario and mutation checks | 8 | 5 | 8 | 320 | Critical | Require language-specific behavior assertions and cross-runtime oracle fixtures. |
| FM-043 | Testing | Mutation scanner matches its own test vocabulary | Valid system is rejected or mutant falsely killed | Naive scan lacks subject boundaries | Observed and repaired in checkpoint work | 6 | 4 | 7 | 168 | High | Scan admitted production paths only; prove mutation changes semantics. |
| FM-044 | Testing | Tests are coupled to source strings/symbol names | Refactor breaks audit; behavior bug may pass | Static grep substitutes for execution | PR #11 source audit is explicitly scoped | 7 | 5 | 8 | 280 | High | Move high-severity controls into compiled black-box tests when Rust is available. |
| FM-045 | Testing | Capability count passes while combinations are not exercised | Combinatorial gaps remain latent | Cardinality is checked instead of behavior | Exhaustive 8,640-profile enumeration in prior capsule | 8 | 4 | 8 | 256 | High | Execute each profile or use a proven covering array with declared residual risk. |
| FM-046 | Configuration | Missing config silently falls back to defaults | Unadmitted policy executes | Optional load has no provenance | `ConfigSource`, strict load, and validation in PR #11 | 9 | 5 | 8 | 360 | Critical | Strict load for actuation; defaults only in admitted development mode. |
| FM-047 | Configuration | Fields are individually valid but relationally incoherent | Unsafe or meaningless policy composition | No cross-field invariants | Reward normalization and relational validation added | 8 | 5 | 7 | 280 | High | Encode relational invariants and negative fixtures; receipt config hash. |
| FM-048 | Security | Secret values leak through inventory or logs | Credential compromise and lateral access | Broad `env`, debug logs, or error dumps | This document excludes values and identifiers | 10 | 3 | 8 | 240 | Critical | Allowlist safe fields; redact by default; never commit environment dumps. |
| FM-049 | Security | Provider-internal topology is guessed or over-disclosed | Security exposure and false operating model | Pressure to document beyond observable boundary | Observed/inferred/unknown classification | 10 | 2 | 9 | 180 | Critical | Document only facts; mark unknowns; omit credentials and internal topology. |
| FM-050 | Supply chain | Preinstalled toolchains lack SBOM/provenance receipt | Compromised or unexpected binaries are trusted | Base-image lineage is not guest-visible | Version/path inventory only | 7 | 6 | 8 | 336 | Critical | Generate SBOM, hash executables, and obtain signed platform provenance. |
| FM-051 | Lifecycle | Environment inventory becomes stale in the next session | Future decisions use obsolete versions/capacity | Ephemeral images can change | Observation timestamp and exact versions | 6 | 8 | 6 | 288 | High | Regenerate inventory at session start and diff against baseline. |
| FM-052 | Time | UTC/local-time confusion changes schedules or evidence | Wrong deadlines, recency, or event ordering | Container UTC differs from operator timezone | UTC timestamp plus explicit local zone | 5 | 4 | 7 | 140 | Medium | Store UTC in evidence and render local time separately. |
| FM-053 | Filesystem | Read-only cgroup/sysfs prevents tuning or introspection | Controls cannot be changed; diagnostics may be incomplete | Sandbox isolation mounts controls read-only | Mount inventory | 4 | 6 | 3 | 72 | Low | Treat tuning as unavailable; avoid tests requiring host mutation. |
| FM-054 | Filesystem | Group-writable artifact paths produce ownership/mode drift | Artifacts are unreadable or unexpectedly mutable | `/mnt/data` is setgid and shared-group writable | Mode/owner inventory | 6 | 5 | 6 | 180 | High | Normalize permissions, hash after copy, and test as target user. |
| FM-055 | Observability | Tool or subprocess stderr is truncated or discarded | Root cause is lost; repeated repair cycles | Pipes, redirection, and output limits | Fail-loud Makefile change and bounded log capture | 7 | 6 | 7 | 294 | High | Preserve stderr, exit code, tail, and full artifact log. |
| FM-056 | Observability | Failure is classified by persuasive narrative instead of exact witness | Wrong repair and status inflation | Summary is not bound to machine evidence | Machine-readable receipts and exact IDs | 9 | 5 | 8 | 360 | Critical | Narrative cites receipt fields; unsupported claims are `UNKNOWN`. |

## 8. Standard work: preflight, execution, and publication

### 8.1 Preflight

1. Record repository, exact base SHA, intended branch, and expected changed paths.
2. Verify branch nonexistence or inspect its merge base before reuse.
3. Inventory required toolchains, path dependencies, network transport, disk, memory, file descriptors, and browser executable.
4. Classify each requirement as `PRESENT`, `ABSENT`, `BLOCKED_TRANSPORT`, or `UNKNOWN`.
5. Refuse to silently substitute CI, a connector, a mock, Node-WebAssembly, or a source scan for a required local/native execution layer.

### 8.2 Execution

1. Materialize the exact subject locally.
2. Clean prior outputs.
3. Run the smallest end-to-end tracer bullet.
4. Run positive, negative, mutation, and replay controls.
5. Preserve stdout, stderr, exit code, command line, tool versions, source hashes, and output hashes.
6. Stop on the earliest invariant failure; repair the producing boundary rather than masking it.
7. Re-run from a clean state.

### 8.3 Publication

1. Assemble an atomic Git tree when multiple files form one logical change.
2. Verify the branch merge base and compare expected changed paths/counts.
3. Open a draft PR with exact base/head and standing dimensions.
4. Do not commit binaries, credentials, transient caches, or unbound evidence.
5. Bind committed evidence to exact source and toolchain identities.
6. Re-fetch PR metadata after GitHub computes mergeability.

## 9. Andon response taxonomy

| Code | Stop condition | Required response |
|---|---|---|
| `WRONG_SUBJECT` | Repo/base/branch/path differs from admitted scope | Stop all writes; recover exact subject. |
| `CONTAMINATED_HISTORY` | Unexpected merge base, commit count, or changed path | Close/refuse PR; recreate from exact base. |
| `BLOCKED_TRANSPORT` | Required bytes cannot be reached | Record attempted transports; use admitted alternate materialization or stop. |
| `BLOCKED_DEPENDENCY` | Required tool/runtime/path dependency absent | Install from verified offline source or downgrade standing. |
| `FAIL_OPEN` | Error is swallowed, rewritten, or converted to success | Replace fallback with typed failure and negative fixture. |
| `EVIDENCE_DRIFT` | Two clean runs produce different identity evidence | Remove nondeterministic fields; do not publish `ALIVE`. |
| `FALSE_STANDING` | Claim exceeds executed evidence | Retract claim; split standing by layer. |
| `SECRET_EXPOSURE` | Credential-bearing value enters logs, source, or artifact | Stop, redact, rotate credential, and audit access. |
| `RESOURCE_MURI` | Memory/disk/fd/time budget approaches limit | Stop new work, checkpoint, clean, and reduce batch size. |
| `CONNECTOR_DEGRADED` | Search/API returns 5xx, truncation, or stale data | Use exact object APIs and verify counts/hashes. |

## 10. Control plan and kaizen backlog

### Immediate controls

1. Keep PR #11's fail-loud doctor, strict configuration loading, validation, pinned nightly, source receipt, and replay.
2. Add this FMEA to the repository's operational standard work.
3. Add a non-root replay fixture.
4. Add exact environment inventory generation with an allowlist and automatic redaction.
5. Add branch-contamination preflight using merge-base and changed-path assertions.
6. Add compiler/runtime path and version fields to every execution receipt.

### Next controls

1. Provide an offline, checksum-pinned Rust toolchain and exact sibling repositories.
2. Add a standalone WASM engine and distinguish WASI from browser/Node WebAssembly.
3. Add SBOM and signed provenance for the base image and preinstalled compilers.
4. Replace source-string controls with compiled black-box controls once Rust is available.
5. Add memory, disk, file-descriptor, and timeout budgets to the doctor.
6. Execute browser tests as a non-root user with explicit sandbox configuration.

### Residual risk

Even after these controls, the guest cannot prove the cloud provider's physical isolation, host patch status, backup policy, image signing pipeline, or network SLO. Those require platform-owner attestations outside the repository.

## 11. Review cadence

- Re-run the inventory at the start of every new execution session.
- Re-score any failure mode after a material incident or control change.
- Add every newly observed failure as a regression fixture.
- Review Critical and High RPN items before promoting a PR from draft.
- Require an owner, due date, and evidence link for any accepted Critical residual risk.

## 12. Current document receipt

```text
observation_time_utc = 2026-08-02T20:10:39Z
environment_scope    = live ephemeral Linux guest + connected GitHub control plane
failure_modes        = 56
secret_values        = excluded
provider_topology    = unknown
```

This document is a point-in-time operational baseline. It must be regenerated or explicitly re-admitted when the execution image, tool versions, repository base, or connector capabilities change.
