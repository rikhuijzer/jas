default:
    just --list

# Run the Rust harness against the standalone Bash script.
test:
    cargo test --locked

# Check Bash and test-harness formatting.
check:
    bash -n jas
    shellcheck jas
    cargo fmt --all -- --check
    cargo clippy --tests --locked -- -D warnings
