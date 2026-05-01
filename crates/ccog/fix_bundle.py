import re

with open("crates/ccog/src/export/bundle.rs", "r") as f:
    content = f.read()

content = content.replace("use std::collections::BTreeMap;", "use crate::utils::dense::{fnv1a_64, PackedKeyTable};")
content = content.replace("pub entries: BTreeMap<String, Vec<u8>>,", "pub entries: PackedKeyTable<String, Vec<u8>>,")
content = content.replace("let mut entries: BTreeMap<String, Vec<u8>> = BTreeMap::new();", "let mut entries: PackedKeyTable<String, Vec<u8>> = PackedKeyTable::new();")
content = content.replace("entries.insert(\"ontology-refs.txt\".into(), refs_blob.into_bytes());", "entries.insert(fnv1a_64(b\"ontology-refs.txt\"), \"ontology-refs.txt\".into(), refs_blob.into_bytes());")
content = content.replace("entries.insert(\"powl64-path.bin\".into(), powl64_path);", "entries.insert(fnv1a_64(b\"powl64-path.bin\"), \"powl64-path.bin\".into(), powl64_path);")
content = content.replace("entries.insert(\"receipt.jsonld\".into(), receipt_jsonld);", "entries.insert(fnv1a_64(b\"receipt.jsonld\"), \"receipt.jsonld\".into(), receipt_jsonld);")
content = content.replace("entries.insert(\"tier.txt\".into(), format!(\"{:?}\\n\", tier).into_bytes());", "entries.insert(fnv1a_64(b\"tier.txt\"), \"tier.txt\".into(), format!(\"{:?}\\n\", tier).into_bytes());")
content = content.replace("entries.insert(\"trace.jsonld\".into(), trace_jsonld);", "entries.insert(fnv1a_64(b\"trace.jsonld\"), \"trace.jsonld\".into(), trace_jsonld);")
content = content.replace("entries.insert(\"manifest.json\".into(), manifest);", "entries.insert(fnv1a_64(b\"manifest.json\"), \"manifest.json\".into(), manifest);")
content = content.replace("for (name, data) in &self.entries {", "for (_, name, data) in self.entries.iter() {")
content = content.replace("entries.insert(path, buf);", "entries.insert(fnv1a_64(path.as_bytes()), path.clone(), buf);")
content = content.replace("entries\n            .get(\"manifest.json\")", "entries\n            .get(fnv1a_64(b\"manifest.json\"))")
content = content.replace("entries\n                .get(name)", "entries\n                .get(fnv1a_64(name.as_bytes()))")
content = content.replace("self.entries.get(name).map(Vec::as_slice)", "self.entries.get(fnv1a_64(name.as_bytes())).map(Vec::as_slice)")
content = content.replace("fn manifest_json(entries: &BTreeMap<String, Vec<u8>>) -> Vec<u8> {", "fn manifest_json(entries: &PackedKeyTable<String, Vec<u8>>) -> Vec<u8> {")
content = content.replace("for (name, data) in entries {", "for (_, name, data) in entries.iter() {")

with open("crates/ccog/src/export/bundle.rs", "w") as f:
    f.write(content)
