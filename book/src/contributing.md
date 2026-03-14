# Contributing to Multiplayer-Template

Thank you for your interest in contributing!

## Development Setup

1. **Clone the repository**:
   ```bash
   git clone https://github.com/BenDavies1218/Multiplayer-Template.git
   cd Multiplayer-Template
   ```

2. **Install dependencies**:
   - Rust 1.93+
   - Docker (for testing containers)
   - Trunk: `cargo install trunk`

3. **Build and test**:
   ```bash
   cargo build --all
   cargo test --all
   ```

## Workflow

1. Fork the repository
2. Create a feature branch: `git checkout -b feature/amazing-feature`
3. Make your changes
4. Run tests: `cargo test --all`
5. Run formatter: `cargo fmt --all`
6. Run clippy: `cargo clippy --all`
7. Commit with clear message
8. Push to your fork
9. Open a Pull Request

## Code Style

- Follow Rust conventions
- Run `cargo fmt` before committing
- Fix all `cargo clippy` warnings
- Add tests for new features
- Update documentation

## Pull Request Process

1. **Ensure CI passes** (tests, formatting, clippy)
2. **Update documentation** if needed
3. **Describe your changes** clearly in PR description
4. **Link related issues** if applicable
5. Wait for review from maintainers

## Running CI Locally

Before pushing, run these checks:

```bash
# Format check
cargo fmt --all -- --check

# Linting
cargo clippy --all -- -D warnings

# Tests
cargo test --all

# Build all targets
cargo build --all
```

## Testing Docker Builds

```bash
# Build server image
docker build -f Dockerfile.server -t multiplayer-server .

# Build web image
docker build -f Dockerfile.web -t multiplayer-web .

# Test with docker-compose
docker-compose up
```

## Questions?

Open an issue or discussion on GitHub.
