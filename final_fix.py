import os
import glob
import re

def fix_file(file_path):
    with open(file_path, 'r') as f:
        content = f.read()
    
    # 1. Fix the unclosed loop in reference functions if it exists
    # Find something like: let mut res = val; for i in 0..len { if start + i < 64 { res &= !(1 << (start + i)); }
    # and add missing closing braces
    if "for i in 0..len { if start + i < 64 { res &= !(1 << (start + i)); }" in content:
        content = content.replace("for i in 0..len { if start + i < 64 { res &= !(1 << (start + i)); }", 
                                  "for i in 0..len { if start + i < 64 { res &= !(1 << (start + i)); } } res")
        # Ensure it ends with a brace
        if not content.strip().endswith("}"):
             content += "\n    }\n}\n"

    # 2. Count braces and add missing ones at the end of the file
    open_braces = content.count('{')
    close_braces = content.count('}')
    
    if open_braces > close_braces:
        content += "\n" + ("}" * (open_braces - close_braces))
        
    with open(file_path, 'w') as f:
        f.write(content)

files = glob.glob("src/bcinr_extended/*.rs")
for f in files:
    if "mod.rs" in f: continue
    fix_file(f)
