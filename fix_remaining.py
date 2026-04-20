import os

def fix_bclr():
    path = "src/bcinr_extended/bclr_u64.rs"
    if not os.path.exists(path): return
    with open(path, 'r') as f: content = f.read()
    content = content.replace("} } res", "} } res }")
    with open(path, 'w') as f: f.write(content)

def fix_signum():
    path = "src/bcinr_extended/branchless_signum_i64.rs"
    if not os.path.exists(path): return
    with open(path, 'r') as f: content = f.read()
    content = content.replace("(if (val as i64) > 0 { 1 }", "if (val as i64) > 0 { 1 }")
    content = content.replace("else { 0 })", "else { 0 }")
    with open(path, 'w') as f: f.write(content)

fix_bclr()
fix_signum()
