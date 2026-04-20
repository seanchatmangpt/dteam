import os
import re

def fix_file(path):
    with open(path, 'r') as f: content = f.read()
    
    # 1. Vision2030Kernel -> Vision2030Kernel<1>
    content = re.sub(r'Vision2030Kernel(?![ <])', 'Vision2030Kernel<1>', content)
    
    # 2. kernel.powl_executed_mask = 0 -> kernel.powl_executed_mask = KBitSet::zero()
    content = content.replace("kernel.powl_executed_mask = 0;", "kernel.powl_executed_mask = KBitSet::zero();")
    
    # 3. kernel.marking.0 = 1 -> kernel.marking = SwarMarking::new(1)
    content = content.replace("kernel.marking.0 = 1;", "kernel.marking = SwarMarking::new(1);")
    
    with open(path, 'w') as f: f.write(content)

fix_file("src/jtbd_counterfactual_tests.rs")
fix_file("src/jtbd_tests.rs")
fix_file("examples/self_play.rs")
