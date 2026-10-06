set shell := ["bash", "-eu", "-o", "pipefail", "-c"]

default: check

# Check formatting and whitespace without compiling.
check: check-just check-format check-whitespace

check-just:
    {{ quote(just_executable()) }} --fmt --check

check-format:
    cargo fmt --all -- --check

check-whitespace:
    git diff --check
    git diff --cached --check

format:
    {{ quote(just_executable()) }} --fmt
    cargo fmt --all

lint: check clippy

clippy:
    cargo clippy --locked --all-targets --features setup

# GTK tests require a display; CI supplies a virtual display.
test: check unit-tests

unit-tests:
    cargo test --locked --features setup

# Build the self-contained Linux binary, including embedded resources.
build: check package

package:
    cargo build --package bank-files --locked --release --features setup

ci: lint test build

run:
    cargo run --locked --features setup
