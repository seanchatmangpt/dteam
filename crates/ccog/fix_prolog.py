import re

with open("crates/ccog/src/breeds/prolog.rs", "r") as f:
    content = f.read()

content = content.replace("use std::collections::{HashMap, HashSet, VecDeque};", "use std::collections::{HashSet, VecDeque};\nuse crate::utils::dense::{fnv1a_64, PackedKeyTable};")
content = content.replace("let mut parent: HashMap<String, GraphIri> = HashMap::new();", "let mut parent: PackedKeyTable<String, GraphIri> = PackedKeyTable::new();")
content = content.replace("parent.insert(n_str.clone(), current.clone());", "parent.insert(fnv1a_64(n_str.as_bytes()), n_str.clone(), current.clone());")
content = content.replace("while let Some(p) = parent.get(&curr) {", "while let Some(p) = parent.get(fnv1a_64(curr.as_bytes())) {")

with open("crates/ccog/src/breeds/prolog.rs", "w") as f:
    f.write(content)
