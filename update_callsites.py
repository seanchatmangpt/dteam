import os
import re

files = ["src/jtbd_tests.rs", "src/jtbd_counterfactual_tests.rs", "examples/self_play.rs"]

for f_path in files:
    with open(f_path, 'r') as f:
        content = f.read()
    
    # 1. Update Vision2030Kernel instantiation
    content = content.replace("Vision2030Kernel::new()", "Vision2030Kernel::<1>::new()")
    content = content.replace("let mut kernel = Vision2030Kernel::new();", "let mut kernel = Vision2030Kernel::<1>::new();")
    
    # 2. Update Vision2030Kernel type hint
    content = content.replace("(Vision2030Kernel, Vec<AutonomicResult>)", "(Vision2030Kernel<1>, Vec<AutonomicResult>)")
    content = content.replace("Vision2030Kernel", "Vision2030Kernel<1>")
    
    with open(f_path, 'w') as f:
        f.write(content)
