test:
    cargo make ci

test-full:
    cargo make ci && cargo make boundary-tests

polish:
    cargo fmt --all && cargo clippy --workspace -- -D warnings

build:
    cargo build --workspace

clean:
    cargo clean

doc:
    cargo doc --workspace --no-deps

ci: polish test-full
