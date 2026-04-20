import os
import re

def fix_file(path, words=1):
    with open(path, 'r') as f: content = f.read()
    
    # Imports
    if "use dteam::utils::dense_kernel::KBitSet;" not in content:
        content = content.replace("use dteam::powl::core::PowlModel;", "use dteam::powl::core::PowlModel;\nuse dteam::utils::dense_kernel::KBitSet;")
    if "use dteam::simd::SwarMarking;" not in content:
        content = content.replace("use dteam::powl::core::PowlModel;", "use dteam::powl::core::PowlModel;\nuse dteam::simd::SwarMarking;")

    # Fix Vision2030Kernel instantiation and types
    content = re.sub(r'Vision2030Kernel<1>::<1>', 'Vision2030Kernel<1>', content)
    content = re.sub(r'Vision2030Kernel<1>(?![ <])', f'Vision2030Kernel<{words}>', content)
    content = re.sub(r'Vision2030Kernel(?![ <])', f'Vision2030Kernel<{words}>', content)
    
    # Fix KBitSet and SwarMarking
    content = content.replace("kernel.powl_executed_mask = 0;", f"kernel.powl_executed_mask = KBitSet::<{words}>::zero();")
    content = content.replace("kernel.marking.0 = 1;", f"kernel.marking = SwarMarking::new(1);")
    
    with open(path, 'w') as f: f.write(content)

fix_file("src/jtbd_counterfactual_tests.rs")
fix_file("src/jtbd_tests.rs")
fix_file("examples/self_play.rs")
fix_file("examples/dogfood.rs")
