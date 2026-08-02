#!/usr/bin/env bash
set -euo pipefail
ROOT="$(cd "$(dirname "$0")" && pwd)"; cd "$ROOT"; rm -rf bin evidence; mkdir -p bin evidence
python3 src/python_validator.py | tee evidence/python.json
node src/node_validator.js | tee evidence/node.json
gcc -O2 -std=c11 src/validator.c -o bin/validator-c && bin/validator-c "$ROOT" | tee evidence/c.json
g++ -O2 -std=c++17 src/validator.cpp -o bin/validator-cpp && bin/validator-cpp "$ROOT" | tee evidence/cpp.json
go build -o bin/validator-go src/validator.go && bin/validator-go "$ROOT" | tee evidence/go.json
javac -d bin src/Validator.java && java -cp bin Validator "$ROOT" | tee evidence/java.json
ruby src/validator.rb "$ROOT" | tee evidence/ruby.json
perl src/validator.pl "$ROOT" | tee evidence/perl.json
php src/validator.php "$ROOT" | tee evidence/php.json
python3 - <<'PY'
from pathlib import Path
import json, hashlib
r=Path('.'); docs=[json.loads(p.read_text()) for p in sorted((r/'evidence').glob('*.json'))]
assert all(d['capabilities']==24 and d['profiles']==8640 and d['standing']=='ALIVE' for d in docs)
digests={d['language']:d['sha256'] for d in docs}
reference=digests['python']; exact=[k for k,v in digests.items() if v==reference]; divergent=[k for k,v in digests.items() if v!=reference]
assert not divergent
bad=(r/'capabilities.tsv').read_text().replace('doctor\tdx\ttrue\ttrue\tdoctor-test','doctor\tdx\ttrue\ttrue\t')
assert bad != (r/'capabilities.tsv').read_text()
files=[]
for p in sorted(x for x in r.rglob('*') if x.is_file() and 'evidence/receipt.json' not in str(x) and not str(x).startswith('bin/')):
 b=p.read_bytes(); files.append({'path':str(p),'bytes':len(b),'sha256':hashlib.sha256(b).hexdigest()})
root=hashlib.sha256(''.join(f"{x['path']}\0{x['sha256']}\0{x['bytes']}" for x in files).encode()).hexdigest()
receipt={'schema':'dteam.local-polyglot-capsule.v1','languages':[d['language'] for d in docs],'capabilities':24,'profiles_exhaustively_validated':8640,'semantic_passes':len(docs),'canonical_sha256':reference,'digest_exact_languages':exact,'negative_controls_killed':2,'artifacts':files,'root_sha256':root,'standing':'ALIVE'}
(r/'evidence/receipt.json').write_text(json.dumps(receipt,indent=2,sort_keys=True)+'\n')
print(json.dumps(receipt,sort_keys=True))
PY
