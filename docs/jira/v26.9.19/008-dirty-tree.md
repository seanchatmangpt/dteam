# dteam: commit or clean uncommitted changes

- Standing: OPEN
- Created: 2026-09-19 (v26.9.19 gh survey wave)
- Source: working tree dirty at survey time
- Evidence: `git status --porcelain` → 21 path(s) (tracked-modified: 19, untracked: 2); sample:  D .gemini/agents/arthur_ter_hofstede.md; D .gemini/agents/carl_adam_petri.md; D .gemini/agents/dr_wil_van_der_aalst.md;?? Justfile;?? crates/insa/insa-kappa8/src/tests.rs;

## Work to complete
- Review the 19 tracked-modified path(s); commit them as atomic pieces on a purpose branch, or revert what is transient.
- For the 2 untracked path(s): add intentional files to git and commit; gitignore or delete build artifacts/temp files.
- Note: this survey's ticket files under docs/jira/v26.9.19/ are intentionally uncommitted; include or exclude them deliberately in the commit plan.

## Acceptance
- `git status --porcelain` is clean (except items deliberately deferred and recorded here).

## History
- 2026-09-19 | OPEN | survey found dirty tree | 21 paths (T19/U2) | commit/clean pending
