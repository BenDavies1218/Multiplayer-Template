# Installation

## Prerequisites

- **Rust 1.93+** (edition 2024)
- **Docker** (optional, for container deployment)
- **Trunk** (for web builds): `cargo install trunk`

## Clone the Repository

```bash
git clone https://github.com/BenDavies1218/Multiplayer-Template.git
cd Multiplayer-Template
```

## Build Everything

```bash
# Build all workspace crates and binaries
cargo build --all

# Build in release mode (production optimized)
cargo build --all --release
```

## Run Tests

```bash
cargo test --all
```

## Cargo Aliases

Fast development builds with dynamic linking are available via `.cargo/config.toml`:

```bash
cargo dev-native   # Run native client with dynamic linking (client -c 1)
cargo dev-server   # Run server with dynamic linking
cargo dev-viewer   # Run world viewer with dynamic linking
```

## Adding Dependencies

Add dependencies to the workspace root `Cargo.toml`:

```toml
[workspace.dependencies]
your-crate = "1.0"
```

Then reference in individual crates:

```toml
[dependencies]
your-crate.workspace = true
```
