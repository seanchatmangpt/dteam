#!/usr/bin/env python3
"""Rewrite or archive every tracked Markdown file and emit a closure receipt.

The tool preserves stable Markdown paths. Canonical documents remain authoritative.
Every other Markdown file is copied byte-for-byte to ``docs/archive/source/<path>.txt``
and replaced at its original path by a generated supersession stub.
"""

from __future__ import annotations

import argparse
import hashlib
import json
import os
import posixpath
import re
import subprocess
import sys
from dataclasses import dataclass, asdict
from pathlib import Path, PurePosixPath
from typing import Iterable
from urllib.parse import unquote

SUPERSEDED_MARKER = "<!-- documentation-closure: superseded -->"
ARCHIVE_ROOT = PurePosixPath("docs/archive/source")
INVENTORY_JSON = PurePosixPath("docs/MARKDOWN_INVENTORY.json")
INVENTORY_MD = PurePosixPath("docs/MARKDOWN_INVENTORY.md")
MAP_MD = PurePosixPath("docs/DOCUMENTATION_MAP.md")
ARCHIVE_INDEX = PurePosixPath("docs/archive/README.md")
RECEIPT_JSON = PurePosixPath("docs/DOCUMENTATION_CLOSURE.json")

CANONICAL: dict[str, tuple[str, str]] = {
    "README.md": ("canonical", "project entry, subject boundaries, and quick start"),
    "AGENTS.md": ("canonical", "implementation doctrine for automated and human agents"),
    "CONTRIBUTING.md": ("canonical", "contribution workflow and definition of done"),
    "PHILOSOPHY.md": ("canonical", "project philosophy and licensing rationale"),
    "ATTRIBUTION.md": ("reference", "third-party attribution and retained provenance"),
    "docs/ARCHITECTURE.md": ("canonical", "architecture and authority boundaries"),
    "docs/VALIDATION.md": ("canonical", "verification, evidence, and release standing"),
    "docs/OPERATIONS.md": ("canonical", "operator and developer procedures"),
    "docs/RESEARCH.md": ("canonical", "research program, questions, and evaluation discipline"),
    "docs/GLOSSARY.md": ("canonical", "authoritative terminology"),
    "docs/DOCUMENTATION_MAP.md": ("canonical", "documentation authority map"),
    "docs/DOCUMENTATION_MIGRATION.md": ("reference", "repository-wide migration contract"),
    "docs/MARKDOWN_INVENTORY.md": ("reference", "human-readable Markdown inventory"),
    "docs/archive/README.md": ("reference", "archive index and provenance policy"),
}

LINK_RE = re.compile(r"(?<!!)\[[^\]]*\]\(([^)]+)\)")
FENCE_RE = re.compile(r"^```", re.MULTILINE)


@dataclass(frozen=True)
class Entry:
    path: str
    disposition: str
    owner: str
    replacement: str | None
    archive: str | None
    sha256: str
    bytes: int
    lines: int
    archived_source_sha256: str | None


def run_git(root: Path, *args: str) -> bytes:
    completed = subprocess.run(
        ("git", *args),
        cwd=root,
        stdout=subprocess.PIPE,
        stderr=subprocess.PIPE,
        check=False,
    )
    if completed.returncode != 0:
        raise RuntimeError(completed.stderr.decode("utf-8", errors="replace"))
    return completed.stdout


def markdown_paths(root: Path, include_untracked: bool = True) -> tuple[str, ...]:
    args = ["ls-files", "-z"]
    if include_untracked:
        args.extend(("--cached", "--others", "--exclude-standard"))
    args.extend(("--", "*.md"))
    raw = run_git(root, *args)
    return tuple(sorted({item.decode("utf-8") for item in raw.split(b"\0") if item}))


def sha256(data: bytes) -> str:
    return hashlib.sha256(data).hexdigest()


def relative_link(source: str, target: str) -> str:
    parent = posixpath.dirname(source) or "."
    return posixpath.relpath(target, parent)


def archive_path(path: str) -> str:
    return str(ARCHIVE_ROOT / f"{path}.txt")


def superseded_stub(path: str, archived: str) -> str:
    archive_link = relative_link(path, archived)
    map_link = relative_link(path, str(MAP_MD))
    return (
        f"{SUPERSEDED_MARKER}\n"
        "# Superseded documentation\n\n"
        f"`{path}` is retained as a stable repository path, but it is no longer an authoritative document.\n\n"
        f"- Exact archived source: [{archived}]({archive_link})\n"
        f"- Current documentation authorities: [{MAP_MD}]({map_link})\n\n"
        "Historical claims in the archived source describe their original context and do not establish current implementation standing.\n"
    )


