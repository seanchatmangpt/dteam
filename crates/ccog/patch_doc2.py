import sys

path = "/Users/sac/insa/insa-truthforge/src/lib.rs"
with open(path, "r") as f:
    content = f.read()

# Replace the literal '\n' and missing brace issues
content = content.replace("/// Verifies the signature of the RDF ontology projection graph.\\npub fn verify_ontology_signature", "/// Verifies the signature of the RDF ontology projection graph.\npub fn verify_ontology_signature")

with open(path, "w") as f:
    f.write(content)
