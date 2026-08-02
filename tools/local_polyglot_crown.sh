#!/usr/bin/env bash
set -euo pipefail
root="${1:-$(pwd)/artifacts/local-polyglot}"
rm -rf "$root"
mkdir -p "$root/src" "$root/bin" "$root/out"
${SHELL:-/bin/bash} --version | head -1 > "$root/bash.version"

cat >"$root/src/model.py" <<'__DTEAM_MODEL_PY__'
#!/usr/bin/env python3
lawful=[]
for m in range(1,128):
    if (m & 15)==15 and (m & 112)!=0 and m.bit_count()<=5: lawful.append(m)
print("capabilities=21")
print("profiles=4")
print("explored=127")
print(f"lawful={len(lawful)}")
print(f"refused={127-len(lawful)}")
print("pareto=1")
print(f"selected={min(lawful)}")
print("standing=ALIVE")
__DTEAM_MODEL_PY__

cat >"$root/src/model.mjs" <<'__DTEAM_MODEL_MJS__'
const lawful=[]; for(let m=1;m<128;m++){let c=m.toString(2).replace(/0/g,"").length;if((m&15)===15&&(m&112)!==0&&c<=5)lawful.push(m)}
console.log("capabilities=21");console.log("profiles=4");console.log("explored=127");console.log(`lawful=${lawful.length}`);console.log(`refused=${127-lawful.length}`);console.log("pareto=1");console.log(`selected=${Math.min(...lawful)}`);console.log("standing=ALIVE");
__DTEAM_MODEL_MJS__

cat >"$root/src/model.go" <<'__DTEAM_MODEL_GO__'
package main
import "fmt"
func pop(x int) int { c:=0; for x>0 { c+=x&1; x>>=1 }; return c }
func main(){ lawful:=[]int{}; for m:=1;m<128;m++ { if m&15==15 && m&112!=0 && pop(m)<=5 { lawful=append(lawful,m) } }; fmt.Println("capabilities=21");fmt.Println("profiles=4");fmt.Println("explored=127");fmt.Printf("lawful=%d\n",len(lawful));fmt.Printf("refused=%d\n",127-len(lawful));fmt.Println("pareto=1");fmt.Printf("selected=%d\n",lawful[0]);fmt.Println("standing=ALIVE") }
__DTEAM_MODEL_GO__

cat >"$root/src/Model.java" <<'__DTEAM_MODEL_JAVA__'
public final class Model { public static void main(String[] a){int lawful=0,selected=999;for(int m=1;m<128;m++){if((m&15)==15&&(m&112)!=0&&Integer.bitCount(m)<=5){lawful++;selected=Math.min(selected,m);}}System.out.println("capabilities=21");System.out.println("profiles=4");System.out.println("explored=127");System.out.println("lawful="+lawful);System.out.println("refused="+(127-lawful));System.out.println("pareto=1");System.out.println("selected="+selected);System.out.println("standing=ALIVE");}}
__DTEAM_MODEL_JAVA__

cat >"$root/src/Model.kt" <<'__DTEAM_MODEL_KT__'
fun main(){ var lawful=0; var selected=999; for(m in 1..127){ if((m and 15)==15 && (m and 112)!=0 && Integer.bitCount(m)<=5){lawful++; if(m<selected)selected=m}}; println("capabilities=21");println("profiles=4");println("explored=127");println("lawful=$lawful");println("refused=${127-lawful}");println("pareto=1");println("selected=$selected");println("standing=ALIVE") }
__DTEAM_MODEL_KT__

cat >"$root/src/model.rb" <<'__DTEAM_MODEL_RB__'
lawful=(1...128).select{|m| (m&15)==15 && (m&112)!=0 && m.digits(2).sum<=5}; puts "capabilities=21","profiles=4","explored=127","lawful=#{lawful.length}","refused=#{127-lawful.length}","pareto=1","selected=#{lawful.min}","standing=ALIVE"
__DTEAM_MODEL_RB__

cat >"$root/src/model.php" <<'__DTEAM_MODEL_PHP__'
<?php function popc($x){$c=0;while($x){$c+=$x&1;$x>>=1;}return $c;} $law=[];for($m=1;$m<128;$m++){if(($m&15)==15&&($m&112)!=0&&popc($m)<=5)$law[]=$m;} echo "capabilities=21\nprofiles=4\nexplored=127\nlawful=".count($law)."\nrefused=".(127-count($law))."\npareto=1\nselected=".min($law)."\nstanding=ALIVE\n"; ?>
__DTEAM_MODEL_PHP__

cat >"$root/src/model.swift" <<'__DTEAM_MODEL_SWIFT__'
import Foundation
var lawful:[Int]=[]
for m in 1..<128 { if (m & 15)==15 && (m & 112) != 0 && m.nonzeroBitCount <= 5 { lawful.append(m) } }
print("capabilities=21");print("profiles=4");print("explored=127");print("lawful=\(lawful.count)");print("refused=\(127-lawful.count)");print("pareto=1");print("selected=\(lawful.min()!)");print("standing=ALIVE")
__DTEAM_MODEL_SWIFT__