def is_superseded(path: Path) -> bool:
    try:
        return path.read_text(encoding="utf-8").startswith(SUPERSEDED_MARKER)
    except UnicodeDecodeError:
        return False


def disposition_for(path: str, root: Path) -> tuple[str, str, str | None, str | None]:
    if path in CANONICAL:
        disposition, purpose = CANONICAL[path]
        return disposition, purpose, None, None
    file_path = root / path
    if is_superseded(file_path):
        archived = archive_path(path)
        return "superseded", str(MAP_MD), str(MAP_MD), archived
    return "unclassified", "", None, None


def write_text(path: Path, content: str) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text(content, encoding="utf-8", newline="\n")


def ensure_readme_map_link(root: Path) -> None:
    path = root / "README.md"
    text = path.read_text(encoding="utf-8")
    if "docs/DOCUMENTATION_MAP.md" in text:
        return
    needle = "## Documentation\n"
    addition = "- [`docs/DOCUMENTATION_MAP.md`](docs/DOCUMENTATION_MAP.md) — canonical documentation authority map\n"
    if needle in text:
        text = text.replace(needle, needle + "\n" + addition, 1)
    else:
        text += "\n## Documentation map\n\n" + addition
    write_text(path, text)


def rewrite_noncanonical(root: Path) -> list[tuple[str, str]]:
    rewritten: list[tuple[str, str]] = []
    for path in markdown_paths(root):
        if path in CANONICAL:
            continue
        source = root / path
        if is_superseded(source):
            continue
        archived = archive_path(path)
        archive_file = root / archived
        original = source.read_bytes()
        archive_file.parent.mkdir(parents=True, exist_ok=True)
        if archive_file.exists() and archive_file.read_bytes() != original:
            raise RuntimeError(f"archive identity conflict: {archived}")
        archive_file.write_bytes(original)
        write_text(source, superseded_stub(path, archived))
        rewritten.append((path, archived))
    return rewritten


def provisional_records(root: Path) -> list[dict[str, str | None]]:
    records: list[dict[str, str | None]] = []
    for path in markdown_paths(root):
        disposition, owner, replacement, archived = disposition_for(path, root)
        records.append(
            {
                "path": path,
                "disposition": disposition,
                "owner": owner,
                "replacement": replacement,
                "archive": archived,
            }
        )
    return records


def render_documentation_map(records: Iterable[dict[str, str | None]]) -> str:
    records = list(records)
    canonical = [record for record in records if record["disposition"] in {"canonical", "reference"}]
    superseded = [record for record in records if record["disposition"] == "superseded"]
    lines = [
        "# Documentation authority map",
        "",
        "This page is the authoritative index for repository documentation. A concept has one current owner; historical Markdown remains reachable through stable supersession stubs and exact archived source files.",
        "",
        "## Canonical and reference documents",
        "",
        "| Document | Disposition | Authority |",
        "|---|---|---|",
    ]
    for record in canonical:
        path = str(record["path"])
        link = relative_link(str(MAP_MD), path)
        lines.append(f"| [`{path}`]({link}) | {record['disposition']} | {record['owner']} |")
    lines.extend(
        [
            "",
            "## Historical documentation",
            "",
            f"{len(superseded)} historical Markdown paths are retained as supersession stubs. Their original bytes are indexed in [`docs/archive/README.md`](archive/README.md).",
            "",
            "## Authority rules",
            "",
            "- Architecture belongs to `docs/ARCHITECTURE.md`.",
            "- Operational procedures belong to `docs/OPERATIONS.md`.",
            "- Evidence and release standing belong to `docs/VALIDATION.md`.",
            "- Research questions and evaluation methodology belong to `docs/RESEARCH.md`.",
            "- Terminology belongs to `docs/GLOSSARY.md`.",
            "- Contribution and agent behavior belong to `CONTRIBUTING.md` and `AGENTS.md`.",
            "- Licensing motivation belongs to `PHILOSOPHY.md`; legal terms belong only to `LICENSE`.",
            "",
            "A historical file may explain lineage, but it may not override a canonical owner.",
        ]
    )
    return "\n".join(lines) + "\n"


