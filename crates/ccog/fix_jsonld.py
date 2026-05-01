import re

with open("crates/ccog/src/export/jsonld.rs", "r") as f:
    content = f.read()

content = content.replace("use std::collections::BTreeMap;", "use crate::utils::dense::{fnv1a_64, PackedKeyTable};")
content = content.replace("let mut m = BTreeMap::new();", "let mut m = PackedKeyTable::new();")
content = content.replace("m.insert((*k).to_string(), Value::String((*v).to_string()));", "m.insert(fnv1a_64(k.as_bytes()), (*k).to_string(), Value::String((*v).to_string()));")
content = content.replace("fn btreemap_to_value(m: BTreeMap<String, Value>) -> Value {", "fn btreemap_to_value(m: PackedKeyTable<String, Value>) -> Value {")
content = content.replace("for (k, v) in m {", "for (_, k, v) in m.iter() {\n        let k = k.clone();\n        let v = v.clone();")
content = content.replace("let mut root: BTreeMap<String, Value> = BTreeMap::new();", "let mut root: PackedKeyTable<String, Value> = PackedKeyTable::new();")

def replace_insert(match):
    key_str = match.group(1)
    val_str = match.group(2)
    return f'root.insert(fnv1a_64({key_str}.as_bytes()), {key_str}.into(), {val_str});'

content = re.sub(r'root\.insert\((.*?)\.into\(\),\s*(.*?)\);', replace_insert, content)

content = content.replace("let mut node: BTreeMap<String, Value> = BTreeMap::new();", "let mut node: PackedKeyTable<String, Value> = PackedKeyTable::new();")

def replace_node_insert(match):
    key_str = match.group(1)
    val_str = match.group(2)
    return f'node.insert(fnv1a_64({key_str}.as_bytes()), {key_str}.into(), {val_str});'

content = re.sub(r'node\.insert\((.*?)\.into\(\),\s*(.*?)\);', replace_node_insert, content)

content = content.replace("let mut m: BTreeMap<String, Value> = BTreeMap::new();", "let mut m: PackedKeyTable<String, Value> = PackedKeyTable::new();")

def replace_m_insert(match):
    key_str = match.group(1)
    val_str = match.group(2)
    return f'm.insert(fnv1a_64({key_str}.as_bytes()), {key_str}.into(), {val_str});'

content = re.sub(r'm\.insert\((.*?)\.into\(\),\s*(.*?)\);', replace_m_insert, content)

with open("crates/ccog/src/export/jsonld.rs", "w") as f:
    f.write(content)
