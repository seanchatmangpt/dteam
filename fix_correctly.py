import os
import re

def fix_file(path, words=1):
    with open(path, 'r') as f: content = f.read()
    
    # 1. Fix Imports
    content = content.replace("Vision2030Kernel<1>", "Vision2030Kernel")
    if "use crate::utils::dense_kernel::KBitSet;" not in content and "use dteam::utils::dense_kernel::KBitSet;" not in content:
        if "use crate::autonomic" in content:
            content = content.replace("use crate::autonomic", "use crate::utils::dense_kernel::KBitSet;\nuse crate::simd::SwarMarking;\nuse crate::autonomic")
        elif "use dteam::autonomic" in content:
            content = content.replace("use dteam::autonomic", "use dteam::utils::dense_kernel::KBitSet;\nuse dteam::simd::SwarMarking;\nuse dteam::autonomic")

    # 2. Fix instantiations
    content = re.sub(r'Vision2030Kernel::new\(\)', f'Vision2030Kernel::<{words}>::new()', content)
    content = re.sub(r'Vision2030Kernel::<1>::new\(\)', f'Vision2030Kernel::<{words}>::new()', content)
    
    # 3. Fix Type hints (if any)
    content = re.sub(r'Vision2030Kernel(?![ <])', f'Vision2030Kernel<{words}>', content)
    
    # 4. Fix field assignments
    content = content.replace("kernel.powl_executed_mask = KBitSet::zero();", f"kernel.powl_executed_mask = KBitSet::<{words}>::zero();")
    content = content.replace("kernel.powl_executed_mask = 0;", f"kernel.powl_executed_mask = KBitSet::<{words}>::zero();")
    content = content.replace("kernel.marking = SwarMarking::new(1);", f"kernel.marking = SwarMarking::new(1);")
    content = content.replace("kernel.marking.0 = 1;", f"kernel.marking = SwarMarking::new(1);")

    with open(path, 'w') as f: f.write(content)

fix_file("src/jtbd_counterfactual_tests.rs")
fix_file("src/jtbd_tests.rs")
fix_file("examples/self_play.rs")
fix_file("examples/dogfood.rs")