def render_archive_index(records: Iterable[dict[str, str | None]]) -> str:
    superseded = [record for record in records if record["disposition"] == "superseded"]
    lines = [
        "# Documentation archive",
        "",
        "The archive preserves exact pre-migration Markdown bytes while keeping every original Markdown path as a stable redirect.",
        "",
        "## Provenance contract",
        "",
        "- `Original path` is rewritten as a supersession stub.",
        "- `Archived source` is an exact byte copy stored with the suffix `.md.txt`.",
        "- Archived text is historical evidence, not current implementation authority.",
        "- SHA-256 identities are recorded in `docs/MARKDOWN_INVENTORY.json`.",
        "",
        "## Archived paths",
        "",
        "| Original path | Archived source |",
        "|---|---|",
    ]
    for record in superseded:
        path = str(record["path"])
        archived = str(record["archive"])
        original_link = relative_link(str(ARCHIVE_INDEX), path)
        archived_link = relative_link(str(ARCHIVE_INDEX), archived)
        lines.append(f"| [`{path}`]({original_link}) | [`{archived}`]({archived_link}) |")
    return "\n".join(lines) + "\n"


def render_inventory_md(records: Iterable[dict[str, str | None]]) -> str:
    records = list(records)
    counts: dict[str, int] = {}
    for record in records:
        disposition = str(record["disposition"])
        counts[disposition] = counts.get(disposition, 0) + 1
    lines = [
        "# Markdown inventory",
        "",
        "This inventory is generated by `tools/documentation_closure.py`. Cryptographic identities and archive-source identities are stored in `docs/MARKDOWN_INVENTORY.json`.",
        "",
        "## Summary",
        "",
    ]
    for disposition in sorted(counts):
        lines.append(f"- `{disposition}`: {counts[disposition]}")
    lines.extend(["", "## Files", "", "| Path | Disposition | Owner or replacement |", "|---|---|---|"])
    for record in records:
        path = str(record["path"])
        target = str(record["replacement"] or record["owner"])
        lines.append(f"| `{path}` | {record['disposition']} | {target} |")
    return "\n".join(lines) + "\n"


def final_entries(root: Path) -> tuple[Entry, ...]:
    result: list[Entry] = []
    for path in markdown_paths(root):
        disposition, owner, replacement, archived = disposition_for(path, root)
        raw = (root / path).read_bytes()
        archived_digest = None
        if archived:
            archive_file = root / archived
            if archive_file.exists():
                archived_digest = sha256(archive_file.read_bytes())
        result.append(
            Entry(
                path=path,
                disposition=disposition,
                owner=owner,
                replacement=replacement,
                archive=archived,
                sha256=sha256(raw),
                bytes=len(raw),
                lines=raw.count(b"\n") + (1 if raw and not raw.endswith(b"\n") else 0),
                archived_source_sha256=archived_digest,
            )
        )
    return tuple(result)


def parse_links(text: str) -> Iterable[str]:
    # Generated canonical documents do not contain image links. Ignore URLs, anchors,
    # mail addresses, and templating expressions.
    for match in LINK_RE.finditer(text):
        target = match.group(1).strip().split(maxsplit=1)[0].strip("<>")
        if not target or target.startswith(("#", "http://", "https://", "mailto:", "tel:", "data:")):
            continue
        if "${{" in target or "{{" in target:
            continue
        yield unquote(target.split("#", 1)[0].split("?", 1)[0])


def validate_links(root: Path, entries: Iterable[Entry]) -> list[str]:
    failures: list[str] = []
    for entry in entries:
        source = root / entry.path
        text = source.read_text(encoding="utf-8")
        for target in parse_links(text):
            resolved = (source.parent / target).resolve()
            try:
                resolved.relative_to(root.resolve())
            except ValueError:
                failures.append(f"outside-root:{entry.path}:{target}")
                continue
            if not resolved.exists():
                failures.append(f"broken:{entry.path}:{target}")
    return sorted(set(failures))


def markdown_graph(root: Path, entries: Iterable[Entry]) -> dict[str, set[str]]:
    paths = {entry.path for entry in entries}
    graph = {path: set() for path in paths}
    for path in paths:
        source = root / path
        for target in parse_links(source.read_text(encoding="utf-8")):
            resolved = (source.parent / target).resolve()
            try:
                relative = resolved.relative_to(root.resolve()).as_posix()
            except ValueError:
                continue
            if relative in paths:
                graph[path].add(relative)
    return graph


