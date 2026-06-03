import sys

path = "/Users/sac/clap-noun-verb/src/cli/mod.rs"
with open(path, "r") as f:
    content = f.read()

content = content.replace("pub mod generated_verbs;", "")
content = content.replace("pub mod domain_traits;", "")

with open(path, "w") as f:
    f.write(content)
