import os
import subprocess

docs_dir = "../insa/docs"
tex_path = os.path.join(docs_dir, "insa_thesis.tex")

with open(tex_path, "r") as f:
    content = f.read()

# The book class does not support \begin{abstract} by default.
# We will use \chapter*{Abstract} instead.
content = content.replace(r"\begin{abstract}", r"\chapter*{Abstract}")
content = content.replace(r"\end{abstract}", "")

with open(tex_path, "w") as f:
    f.write(content)

subprocess.run(["pdflatex", "-interaction=nonstopmode", "insa_thesis.tex"], cwd=docs_dir)
# Run a second time for Table of Contents resolution
subprocess.run(["pdflatex", "-interaction=nonstopmode", "insa_thesis.tex"], cwd=docs_dir)
