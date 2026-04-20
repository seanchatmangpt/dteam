import os
import re

def fix_file(path, words=1):
    with open(path, 'r') as f: content = f.read()
    
    # Imports
    content = content.replace("Vision2030Kernel<1>", "Vision2030Kernel")
    if "use crate::utils::dense_kernel::KBitSet;" not in content:
        content = content.replace("use crate::simd::SwarMarking;", "use crate::simd::SwarMarking;\n    use crate::utils::dense_kernel::KBitSet;")
    
    # Types
    content = re.sub(r'Vision2030Kernel(?![ <])', f'Vision2030Kernel<{words}>', content)
    
    # Instantiation
    content = content.replace("Vision2030Kernel::<1>::new()", f"Vision2030Kernel::<{words}>::new()")
    content = re.sub(r'Vision2030Kernel::new\(\)', f'Vision2030Kernel::<{words}>::new()', content)
    
    # Cleanup duplication
    content = content.replace(f"Vision2030Kernel<{words}>::<{words}>::new()", f"Vision2030Kernel::<{words}>::new()")

    with open(path, 'w') as f: f.write(content)

fix_file("src/jtbd_tests.rs")
fix_file("src/jtbd_counterfactual_tests.rs")