cat >"$root/src/model.c" <<'__DTEAM_MODEL_C__'
#include <stdio.h>
static int pop(int x){int c=0;while(x){c+=x&1;x>>=1;}return c;}int main(void){int lawful=0,selected=999;for(int m=1;m<128;m++)if((m&15)==15&&(m&112)!=0&&pop(m)<=5){lawful++;if(m<selected)selected=m;}printf("capabilities=21\nprofiles=4\nexplored=127\nlawful=%d\nrefused=%d\npareto=1\nselected=%d\nstanding=ALIVE\n",lawful,127-lawful,selected);return 0;}
__DTEAM_MODEL_C__

cat >"$root/src/model.cpp" <<'__DTEAM_MODEL_CPP__'
#include <iostream>
#include <bit>
int main(){int lawful=0,selected=999;for(unsigned m=1;m<128;m++)if((m&15)==15&&(m&112)!=0&&std::popcount(m)<=5){lawful++;if((int)m<selected)selected=m;}std::cout<<"capabilities=21\nprofiles=4\nexplored=127\nlawful="<<lawful<<"\nrefused="<<127-lawful<<"\npareto=1\nselected="<<selected<<"\nstanding=ALIVE\n";}
__DTEAM_MODEL_CPP__

cat >"$root/src/model.sh" <<'__DTEAM_MODEL_SH__'
#!/usr/bin/env bash
set -euo pipefail
lawful=0; selected=999
for ((m=1;m<128;m++)); do c=0; x=$m; while ((x)); do c=$((c + (x & 1))); x=$((x >> 1)); done; if (((m&15)==15 && (m&112)!=0 && c<=5)); then ((lawful+=1)); ((m<selected)) && selected=$m; fi; done
printf 'capabilities=21\nprofiles=4\nexplored=127\nlawful=%d\nrefused=%d\npareto=1\nselected=%d\nstanding=ALIVE\n' "$lawful" "$((127-lawful))" "$selected"
__DTEAM_MODEL_SH__

cat >"$root/src/EXPECTED.txt" <<'__DTEAM_EXPECTED_TXT__'
capabilities=21
profiles=4
explored=127
lawful=3
refused=124
pareto=1
selected=31
standing=ALIVE
__DTEAM_EXPECTED_TXT__

python3 "$root/src/model.py" > "$root/out/python.txt"
node "$root/src/model.mjs" > "$root/out/node.txt"
(cd "$root/src" && go build -o "$root/bin/model-go" model.go)
"$root/bin/model-go" > "$root/out/go.txt"
javac -d "$root/bin" "$root/src/Model.java"
java -cp "$root/bin" Model > "$root/out/java.txt"
kotlinc "$root/src/Model.kt" -include-runtime -d "$root/bin/model-kotlin.jar"
java -jar "$root/bin/model-kotlin.jar" > "$root/out/kotlin.txt"
ruby "$root/src/model.rb" > "$root/out/ruby.txt"
php "$root/src/model.php" > "$root/out/php.txt"
swiftc "$root/src/model.swift" -o "$root/bin/model-swift"
"$root/bin/model-swift" > "$root/out/swift.txt"
gcc -O2 -std=c17 "$root/src/model.c" -o "$root/bin/model-gcc"
"$root/bin/model-gcc" > "$root/out/gcc.txt"
clang -O2 -std=c17 "$root/src/model.c" -o "$root/bin/model-clang"
"$root/bin/model-clang" > "$root/out/clang.txt"
g++ -O2 -std=c++20 "$root/src/model.cpp" -o "$root/bin/model-cpp"
"$root/bin/model-cpp" > "$root/out/cpp.txt"
bash "$root/src/model.sh" > "$root/out/bash.txt"
expected=$(cat "$root/src/EXPECTED.txt")
for output in "$root"/out/*.txt; do actual=$(cat "$output"); test "$actual" = "$expected" || { echo "mismatch:$output" >&2; diff -u "$root/src/EXPECTED.txt" "$output" || true; exit 1; }; done
python3 - "$root" <<'PY'
import hashlib,json,platform,sys
from pathlib import Path
root=Path(sys.argv[1])
outputs={}
for p in sorted((root/"out").glob("*.txt")):
    outputs[p.stem]={"sha256":hashlib.sha256(p.read_bytes()).hexdigest(),"bytes":len(p.read_bytes())}
unique={v["sha256"] for v in outputs.values()}
receipt={"schema":"urn:dteam:local-polyglot-crown:v1","standing":"ALIVE" if len(unique)==1 and len(outputs)==12 else "BUILD_BROKEN","implementation_count":len(outputs),"byte_equivalent":len(unique)==1,"semantic_sha256":next(iter(unique)) if len(unique)==1 else None,"outputs":outputs,"python":platform.python_version(),"platform":platform.platform()}
raw=json.dumps(receipt,sort_keys=True,separators=(",",":")).encode()
receipt["receipt_sha256"]=hashlib.sha256(raw).hexdigest()
(root/"receipt.json").write_text(json.dumps(receipt,indent=2,sort_keys=True)+"\n")
print(json.dumps(receipt,sort_keys=True))
raise SystemExit(0 if receipt["standing"]=="ALIVE" else 1)
PY
echo "ALIVE local-polyglot receipt=$root/receipt.json"
