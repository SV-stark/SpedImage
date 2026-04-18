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

# Run cargo deny to check licenses and vulnerabilities
deny:
    cargo deny --version >$null 2>&1 || cargo install cargo-deny
    cargo deny check

# Check conventional commits
cog-check:
    cog check

# Full check (fmt, lint, deny, test, cog)
check-all: fmt lint deny test cog-check

# Generate changelog
changelog:
    git cliff -o CHANGELOG.md

# Build installer
installer: build-release
    makensis installer.nsi
