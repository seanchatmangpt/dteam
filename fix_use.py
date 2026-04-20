import os
import re

files = ["src/jtbd_tests.rs", "src/jtbd_counterfactual_tests.rs"]

for f_path in files:
    with open(f_path, 'r') as f:
        content = f.read()
    
    # Replace use crate::autonomic::{..., Vision2030Kernel<1>, ...} with use crate::autonomic::{..., Vision2030Kernel, ...}
    content = content.replace("Vision2030Kernel<1>", "Vision2030Kernel")
    
    with open(f_path, 'w') as f:
        f.write(content)
