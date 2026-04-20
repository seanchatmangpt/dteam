import os
import re

files = ["examples/dogfood.rs", "examples/self_play.rs"]

for f_path in files:
    with open(f_path, 'r') as f:
        content = f.read()
    
    # Fix import
    content = content.replace("Vision2030Kernel<1>", "Vision2030Kernel")
    
    with open(f_path, 'w') as f:
        f.write(content)
