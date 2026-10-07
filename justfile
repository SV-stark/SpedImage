set shell := ["pwsh.exe", "-NoProfile", "-Command"]
set unstable := true

# Show available commands
default:
    @just --list

# Format code
fmt:
    cargo fmt --all
    just --fmt

# Lint code using clippy
alias clippy := lint
lint:
    cargo clippy --workspace --all-targets -- -D warnings

# Run tests using cargo-nextest (fallback to cargo test if not installed)
test:
    cargo nextest run || cargo test

# Run benchmarks using divan
bench:
    cargo bench

# Build the application in release mode
build-release:
    cargo build --release

# Check for unused dependencies
machete:
    cargo machete

# Build release and assert the binary stays inside the README's size budget
size-check: build-release
    pwsh -File scripts/check_size.ps1 -Path target/release/spedimage.exe

# Check conventional commits
cog-check:
    cog check

# Full check (fmt, lint, test, cog)
check-all: fmt lint test cog-check

# Everything CI runs, plus the size budget gate
check-ci: fmt lint test size-check

# Generate changelog
changelog:
    git cliff -o CHANGELOG.md

# Build installer
installer: build-release
    makensis installer.nsi
