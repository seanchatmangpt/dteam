import sys

path = "/Users/sac/insa/insa-proof/src/powl64.rs"
with open(path, "r") as f:
    content = f.read()

# Fix the literal \n
content = content.replace("/// Fix 6: Byzantine Wire Assurance Cryptographic Signature\\n    pub signature: [u8; 32],", "/// Fix 6: Byzantine Wire Assurance Cryptographic Signature\n    pub signature: [u8; 32],")

with open(path, "w") as f:
    f.write(content)
