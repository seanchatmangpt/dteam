import os
import re

files = ["src/jtbd_counterfactual_tests.rs", "examples/self_play.rs"]

for f_path in files:
    with open(f_path, 'r') as f:
        lines = f.readlines()
    
    seen = set()
    new_lines = []
    for line in lines:
        if line.strip().startswith("use ") and line.strip() in seen:
            continue
        if line.strip().startswith("use "):
            seen.add(line.strip())
        new_lines.append(line)
    
    with open(f_path, 'w') as f:
        f.writelines(new_lines)

