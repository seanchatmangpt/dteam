# Makefile for dteam
# Targets: build, test, bench, doc, lint, fmt, check, doctor, doctor-artifacts, audit

.PHONY: build test bench doc clean lint fmt check doctor doctor-artifacts audit acceptance run

PYTHON ?= python3
AUDIT_DIR ?= artifacts/innovation-audit

build:
	cargo build --release

test:
	cargo test --lib -- --nocapture

bench:
	cargo bench

# Fail-loud repository and workspace preflight. Never suppresses the producing error.
doctor:
	$(PYTHON) tools/innovation_80_20.py doctor --output-dir "$(AUDIT_DIR)"

# Epistemic diagnosis of generated AutoML plans. Requires plan artifacts.
doctor-artifacts:
	cargo run --bin doctor -- --json

# Deterministic source audit and receipt without requiring the Rust dependency closure.
audit:
	$(PYTHON) tools/innovation_80_20.py audit --output-dir "$(AUDIT_DIR)"

# Runs mutation tests and proves two-pass deterministic audit replay.
acceptance:
	$(PYTHON) -m unittest -v tests.test_innovation_80_20
	$(PYTHON) tools/innovation_80_20.py replay --output-dir "$(AUDIT_DIR)"

# Start the autonomic live loop
run:
	cargo run --example autonomic_runner

lint:
	cargo clippy --all-targets -- -D warnings

fmt:
	cargo fmt --all

check:
	cargo check

doc:
	pdflatex -interaction=nonstopmode -halt-on-error -output-directory=docs/thesis docs/thesis/main.tex
	pdflatex -interaction=nonstopmode -halt-on-error -output-directory=docs/thesis docs/thesis/main.tex
	mv docs/thesis/main.pdf docs/thesis/dteam-whitepaper.pdf

clean:
	cargo clean
	rm -rf "$(AUDIT_DIR)"
	rm -f docs/thesis/*.aux docs/thesis/*.log docs/thesis/*.out docs/thesis/*.toc docs/thesis/*.pdf
