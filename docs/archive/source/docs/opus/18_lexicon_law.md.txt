# 18 — Lexicon Law

## The rule

Documentation and abstractions must remain bit-native.

**Allowed:**
- `8^n`, `64^n`, `U_{1,n}`
- exact bit counts (e.g. `4,096 bits`, `262,144 bits`)
- `truth`, `scratch`, `motion`, `delta`, `receipt`

**Forbidden:**
The ordinary storage noun that collapses bit-native ontology back into
conventional storage vocabulary. The forbidden word is never written in the
source of the checker itself; it is reconstructed from code points at runtime.

## Why

Preserves conceptual closure. The moment an agent writes the forbidden word,
it has likely left the unibit ontology and started using the wrong
abstraction. Enforce at build time, not review time.

## `bin/check-lexicon.mjs`

```javascript
#!/usr/bin/env node
import fs from "node:fs";
import path from "node:path";

const root = process.argv[2] ? path.resolve(process.argv[2]) : process.cwd();

const forbidden = String.fromCharCode(98, 121, 116, 101);
const pattern = new RegExp(`\\b${forbidden}s?\\b`, "i");

const ignoredDirs = new Set([
  ".git", "target", "node_modules", "dist", "build", "coverage",
  ".next", ".turbo", "vendor",
]);
const ignoredFiles = new Set([
  "Cargo.lock", "pnpm-lock.yaml", "package-lock.json", "yarn.lock",
]);
const allowFiles = new Set([path.normalize("bin/check-lexicon.mjs")]);
const textExtensions = new Set([
  ".rs", ".md", ".toml", ".yml", ".yaml", ".json", ".js", ".mjs",
  ".cjs", ".ts", ".tsx", ".jsx", ".sh", ".bash", ".zsh", ".fish",
  ".c", ".h", ".hpp", ".cpp", ".cc", ".hh", ".nix", ".txt",
]);

function walk(dir, out = []) {
  for (const entry of fs.readdirSync(dir, { withFileTypes: true })) {
    const full = path.join(dir, entry.name);
    const rel = path.relative(root, full);
    if (entry.isDirectory()) {
      if (!ignoredDirs.has(entry.name)) walk(full, out);
      continue;
    }
    if (!entry.isFile()) continue;
    if (ignoredFiles.has(entry.name)) continue;
    if (allowFiles.has(path.normalize(rel))) continue;
    if (!textExtensions.has(path.extname(full))) continue;
    out.push(full);
  }
  return out;
}

const failures = [];
for (const file of walk(root)) {
  const text = fs.readFileSync(file, "utf8");
  const lines = text.split(/\r?\n/);
  for (let i = 0; i < lines.length; i++) {
    const match = pattern.exec(lines[i]);
    if (!match) continue;
    failures.push({ file: path.relative(root, file), line: i + 1,
                    col: match.index + 1, value: match[0] });
  }
}

if (failures.length > 0) {
  console.error("\nunibit lexicon violation\n");
  console.error("Use bit-native notation only: 8^n, 64^n, U_{1,n}, or exact bit counts.\n");
  for (const f of failures) {
    console.error(`${f.file}:${f.line}:${f.col} -> ${JSON.stringify(f.value)}`);
  }
  process.exit(1);
}

console.log("unibit lexicon check passed");
```

## Replacement dictionary

| Concept | Correct language |
|---|---|
| smallest work atom | `U_{1,8}` or `8 bits` |
| word atom | `U_{1,64}` or `64 bits` |
| semantic cache line | `U_{1,512}` or `512 bits` |
| attention block | `U_{1,4096}` or `4,096 bits` |
| active tile | `U_{1,32768}` or `32,768 bits` |
| active universe | `U_{1,262144}` or `262,144 bits` |
| meaning field | `U_{1,16777216}` or `16,777,216 bits` |

## Rule

```
Memory is 64^n.
Work is 8^n.
Use exact bit counts when clarity is required.
```

## CI gate

```yaml
name: lexicon
on:
  pull_request:
  push:
    branches: [main]
jobs:
  lexicon:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: actions/setup-node@v4
        with:
          node-version: 22
      - run: node bin/check-lexicon.mjs
```

## Purpose

Prevents conceptual regression. Fails the build immediately so code agents
learn the ontology at emit time, not review time.
