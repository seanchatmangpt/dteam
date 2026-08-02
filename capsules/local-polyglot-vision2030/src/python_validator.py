from pathlib import Path
import json, hashlib
ROOT=Path(__file__).resolve().parents[1]
caps=[l.split('\t') for l in (ROOT/'capabilities.tsv').read_text().splitlines()[1:] if l.strip()]
axes=[]
for line in (ROOT/'axes.tsv').read_text().splitlines(): k,v=line.split('\t'); axes.append((k,v.split(',')))
product=1
for _,v in axes: product*=len(v)
assert len(caps)==24 and len({c[0] for c in caps})==24 and product==8640
assert 'ORDER BY' in (ROOT/'queries/extract.rq').read_text()
out={'language':'python','capabilities':24,'profiles':8640,'sha256':hashlib.sha256((ROOT/'canonical.bin').read_bytes()).hexdigest(),'standing':'ALIVE'}
print(json.dumps(out,sort_keys=True))
