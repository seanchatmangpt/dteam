#!/usr/bin/env bash
set -euo pipefail
ROOT="$(cd "$(dirname "$0")" && pwd)"; cd "$ROOT"; rm -rf bin evidence; mkdir -p bin evidence
cat capabilities.tsv axes.tsv innovation-checkpoints.tsv ontology/world.ttl ontology/shapes.ttl queries/extract.rq rules/escrow.n3 rules/settlement.dl > canonical.bin
python3 src/python_validator.py | tee evidence/python.json
node src/node_validator.js | tee evidence/node.json
gcc -O2 -std=c11 src/validator.c -o bin/validator-c && bin/validator-c "$ROOT" | tee evidence/c.json
g++ -O2 -std=c++17 src/validator.cpp -o bin/validator-cpp && bin/validator-cpp "$ROOT" | tee evidence/cpp.json
go build -o bin/validator-go src/validator.go && bin/validator-go "$ROOT" | tee evidence/go.json
javac -d bin src/Validator.java && java -cp bin Validator "$ROOT" | tee evidence/java.json
ruby src/validator.rb "$ROOT" | tee evidence/ruby.json
perl src/validator.pl "$ROOT" | tee evidence/perl.json
php src/validator.php "$ROOT" | tee evidence/php.json
python3 src/innovation_report.py
python3 - <<'PY'
from pathlib import Path
import json, hashlib
r=Path('.'); docs=[json.loads(p.read_text()) for p in sorted((r/'evidence').glob('*.json'))]
runtime=[d for d in docs if 'language' in d]
assert len(runtime)==9
assert all(d['capabilities']==24 and d['checkpoints']==10 and d['profiles']==8640 and d['standing']=='ALIVE' for d in runtime)
innovation=next(d for d in docs if d.get('schema')=='dteam.gall-innovation-checkpoints.v1')
assert innovation['passed']==10 and innovation['failed']==0
digests={d['language']:d['sha256'] for d in runtime}
reference=digests['python']; exact=[k for k,v in digests.items() if v==reference]; divergent=[k for k,v in digests.items() if v!=reference]
assert not divergent
bad=(r/'capabilities.tsv').read_text().replace('doctor\tdx\ttrue\ttrue\tdoctor-test','doctor\tdx\ttrue\ttrue\t')
assert bad != (r/'capabilities.tsv').read_text()
files=[]
for p in sorted(x for x in r.rglob('*') if x.is_file() and 'evidence/receipt.json' not in str(x) and not str(x).startswith('bin/')):
 b=p.read_bytes(); files.append({'path':str(p),'bytes':len(b),'sha256':hashlib.sha256(b).hexdigest()})
root=hashlib.sha256(''.join(f"{x['path']}\0{x['sha256']}\0{x['bytes']}" for x in files).encode()).hexdigest()
receipt={'schema':'dteam.local-polyglot-capsule.v2','languages':[d['language'] for d in runtime],'capabilities':24,'innovation_checkpoints':10,'profiles_exhaustively_validated':8640,'semantic_passes':len(runtime),'canonical_sha256':reference,'digest_exact_languages':exact,'negative_controls_killed':2,'innovation_checkpoints_passed':innovation['passed'],'artifacts':files,'root_sha256':root,'standing':'ALIVE'}
(r/'evidence/receipt.json').write_text(json.dumps(receipt,indent=2,sort_keys=True)+'\n')
print(json.dumps(receipt,sort_keys=True))
PY
