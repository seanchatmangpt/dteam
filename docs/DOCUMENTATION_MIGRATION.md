# Repository-wide Markdown closure

This document records the completed documentation migration stacked on `agent/ggen-alive-closure`.

## Objective

Every tracked `*.md` file receives one explicit disposition:

- **Canonical** — current authoritative documentation.
- **Reference** — retained current policy, attribution, inventory, or migration evidence.
- **Superseded** — stable redirect to the current authority map and exact archived source.

The migration does not silently erase historical documents. A superseded Markdown file remains at its original path as a redirect, while its exact prior bytes are preserved beneath `docs/archive/source/` using the suffix `.md.txt`.

## Canonical structure

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
  DOCUMENTATION_MAP.md
  MARKDOWN_INVENTORY.md
  DOCUMENTATION_MIGRATION.md
  archive/
    README.md
    source/
```

## Completed disposition

```text
canonical: 10
reference: 4
superseded: 227
total Markdown paths: 241
```

The complete path-level inventory is available in:

- [`MARKDOWN_INVENTORY.md`](MARKDOWN_INVENTORY.md)
- [`MARKDOWN_INVENTORY.json`](MARKDOWN_INVENTORY.json)

The documentation authority graph begins at [`DOCUMENTATION_MAP.md`](DOCUMENTATION_MAP.md).

## Validation contract

The closure engine verifies:

```text
tracked_markdown = inventoried_markdown
unclassified = 0
broken_links = 0
orphan_documents = 0
duplicate_authorities = 0
archive_entries_without_provenance = 0
missing_canonical = 0
```

Observed result:

```text
tracked_markdown = 241
inventoried_markdown = 241
unclassified = 0
broken_links = 0
orphan_documents = 0
duplicate_authorities = 0
archive_entries_without_provenance = 0
missing_canonical = 0
standing = ALIVE
```

The machine-readable result is committed as [`DOCUMENTATION_CLOSURE.json`](DOCUMENTATION_CLOSURE.json).

## Editorial rules

- Define terms before using them.
- Separate current behavior, intended behavior, and historical behavior.
- Prefer executable commands over prose-only procedures.
- Do not claim `ALIVE` without an exact observed execution receipt.
- Preserve attribution and licensing semantics.
- Keep architecture, operations, validation, contribution policy, research, and history under distinct authorities.
- Archive superseded designs rather than rewriting historical claims into current claims.

## Reproduction

```bash
python -m unittest tools.tests.test_documentation_closure -v
python tools/documentation_closure.py --root . --apply
python tools/documentation_closure.py --root . --check
```

`--apply` is idempotent. A second run must preserve every archived source identity and produce zero new Markdown rewrites.

## Standing

The repository-wide Markdown closure subject is `ALIVE`. This standing covers the 241-file documentation inventory and its archive, link, and authority invariants. It does not imply that unrelated software build or runtime subjects are `ALIVE`.
