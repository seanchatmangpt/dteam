from pathlib import Path
import hashlib, json, re
ROOT=Path(__file__).resolve().parents[1]

def rows(name):
    lines=(ROOT/name).read_text().splitlines()
    return [line.split("\t") for line in lines[1:] if line.strip()]

caps=rows('capabilities.tsv')
checks=rows('innovation-checkpoints.tsv')
axes=[]
for line in (ROOT/'axes.tsv').read_text().splitlines():
    key, values=line.split('\t')
    axes.append((key, values.split(',')))
profiles=1
for _, values in axes: profiles*=len(values)
source_files=[p for p in (ROOT/'src').glob('*') if p.name != 'innovation_report.py']
results={}
for path in sorted((ROOT/'evidence').glob('*.json')):
    if path.name in {'receipt.json','innovation-checkpoints.json'}: continue
    try:
        doc=json.loads(path.read_text())
    except json.JSONDecodeError:
        continue
    if 'language' in doc: results[doc['language']]=doc
canonical=hashlib.sha256((ROOT/'canonical.bin').read_bytes()).hexdigest()
all_reversible=all(row[2]=='true' for row in caps)
no_network=True
for path in [ROOT/'run-local.sh', *source_files]:
    text=path.read_text(errors='ignore')
    if re.search(r'\b(curl|wget|git clone|npm install|cargo install)\b', text): no_network=False
contract_fields={'language','capabilities','checkpoints','profiles','sha256','standing'}
contract_ok=all(contract_fields <= set(doc) for doc in results.values())
exact=all(doc['sha256']==canonical for doc in results.values())
criteria={
 'G1': len(results)==9 and all(d['standing']=='ALIVE' for d in results.values()),
 'G2': len(results)==9,
 'G3': exact and contract_ok,
 'G4': len(caps)==24 and profiles==8640,
 'G5': no_network,
 'G6': contract_ok and len(results)==9,
 'G7': (ROOT/'run-local.sh').exists() and len(results)==9,
 'G8': 'set -euo pipefail' in (ROOT/'run-local.sh').read_text(),
 'G9': all_reversible and len(caps)==24,
 'G10': len(checks)==10 and profiles==8640 and len(results)==9,
}
entries=[]
for row in checks:
    cid,lineage,principle,observable,admission,falsifier=row
    entries.append({'id':cid,'lineage':lineage,'principle':principle,'observable':observable,'admission':admission,'falsifier':falsifier,'state':'PASS' if criteria.get(cid,False) else 'FAIL'})
failed=[x for x in entries if x['state']!='PASS']
out={'schema':'dteam.gall-innovation-checkpoints.v1','checkpoints':entries,'passed':len(entries)-len(failed),'failed':len(failed),'capabilities':len(caps),'profiles':profiles,'runtime_languages':sorted(results),'canonical_sha256':canonical,'standing':'ALIVE' if not failed else 'BUILD_BROKEN'}
(ROOT/'evidence'/'innovation-checkpoints.json').write_text(json.dumps(out,indent=2,sort_keys=True)+'\n')
print(json.dumps(out,sort_keys=True))
raise SystemExit(0 if not failed else 1)
