import re

with open("crates/ccog/src/breeds/eliza.rs", "r") as f:
    content = f.read()

content = content.replace("if let Some(iris) = index.get_mut(&label) {", "if let Some(iris) = index.get_mut(fnv1a_64(label.as_bytes())) {")
content = content.replace("index.insert(label.clone(), vec![subject.clone()]);", "index.insert(fnv1a_64(label.as_bytes()), label.clone(), vec![subject.clone()]);")

with open("crates/ccog/src/breeds/eliza.rs", "w") as f:
    f.write(content)