def unreachable_documents(root: Path, entries: Iterable[Entry]) -> list[str]:
    entries = tuple(entries)
    graph = markdown_graph(root, entries)
    seen: set[str] = set()
    stack = ["README.md"]
    while stack:
        current = stack.pop()
        if current in seen:
            continue
        seen.add(current)
        stack.extend(sorted(graph.get(current, ())))
    return sorted(set(graph) - seen)


def validate(entries: tuple[Entry, ...], root: Path) -> dict[str, object]:
    unclassified = sorted(entry.path for entry in entries if entry.disposition == "unclassified")
    missing_canonical = sorted(set(CANONICAL) - {entry.path for entry in entries})
    archive_missing = sorted(
        entry.path for entry in entries
        if entry.disposition == "superseded" and (not entry.archive or not (root / entry.archive).is_file())
    )
    archive_identity_missing = sorted(
        entry.path for entry in entries
        if entry.disposition == "superseded" and not entry.archived_source_sha256
    )
    broken_links = validate_links(root, entries)
    orphans = unreachable_documents(root, entries)
    return {
        "tracked_markdown": len(markdown_paths(root, include_untracked=False)),
        "inventoried_markdown": len(entries),
        "unclassified": unclassified,
        "missing_canonical": missing_canonical,
        "broken_links": broken_links,
        "orphan_documents": orphans,
        "duplicate_authorities": [],
        "archive_entries_without_provenance": sorted(set(archive_missing + archive_identity_missing)),
    }


def standing(validation: dict[str, object]) -> str:
    numeric_equal = validation["tracked_markdown"] == validation["inventoried_markdown"]
    collections_empty = all(
        not validation[key]
        for key in (
            "unclassified",
            "missing_canonical",
            "broken_links",
            "orphan_documents",
            "duplicate_authorities",
            "archive_entries_without_provenance",
        )
    )
    return "ALIVE" if numeric_equal and collections_empty else "BUILD_BROKEN"


def manufacture(root: Path) -> tuple[Entry, ...]:
    rewrite_noncanonical(root)
    ensure_readme_map_link(root)

    # The generated documents are themselves canonical inventory members.
    provisional = provisional_records(root)
    write_text(root / MAP_MD, render_documentation_map(provisional))
    provisional = provisional_records(root)
    write_text(root / ARCHIVE_INDEX, render_archive_index(provisional))
    provisional = provisional_records(root)
    write_text(root / INVENTORY_MD, render_inventory_md(provisional))

    entries = final_entries(root)
    inventory_payload = {
        "schema": "urn:dteam:markdown-inventory:v1",
        "entries": [asdict(entry) for entry in entries],
    }
    write_text(root / INVENTORY_JSON, json.dumps(inventory_payload, indent=2, sort_keys=True) + "\n")
    entries = final_entries(root)
    result = validate(entries, root)
    receipt = {
        "schema": "urn:dteam:documentation-closure:v1",
        "standing": standing(result),
        "validation": result,
        "inventory_sha256": sha256((root / INVENTORY_JSON).read_bytes()),
        "markdown_set_sha256": sha256(
            json.dumps(
                [(entry.path, entry.sha256, entry.disposition) for entry in entries],
                separators=(",", ":"),
            ).encode("utf-8")
        ),
    }
    raw = json.dumps(receipt, sort_keys=True, separators=(",", ":")).encode("utf-8")
    receipt["receipt_sha256"] = sha256(raw)
    write_text(root / RECEIPT_JSON, json.dumps(receipt, indent=2, sort_keys=True) + "\n")
    return entries


def check(root: Path) -> tuple[str, dict[str, object]]:
    entries = final_entries(root)
    result = validate(entries, root)
    current = standing(result)
    receipt_path = root / RECEIPT_JSON
    if receipt_path.exists():
        receipt = json.loads(receipt_path.read_text(encoding="utf-8"))
        if receipt.get("standing") != current:
            result.setdefault("receipt_errors", []).append("standing_mismatch")
            current = "BUILD_BROKEN"
    else:
        result.setdefault("receipt_errors", []).append("missing_receipt")
        current = "BUILD_BROKEN"
    return current, result


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--root", type=Path, default=Path.cwd())
    mode = parser.add_mutually_exclusive_group(required=True)
    mode.add_argument("--apply", action="store_true")
    mode.add_argument("--check", action="store_true")
    args = parser.parse_args()

    root = args.root.resolve(strict=True)
    if args.apply:
        manufacture(root)
    current, result = check(root)
    print(json.dumps({"standing": current, **result}, indent=2, sort_keys=True))
    return 0 if current == "ALIVE" else 1


if __name__ == "__main__":
    raise SystemExit(main())
