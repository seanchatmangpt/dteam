# Repository-wide Markdown closure

This document defines the acceptance contract for the documentation migration stacked on `agent/ggen-alive-closure`.

## Objective

Every tracked `*.md` file must receive one explicit disposition:

- **Canonical** — rewritten as current authoritative documentation.
- **Reference** — retained and normalized because it documents a stable interface, attribution, policy, or research result.
- **Archived** — moved beneath `docs/archive/` with provenance and a pointer to the current replacement.
- **Superseded** — reduced to a short redirect when the original path must remain stable.
- **Removed** — deleted only when duplicate, generated, or content-free and when no stable link requires preservation.

No Markdown file may remain unclassified.

## Required deliverables

1. A machine-readable inventory of every Markdown path, digest, disposition, owner document, and replacement path.
2. A canonical documentation map rooted at `README.md`.
3. An archive index explaining why each historical document is retained.
4. Link validation with zero broken repository-relative Markdown links.
5. Duplicate-doctrine detection so one concept has one authoritative definition.
6. Evidence-bounded standing: current claims must identify their executed subject and historical claims must be labeled as historical.
7. A final receipt proving that the inventory and filesystem agree exactly.

## Editorial rules

- Define terms before using them.
- Separate current behavior, intended behavior, and historical behavior.
- Prefer executable commands over prose-only procedures.
- Do not claim `ALIVE` without an exact observed execution receipt.
- Preserve attribution and licensing text without semantic alteration.
- Keep architecture, operations, validation, contribution policy, and research history in separate documents.
- Archive failed experiments and superseded designs rather than silently rewriting history.

## Proposed canonical structure

```text
README.md
AGENTS.md
CONTRIBUTING.md
PHILOSOPHY.md
ATTRIBUTION.md
docs/
  ARCHITECTURE.md
  OPERATIONS.md
  VALIDATION.md
  RESEARCH.md
  GLOSSARY.md
  archive/
    README.md
    legacy/
    experiments/
    releases/
```

The exact structure may change during the inventory pass, but every change must preserve one authoritative owner for each concept.

## Definition of done

The migration is complete only when:

```text
tracked_markdown = inventoried_markdown
unclassified = 0
broken_links = 0
orphan_documents = 0
duplicate_authorities = 0
archive_entries_without_provenance = 0
```

The final PR must include the inventory, rewrite/archive changes, validation tooling, tests, and a deterministic documentation-closure receipt.
