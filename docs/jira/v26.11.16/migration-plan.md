# dteam migration plan v26.11.16

Standing: UNVERIFIED

## Identity
- Path: /Users/sac/dteam
- Remote: https://github.com/seanchatmangpt/dteam.git
- Branch: main@5c00d75 (tracks origin/main)
- Last commit: 2026-08-23 | commits: 470 | size: 1086 MB
- Languages/build: Rust multi-crate workspace (Cargo.toml, crates/, apps/, capabilities/),
  cargo-make, Justfile, Makefile, Python research assets (.ruff_cache); CLAUDE.md present
- Nested repos: dteam/ostar_ref — NOT a real nested repo: it is a symlink to
  /Users/sac/chatmangpt/ostar (branch fix/ostar-round3-critical@28b47ab, remote
  seanchatmangpt/ostar, dirty 848, stash 372). Catalogue it under chatmangpt/ostar, not here.

## Observed state
- Dirty (19): Cargo.lock, Cargo.toml, src/lib.rs, crates/ccog/src/runtime/cog8.rs,
  crates/insa/insa-hotpath/src/{cog8,construct8}.rs, 8 files under crates/insa/insa-kappa8/src/,
  4 deleted .gemini/agents/*.md, deleted gitlink `dev-worktree` (no .gitmodules mapping)
- Untracked (11): Justfile, crates/insa/insa-kappa8/src/tests.rs, docs/jira/
- ahead 0 / behind 0 on main; stash 0; unpushed local branches: 158 (census). Seen with
  upstream drift: dev (ahead 59 of origin/dev), fix/dteam-critical (ahead 7),
  fix/dteam-round3 (ahead 1), fix/core-completions & fix/test-coverage (no upstream)
- Large dirs: data/ 723 MB (DomesticDeclarations.xes, open-ontologies, pdc2025),
  artifacts/ 207 MB; target/ absent; .cache 0 B
- Misc root files: dhat-heap.json, ANTI_FAKE_EVIDENCE.log, many *.md reports
- Submodules: broken gitlink `dev-worktree` (deleted, unmapped)

## What it is
dteam is a deterministic process-intelligence and capability-execution platform: a multi-crate
workspace for compiled cognition, process mining, conformance and autonomic execution, plus a
standalone dependency-free capability kernel under capabilities/dteam-kernel.

## Classification
WARM — census value stands; last commit Aug 2026, but live uncommitted kappa8/insa edits.

## Migration risk
HIGH — 158 unpushed branches (dev alone is 59 ahead of origin/dev), 19 dirty files of insa/ccog
engine work, an untracked test module, and ~930 MB of data/ + artifacts/ that are partly
untracked research inputs. The dangling ostar_ref symlink will break on copy.

## Recommended action
COMMIT-PUSH
1. `cd /Users/sac/dteam && git add -A && git commit -m "wip: kappa8/insa engine edits + jira"`
2. `git push origin main`
3. `git push origin dev fix/dteam-critical fix/dteam-round3` — land the branches with upstream
   drift first. Then `git push --all origin`. DECISION: triage the ~150 other local branches
   (many wreckit/* and feat/*) — push all, or delete merged ones with
   `git branch --merged main`.
4. `git rm --cached dev-worktree` in the same commit to clear the unmapped gitlink.
5. DECISION: data/ (723 MB) and artifacts/ (207 MB) — determine which are tracked
   (`git ls-files data artifacts | wc -l`); untracked ones go RSYNC-COPY to the archive volume,
   not into the repo.
6. Remove the symlink `ostar_ref` before any rsync; recreate on the new machine if needed.
7. On new machine: `git clone https://github.com/seanchatmangpt/dteam.git`.

## First commands on the new machine
- `cargo make check` then `cargo make test` (stated, not run; per CLAUDE.md never bare cargo)

## See Also
- ../v26.9.19/
- /Users/sac/docs/jira/v26.11.16/README.md
