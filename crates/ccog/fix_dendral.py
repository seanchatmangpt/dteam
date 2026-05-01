import re

with open("crates/ccog/src/breeds/dendral.rs", "r") as f:
    content = f.read()

content = content.replace("use std::collections::{BTreeMap, HashSet};", "use std::collections::HashSet;\nuse crate::utils::dense::{fnv1a_64, PackedKeyTable};")
content = content.replace("let mut by_activity: BTreeMap<String, (NamedNode, Vec<NamedNode>)> = BTreeMap::new();", "let mut by_activity: PackedKeyTable<String, (NamedNode, Vec<NamedNode>)> = PackedKeyTable::new();")
content = content.replace("by_activity.insert(act.as_str().to_string(), (act, inputs));", "let key = act.as_str().to_string();\n                by_activity.insert(fnv1a_64(key.as_bytes()), key, (act, inputs));")
content = content.replace("for (_, (activity, inputs)) in by_activity {", "for (_, _, (activity, inputs)) in by_activity.iter() {")

with open("crates/ccog/src/breeds/dendral.rs", "w") as f:
    f.write(content)
