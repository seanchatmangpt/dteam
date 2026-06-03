import sys

path = "/Users/sac/insa/insa-truthforge/src/lib.rs"
with open(path, "r") as f:
    content = f.read()

content = content.replace("pub fn verify_ontology_signature", "/// Verifies the signature of the RDF ontology projection graph.\\npub fn verify_ontology_signature")

with open(path, "w") as f:
    f.write(content)
