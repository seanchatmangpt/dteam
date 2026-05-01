import re

with open("crates/ccog/src/abi/jsonld.rs", "r") as f:
    content = f.read()

content = content.replace("use std::collections::BTreeMap;", "use crate::utils::dense::{fnv1a_64, PackedKeyTable};")
content = content.replace("let mut out: Map<String, Value> = Map::with_capacity(b.len());\n    for (k, v) in b {\n        out.insert(k.to_string(), v);\n    }", "let mut out: Map<String, Value> = Map::with_capacity(b.len());\n    for (_, k, v) in b.iter() {\n        out.insert(k.to_string(), v.clone());\n    }")
content = content.replace("fn into_map(b: BTreeMap<&'static str, Value>) -> Map<String, Value> {", "fn into_map(b: PackedKeyTable<&'static str, Value>) -> Map<String, Value> {")
content = content.replace("let mut root: BTreeMap<&'static str, Value> = BTreeMap::new();", "let mut root: PackedKeyTable<&'static str, Value> = PackedKeyTable::new();")

def replace_root_insert(match):
    key_str = match.group(1)
    val_str = match.group(2)
    return f'root.insert(fnv1a_64({key_str}.as_bytes()), {key_str}, {val_str});'

content = re.sub(r'root\.insert\((.*?),\s*(.*?)\);', replace_root_insert, content)

content = content.replace("let mut m: BTreeMap<&'static str, Value> = BTreeMap::new();", "let mut m: PackedKeyTable<&'static str, Value> = PackedKeyTable::new();")

def replace_m_insert(match):
    key_str = match.group(1)
    val_str = match.group(2)
    return f'm.insert(fnv1a_64({key_str}.as_bytes()), {key_str}, {val_str});'

content = re.sub(r'm\.insert\((.*?),\s*(.*?)\);', replace_m_insert, content)

with open("crates/ccog/src/abi/jsonld.rs", "w") as f:
    f.write(content)
