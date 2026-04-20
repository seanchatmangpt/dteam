path = "src/bcinr_extended/bclr_u64.rs"
with open(path, 'r') as f: content = f.read()
# count braces
open_braces = content.count('{')
close_braces = content.count('}')
if close_braces > open_braces:
    content = content.rsplit('}', close_braces - open_braces)[0] + '}'
with open(path, 'w') as f: f.write(content)
