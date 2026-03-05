# CI/CD Implementation Guide for Multiplayer-Template

This document provides step-by-step instructions for implementing CI/CD, Docker containerization, and renaming the repository to "Multiplayer-Template".

## Table of Contents

1. [Overview](#overview)
2. [Prerequisites](#prerequisites)
3. [Phase 1: Repository Preparation](#phase-1-repository-preparation)
4. [Phase 2: Docker Setup](#phase-2-docker-setup)
5. [Phase 3: GitHub Actions Workflows](#phase-3-github-actions-workflows)
6. [Phase 4: Additional Configuration](#phase-4-additional-configuration)
7. [Phase 5: Local Testing](#phase-5-local-testing)
8. [Phase 6: GitHub Repository Setup](#phase-6-github-repository-setup)
9. [Phase 7: Verification](#phase-7-verification)
10. [Implementation Checklist](#implementation-checklist)
11. [Troubleshooting](#troubleshooting)

---

## Overview

**Goal**: Set up a complete CI/CD pipeline with Docker containerization and rename the repository to "Multiplayer-Template" for easy forking and reuse.

**What we'll build**:
- Docker containers for server and web client
- GitHub Actions for automated testing, building, and deployment
- Auto-deployment of web client to GitHub Pages
- Docker images published to GitHub Container Registry
- Automated release builds for all platforms

**Timeline**: ~2-3 hours for complete implementation

---

## Prerequisites

### Required Tools
- [x] Git
- [x] Rust 1.88+
- [x] Docker Desktop (or Docker Engine + Docker Compose)
- [x] GitHub account
- [ ] Trunk (install: `cargo install trunk`)

### Optional Tools
- GitHub CLI (`gh`) for easier repository creation
- Docker Buildx for multi-platform builds

### Required Knowledge
- Basic Docker understanding
- Basic GitHub Actions understanding
- Cargo/Rust build system

---

## Phase 1: Repository Preparation

### Step 1.1: Update Workspace Metadata

**File**: `Cargo.toml` (root)

Update the workspace package metadata:

```toml
[workspace.package]
version = "0.1.0"  # Changed from 0.26.4 (fresh template start)
edition = "2024"
rust-version = "1.88"
authors = ["Your Name <your.email@example.com>"]  # Update with your info
license = "MIT OR Apache-2.0"
repository = "https://github.com/YOUR_USERNAME/Multiplayer-Template"  # Add this line
```

**Command**:
```bash
# Edit Cargo.toml manually or use sed
sed -i '' 's/version = "0.26.4"/version = "0.1.0"/' Cargo.toml
```

### Step 1.2: Update Root README.md

**File**: `README.md` (root)

Changes:
1. Title: Change to "# Multiplayer-Template"
2. Add subtitle: "A production-ready multiplayer game template using Bevy + Lightyear"
3. Update repository references
4. Add CI/CD badges section (update after GitHub repo creation)
5. Add Docker deployment section

**Example Badge Section** (add at top after title):
```markdown
[![CI](https://github.com/YOUR_USERNAME/Multiplayer-Template/workflows/CI/badge.svg)](https://github.com/YOUR_USERNAME/Multiplayer-Template/actions)
[![Docker Server](https://github.com/YOUR_USERNAME/Multiplayer-Template/workflows/Docker%20Server/badge.svg)](https://github.com/YOUR_USERNAME/Multiplayer-Template/actions)
[![Deploy Web](https://github.com/YOUR_USERNAME/Multiplayer-Template/workflows/Deploy%20Web/badge.svg)](https://github.com/YOUR_USERNAME/Multiplayer-Template/actions)
```

**Example Docker Section** (add after Quick Start):
```markdown
## Docker Deployment

### Using Docker Compose (Recommended)

```bash
docker-compose up -d
```

This starts:
- Server on port 5000
- Web client on port 8080

### Using Pre-built Images

```bash
# Pull and run server
docker pull ghcr.io/YOUR_USERNAME/multiplayer-template-server:latest
docker run -p 5000:5000 -v ./certificates:/certificates ghcr.io/YOUR_USERNAME/multiplayer-template-server:latest

# Pull and run web client
docker pull ghcr.io/YOUR_USERNAME/multiplayer-template-web:latest
docker run -p 8080:80 ghcr.io/YOUR_USERNAME/multiplayer-template-web:latest
```

See [DEPLOYMENT.md](DEPLOYMENT.md) for production deployment guide.
```

### Step 1.3: Update App READMEs

Update repository links in:
- `apps/server/README.md`
- `apps/native/README.md`
- `apps/web/README.md`

Search and replace: `multiplayer-bevy` → `Multiplayer-Template`

**Command**:
```bash
# Update all app READMEs
find apps -name "README.md" -exec sed -i '' 's/multiplayer-bevy/Multiplayer-Template/g' {} \;
```

### Step 1.4: Create Environment Variables Configuration

**File**: `.env.example` (root)

Create a template for environment variables:

```bash
# ==============================================
# Multiplayer Template - Environment Variables
# ==============================================

# ============= Server Configuration =============
# Server address (0.0.0.0 for Docker, 127.0.0.1 for local)
SERVER_HOST=127.0.0.1
SERVER_PORT=5000

# Transport protocol: udp, websocket, webtransport
TRANSPORT_TYPE=webtransport

# ============= Network Configuration =============
# Fixed timestep frequency (Hz)
FIXED_TIMESTEP_HZ=64

# Send interval frequency (Hz)
SEND_INTERVAL_HZ=64

# Client timeout (seconds)
CLIENT_TIMEOUT_SECS=3

# ============= Logging Configuration =============
# Log level: error, warn, info, debug, trace
RUST_LOG=info

# ============= Certificate Configuration =============
# Paths for WebTransport certificates
CERT_PATH=./certificates/cert.pem
KEY_PATH=./certificates/key.pem
DIGEST_PATH=./certificates/digest.txt

# ============= Database Configuration =============
# Database URL (for future use when adding persistence)
# Example: postgresql://user:password@localhost:5432/multiplayer_game
# DATABASE_URL=

# ============= Client Configuration =============
# Client interpolation buffer (milliseconds)
INTERPOLATION_BUFFER_MS=100

# Client input delay (frames, 0 for instant)
INPUT_DELAY_FRAMES=0

# ============= Development Configuration =============
# Enable debug mode
DEBUG_MODE=false

# Enable physics debug rendering
PHYSICS_DEBUG=false

# Network conditioner (for testing)
# NETWORK_LATENCY_MS=50
# NETWORK_JITTER_MS=10
# NETWORK_LOSS_PERCENT=0

# ============= Production Configuration =============
# Enable production optimizations
PRODUCTION=false

# Max concurrent connections
MAX_CONNECTIONS=100

# Rate limiting (requests per second per client)
RATE_LIMIT=60
```

**Note**: Add `.env` to `.gitignore` (it should already be there, but verify):

```bash
echo ".env" >> .gitignore
```

### Step 1.5: Clean Up Old Files

Remove the standalone directory:

```bash
rm -rf standalone_avian_3d_character/
```

Update `.gitignore` to remove standalone references:

```bash
# Remove these lines from .gitignore:
# Old standalone directory (can be removed after migration verified)
standalone_avian_3d_character/target/
standalone_avian_3d_character/dist/
```

---

## Phase 2: Docker Setup

### Step 2.1: Create Dockerfile.server

**File**: `Dockerfile.server` (root)

```dockerfile
# Multi-stage build for production server binary

# Build stage
FROM rust:1.88-bookworm as builder

WORKDIR /app

# Copy workspace files
COPY Cargo.toml Cargo.lock ./
COPY crates ./crates
COPY apps ./apps

# Build release binary
RUN cargo build --release -p server

# Runtime stage - minimal image
FROM debian:bookworm-slim

# Install runtime dependencies
RUN apt-get update && \
    apt-get install -y ca-certificates && \
    rm -rf /var/lib/apt/lists/*

# Copy binary from builder
COPY --from=builder /app/target/release/server /usr/local/bin/server

# Copy certificates (required for WebTransport)
COPY certificates /certificates

# Expose game server port
EXPOSE 5000

# Set working directory
WORKDIR /

# Run server
CMD ["server", "server"]
```

### Step 2.2: Create Dockerfile.web

**File**: `Dockerfile.web` (root)

```dockerfile
# Multi-stage build for WASM web client

# Build stage
FROM rust:1.88-bookworm as builder

# Install trunk for WASM builds
RUN cargo install --locked trunk

# Install wasm target
RUN rustup target add wasm32-unknown-unknown

WORKDIR /app

# Copy workspace files
COPY Cargo.toml Cargo.lock ./
COPY crates ./crates
COPY apps ./apps
COPY certificates ./certificates

# Build web client
WORKDIR /app/apps/web
RUN trunk build --release

# Runtime stage - nginx to serve static files
FROM nginx:alpine

# Copy built files from builder
COPY --from=builder /app/apps/web/dist /usr/share/nginx/html

# Copy custom nginx config (optional)
# COPY nginx.conf /etc/nginx/nginx.conf

# Expose HTTP port
EXPOSE 80

# Nginx runs automatically
```

### Step 2.3: Create docker-compose.yml

**File**: `docker-compose.yml` (root)

```yaml
version: '3.8'

services:
  server:
    build:
      context: .
      dockerfile: Dockerfile.server
    container_name: multiplayer-server
    ports:
      - "${SERVER_PORT:-5000}:5000"
    volumes:
      - ./certificates:/certificates:ro
    env_file:
      - .env  # Load environment variables from .env file
    environment:
      - RUST_LOG=${RUST_LOG:-info}
      - SERVER_HOST=0.0.0.0  # Listen on all interfaces in Docker
      - SERVER_PORT=${SERVER_PORT:-5000}
    restart: unless-stopped
    networks:
      - game-network
    healthcheck:
      test: ["CMD", "nc", "-z", "localhost", "5000"]
      interval: 30s
      timeout: 10s
      retries: 3
      start_period: 40s

  web:
    build:
      context: .
      dockerfile: Dockerfile.web
    container_name: multiplayer-web
    ports:
      - "8080:80"
    depends_on:
      server:
        condition: service_healthy
    restart: unless-stopped
    networks:
      - game-network
    healthcheck:
      test: ["CMD", "wget", "--quiet", "--tries=1", "--spider", "http://localhost:80"]
      interval: 30s
      timeout: 10s
      retries: 3

networks:
  game-network:
    driver: bridge

volumes:
  certificates:
```

**Note**: Copy `.env.example` to `.env` before running:

```bash
cp .env.example .env
# Edit .env with your specific configuration
```

### Step 2.4: Create .dockerignore

**File**: `.dockerignore` (root)

```
# Rust build artifacts
target/
**/target/
**/*.rs.bk
Cargo.lock

# Trunk build artifacts
dist/
**/dist/

# Environment files (don't include in image)
.env
.env.*
!.env.example

# Git
.git/
.gitignore
.gitattributes

# IDE
.vscode/
.idea/
*.swp
*.swo
*~

# OS
.DS_Store
Thumbs.db

# Logs
*.log

# CI/CD
.github/

# Documentation (not needed in container)
*.md
!README.md

# Old standalone
standalone_avian_3d_character/
```

---

## Phase 3: GitHub Actions Workflows

Create `.github/workflows/` directory:

```bash
mkdir -p .github/workflows
```

### Step 3.1: CI Workflow (Testing & Linting)

**File**: `.github/workflows/ci.yml`

```yaml
name: CI

on:
  push:
    branches: [ main, develop ]
  pull_request:
    branches: [ main ]

env:
  CARGO_TERM_COLOR: always

jobs:
  check:
    name: Check, Test & Lint
    runs-on: ${{ matrix.os }}
    strategy:
      matrix:
        os: [ubuntu-latest, macos-latest, windows-latest]
        rust: [1.88]

    steps:
      - name: Checkout code
        uses: actions/checkout@v4

      - name: Setup Rust
        uses: dtolnay/rust-toolchain@stable
        with:
          toolchain: ${{ matrix.rust }}
          components: rustfmt, clippy

      - name: Cache cargo registry
        uses: actions/cache@v4
        with:
          path: ~/.cargo/registry
          key: ${{ runner.os }}-cargo-registry-${{ hashFiles('**/Cargo.lock') }}

      - name: Cache cargo index
        uses: actions/cache@v4
        with:
          path: ~/.cargo/git
          key: ${{ runner.os }}-cargo-index-${{ hashFiles('**/Cargo.lock') }}

      - name: Cache cargo build
        uses: actions/cache@v4
        with:
          path: target
          key: ${{ runner.os }}-cargo-build-target-${{ hashFiles('**/Cargo.lock') }}

      - name: Check formatting
        run: cargo fmt --all -- --check

      - name: Run clippy
        run: cargo clippy --all --all-features -- -D warnings

      - name: Run tests
        run: cargo test --all --verbose

      - name: Build all targets
        run: cargo build --all --verbose

  build-wasm:
    name: Build WASM
    runs-on: ubuntu-latest

    steps:
      - name: Checkout code
        uses: actions/checkout@v4

      - name: Setup Rust
        uses: dtolnay/rust-toolchain@stable
        with:
          targets: wasm32-unknown-unknown

      - name: Install Trunk
        run: cargo install --locked trunk

      - name: Cache cargo
        uses: actions/cache@v4
        with:
          path: |
            ~/.cargo/registry
            ~/.cargo/git
            target
          key: ${{ runner.os }}-cargo-wasm-${{ hashFiles('**/Cargo.lock') }}

      - name: Build web client
        working-directory: apps/web
        run: trunk build --release

      - name: Upload web artifact
        uses: actions/upload-artifact@v4
        with:
          name: web-client
          path: apps/web/dist/
          retention-days: 7
```

### Step 3.2: Docker Server Build & Push

**File**: `.github/workflows/docker-server.yml`

```yaml
name: Docker Server

on:
  push:
    branches: [ main ]
    tags: [ 'v*' ]
  workflow_dispatch:

env:
  REGISTRY: ghcr.io
  IMAGE_NAME: ${{ github.repository }}-server

jobs:
  build-and-push:
    name: Build and Push Server Image
    runs-on: ubuntu-latest
    permissions:
      contents: read
      packages: write

    steps:
      - name: Checkout code
        uses: actions/checkout@v4

      - name: Set up Docker Buildx
        uses: docker/setup-buildx-action@v3

      - name: Log in to Container Registry
        uses: docker/login-action@v3
        with:
          registry: ${{ env.REGISTRY }}
          username: ${{ github.actor }}
          password: ${{ secrets.GITHUB_TOKEN }}

      - name: Extract metadata
        id: meta
        uses: docker/metadata-action@v5
        with:
          images: ${{ env.REGISTRY }}/${{ env.IMAGE_NAME }}
          tags: |
            type=ref,event=branch
            type=ref,event=pr
            type=semver,pattern={{version}}
            type=semver,pattern={{major}}.{{minor}}
            type=sha,prefix={{branch}}-
            type=raw,value=latest,enable={{is_default_branch}}

      - name: Build and push
        uses: docker/build-push-action@v5
        with:
          context: .
          file: ./Dockerfile.server
          platforms: linux/amd64,linux/arm64
          push: true
          tags: ${{ steps.meta.outputs.tags }}
          labels: ${{ steps.meta.outputs.labels }}
          cache-from: type=gha
          cache-to: type=gha,mode=max

      - name: Image digest
        run: echo ${{ steps.meta.outputs.digest }}
```

### Step 3.3: Deploy Web Client

**File**: `.github/workflows/deploy-web.yml`

```yaml
name: Deploy Web

on:
  push:
    branches: [ main ]
  workflow_dispatch:

permissions:
  contents: read
  pages: write
  id-token: write

concurrency:
  group: "pages"
  cancel-in-progress: false

jobs:
  build:
    name: Build Web Client
    runs-on: ubuntu-latest

    steps:
      - name: Checkout code
        uses: actions/checkout@v4

      - name: Setup Rust
        uses: dtolnay/rust-toolchain@stable
        with:
          targets: wasm32-unknown-unknown

      - name: Install Trunk
        run: cargo install --locked trunk

      - name: Cache cargo
        uses: actions/cache@v4
        with:
          path: |
            ~/.cargo/registry
            ~/.cargo/git
            target
          key: ${{ runner.os }}-cargo-wasm-deploy-${{ hashFiles('**/Cargo.lock') }}

      - name: Build web client
        working-directory: apps/web
        run: trunk build --release

      - name: Setup Pages
        uses: actions/configure-pages@v4

      - name: Upload artifact
        uses: actions/upload-pages-artifact@v3
        with:
          path: apps/web/dist

  deploy:
    name: Deploy to GitHub Pages
    environment:
      name: github-pages
      url: ${{ steps.deployment.outputs.page_url }}
    runs-on: ubuntu-latest
    needs: build

    steps:
      - name: Deploy to GitHub Pages
        id: deployment
        uses: actions/deploy-pages@v4
```

### Step 3.4: Release Workflow

**File**: `.github/workflows/release.yml`

```yaml
name: Release

on:
  push:
    tags:
      - 'v*'

env:
  CARGO_TERM_COLOR: always

jobs:
  build-binaries:
    name: Build Release Binaries
    runs-on: ${{ matrix.os }}
    strategy:
      matrix:
        include:
          - os: ubuntu-latest
            target: x86_64-unknown-linux-gnu
            suffix: ''
          - os: macos-latest
            target: x86_64-apple-darwin
            suffix: ''
          - os: windows-latest
            target: x86_64-pc-windows-msvc
            suffix: .exe

    steps:
      - name: Checkout code
        uses: actions/checkout@v4

      - name: Setup Rust
        uses: dtolnay/rust-toolchain@stable
        with:
          targets: ${{ matrix.target }}

      - name: Cache cargo
        uses: actions/cache@v4
        with:
          path: |
            ~/.cargo/registry
            ~/.cargo/git
            target
          key: ${{ runner.os }}-cargo-release-${{ hashFiles('**/Cargo.lock') }}

      - name: Build server
        run: cargo build --release -p server --target ${{ matrix.target }}

      - name: Build native client
        run: cargo build --release -p native --target ${{ matrix.target }}

      - name: Package binaries (Unix)
        if: matrix.os != 'windows-latest'
        run: |
          mkdir -p release
          cp target/${{ matrix.target }}/release/server release/server-${{ matrix.target }}${{ matrix.suffix }}
          cp target/${{ matrix.target }}/release/native release/native-client-${{ matrix.target }}${{ matrix.suffix }}
          tar -czf multiplayer-template-${{ matrix.target }}.tar.gz -C release .

      - name: Package binaries (Windows)
        if: matrix.os == 'windows-latest'
        run: |
          mkdir release
          cp target/${{ matrix.target }}/release/server${{ matrix.suffix }} release/server-${{ matrix.target }}${{ matrix.suffix }}
          cp target/${{ matrix.target }}/release/native${{ matrix.suffix }} release/native-client-${{ matrix.target }}${{ matrix.suffix }}
          Compress-Archive -Path release/* -DestinationPath multiplayer-template-${{ matrix.target }}.zip

      - name: Upload artifacts (Unix)
        if: matrix.os != 'windows-latest'
        uses: actions/upload-artifact@v4
        with:
          name: binaries-${{ matrix.target }}
          path: multiplayer-template-${{ matrix.target }}.tar.gz

      - name: Upload artifacts (Windows)
        if: matrix.os == 'windows-latest'
        uses: actions/upload-artifact@v4
        with:
          name: binaries-${{ matrix.target }}
          path: multiplayer-template-${{ matrix.target }}.zip

  build-web:
    name: Build Web Client
    runs-on: ubuntu-latest

    steps:
      - name: Checkout code
        uses: actions/checkout@v4

      - name: Setup Rust
        uses: dtolnay/rust-toolchain@stable
        with:
          targets: wasm32-unknown-unknown

      - name: Install Trunk
        run: cargo install --locked trunk

      - name: Build web client
        working-directory: apps/web
        run: trunk build --release

      - name: Package web client
        run: |
          cd apps/web
          tar -czf ../../multiplayer-template-web.tar.gz dist/

      - name: Upload web artifact
        uses: actions/upload-artifact@v4
        with:
          name: web-client
          path: multiplayer-template-web.tar.gz

  create-release:
    name: Create GitHub Release
    needs: [build-binaries, build-web]
    runs-on: ubuntu-latest
    permissions:
      contents: write

    steps:
      - name: Checkout code
        uses: actions/checkout@v4

      - name: Download all artifacts
        uses: actions/download-artifact@v4
        with:
          path: artifacts

      - name: Create Release
        uses: softprops/action-gh-release@v1
        with:
          files: artifacts/**/*
          generate_release_notes: true
          draft: false
          prerelease: false
```

---

## Phase 4: Additional Configuration

### Step 4.1: Dependabot Configuration

**File**: `.github/dependabot.yml`

```yaml
version: 2
updates:
  # Cargo dependencies
  - package-ecosystem: "cargo"
    directory: "/"
    schedule:
      interval: "weekly"
    open-pull-requests-limit: 10
    groups:
      bevy:
        patterns:
          - "bevy*"
      lightyear:
        patterns:
          - "lightyear*"

  # GitHub Actions
  - package-ecosystem: "github-actions"
    directory: "/"
    schedule:
      interval: "monthly"
    open-pull-requests-limit: 5
```

### Step 4.2: Deployment Documentation

**File**: `DEPLOYMENT.md`

```markdown
# Deployment Guide

This guide covers deploying Multiplayer-Template to production environments.

## Docker Deployment

### Prerequisites

- Docker 20.10+
- Docker Compose 2.0+ (optional)
- Valid SSL/TLS certificates for production

### Quick Start with Docker Compose

1. **Generate production certificates** (if using WebTransport):

\`\`\`bash
# Generate production certificates with your domain
openssl req -x509 -newkey rsa:4096 -keyout certificates/key.pem \
    -out certificates/cert.pem -sha256 -days 365 -nodes \
    -subj "/CN=yourdomain.com"

# Generate digest for web clients
openssl x509 -in certificates/cert.pem -outform der | \
    openssl dgst -sha256 -binary | base64 > certificates/digest.txt
\`\`\`

2. **Pull and run with Docker Compose**:

\`\`\`bash
docker-compose up -d
\`\`\`

3. **Access**:
   - Server: Port 5000
   - Web Client: http://localhost:8080

### Using Pre-built Images

#### Server

\`\`\`bash
docker pull ghcr.io/YOUR_USERNAME/multiplayer-template-server:latest

docker run -d \
  --name multiplayer-server \
  -p 5000:5000 \
  -v $(pwd)/certificates:/certificates:ro \
  -e RUST_LOG=info \
  --restart unless-stopped \
  ghcr.io/YOUR_USERNAME/multiplayer-template-server:latest
\`\`\`

#### Web Client

\`\`\`bash
docker pull ghcr.io/YOUR_USERNAME/multiplayer-template-web:latest

docker run -d \
  --name multiplayer-web \
  -p 8080:80 \
  --restart unless-stopped \
  ghcr.io/YOUR_USERNAME/multiplayer-template-web:latest
\`\`\`

## Environment Variables

### Configuration File

Copy `.env.example` to `.env` and customize:

\`\`\`bash
cp .env.example .env
\`\`\`

### Available Variables

#### Server Configuration

| Variable | Description | Default | Required |
|----------|-------------|---------|----------|
| `SERVER_HOST` | Server bind address | `127.0.0.1` | No |
| `SERVER_PORT` | Server port | `5000` | No |
| `TRANSPORT_TYPE` | Protocol (udp, websocket, webtransport) | `webtransport` | No |

#### Network Configuration

| Variable | Description | Default | Required |
|----------|-------------|---------|----------|
| `FIXED_TIMESTEP_HZ` | Physics tick rate | `64` | No |
| `SEND_INTERVAL_HZ` | Network send rate | `64` | No |
| `CLIENT_TIMEOUT_SECS` | Client disconnect timeout | `3` | No |

#### Logging

| Variable | Description | Default | Required |
|----------|-------------|---------|----------|
| `RUST_LOG` | Log level (error, warn, info, debug, trace) | `info` | No |

#### Certificates (WebTransport only)

| Variable | Description | Default | Required |
|----------|-------------|---------|----------|
| `CERT_PATH` | SSL certificate path | `./certificates/cert.pem` | Yes (WebTransport) |
| `KEY_PATH` | SSL key path | `./certificates/key.pem` | Yes (WebTransport) |
| `DIGEST_PATH` | Certificate digest path | `./certificates/digest.txt` | Yes (WebTransport) |

#### Database (Future Use)

| Variable | Description | Default | Required |
|----------|-------------|---------|----------|
| `DATABASE_URL` | PostgreSQL connection string | None | No |

#### Client Configuration

| Variable | Description | Default | Required |
|----------|-------------|---------|----------|
| `INTERPOLATION_BUFFER_MS` | Network smoothing buffer | `100` | No |
| `INPUT_DELAY_FRAMES` | Input lag compensation | `0` | No |

#### Development

| Variable | Description | Default | Required |
|----------|-------------|---------|----------|
| `DEBUG_MODE` | Enable debug features | `false` | No |
| `PHYSICS_DEBUG` | Show physics wireframes | `false` | No |
| `NETWORK_LATENCY_MS` | Simulated latency | None | No |
| `NETWORK_JITTER_MS` | Simulated jitter | None | No |
| `NETWORK_LOSS_PERCENT` | Simulated packet loss | None | No |

#### Production

| Variable | Description | Default | Required |
|----------|-------------|---------|----------|
| `PRODUCTION` | Production mode | `false` | No |
| `MAX_CONNECTIONS` | Max concurrent clients | `100` | No |
| `RATE_LIMIT` | Requests per second per client | `60` | No |

## Production Considerations

### Security

1. **Use proper TLS certificates** from a CA like Let's Encrypt
2. **Run behind reverse proxy** (nginx, Traefik, Caddy)
3. **Enable firewall rules** to restrict access
4. **Regular updates** via Dependabot

### Performance

1. **Resource limits**: Set CPU/memory limits in docker-compose.yml
2. **Horizontal scaling**: Run multiple server instances behind load balancer
3. **Monitoring**: Add Prometheus metrics and Grafana dashboards

### Networking

For production, update `crates/game-core/src/common/shared.rs`:

\`\`\`rust
pub const SERVER_ADDR: SocketAddr = SocketAddr::new(
    IpAddr::V4(Ipv4Addr::new(0, 0, 0, 0)), // Listen on all interfaces
    SERVER_PORT
);
\`\`\`

## Cloud Deployment

### DigitalOcean / AWS / GCP

1. Create VM instance
2. Install Docker
3. Clone repository
4. Run docker-compose

### Kubernetes

See `kubernetes/` directory for deployment manifests (coming soon).

## Troubleshooting

### Server won't start
- Check certificates exist in `./certificates/`
- Verify port 5000 is not in use
- Check logs: `docker logs multiplayer-server`

### Web client can't connect
- Verify server is running
- Check firewall rules allow port 5000
- For WebTransport, ensure certificate digest matches

### Performance issues
- Use release builds (automatically in Docker)
- Increase resource limits
- Monitor with `docker stats`
\`\`\`

### Step 4.3: Contributing Guide

**File**: `CONTRIBUTING.md`

```markdown
# Contributing to Multiplayer-Template

Thank you for your interest in contributing!

## Development Setup

1. **Clone the repository**:
   \`\`\`bash
   git clone https://github.com/YOUR_USERNAME/Multiplayer-Template.git
   cd Multiplayer-Template
   \`\`\`

2. **Install dependencies**:
   - Rust 1.88+
   - Docker (for testing containers)
   - Trunk: `cargo install trunk`

3. **Build and test**:
   \`\`\`bash
   cargo build --all
   cargo test --all
   \`\`\`

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

\`\`\`bash
# Format check
cargo fmt --all -- --check

# Linting
cargo clippy --all -- -D warnings

# Tests
cargo test --all

# Build all targets
cargo build --all
\`\`\`

## Testing Docker Builds

\`\`\`bash
# Build server image
docker build -f Dockerfile.server -t multiplayer-server .

# Build web image
docker build -f Dockerfile.web -t multiplayer-web .

# Test with docker-compose
docker-compose up
\`\`\`

## Questions?

Open an issue or discussion on GitHub.
\`\`\`

---

## Phase 5: Local Testing

### Step 5.1: Test Docker Builds

```bash
# Build server image
docker build -f Dockerfile.server -t multiplayer-server:test .

# Build web image (this takes 10-15 minutes first time)
docker build -f Dockerfile.web -t multiplayer-web:test .

# Test with docker-compose
docker-compose build
docker-compose up

# Verify:
# - Server logs show "INFO server: ..."
# - Web client accessible at http://localhost:8080
# - Can connect and play
```

### Step 5.2: Test Workflows Locally (Optional)

Install `act` to run GitHub Actions locally:

```bash
# Install act (macOS)
brew install act

# Run CI workflow
act -j check

# Note: This may have limitations compared to actual GitHub Actions
```

### Step 5.3: Verify All Files Created

Checklist of new files:
```bash
ls -la Dockerfile.server
ls -la Dockerfile.web
ls -la docker-compose.yml
ls -la .dockerignore
ls -la .github/workflows/ci.yml
ls -la .github/workflows/docker-server.yml
ls -la .github/workflows/deploy-web.yml
ls -la .github/workflows/release.yml
ls -la .github/dependabot.yml
ls -la DEPLOYMENT.md
ls -la CONTRIBUTING.md
```

---

## Phase 6: GitHub Repository Setup

### Step 6.1: Create GitHub Repository

**Option A: Using GitHub CLI** (recommended):

```bash
gh repo create Multiplayer-Template --public --source=. --remote=origin --push
```

**Option B: Manual via GitHub Website**:

1. Go to https://github.com/new
2. Repository name: `Multiplayer-Template`
3. Description: "A production-ready multiplayer game template using Bevy + Lightyear"
4. Public repository
5. Do NOT initialize with README (we have one)
6. Click "Create repository"

### Step 6.2: Push Code to GitHub

If created manually, connect and push:

```bash
git remote add origin https://github.com/YOUR_USERNAME/Multiplayer-Template.git
git branch -M main
git add .
git commit -m "Initial commit: Multiplayer template with CI/CD"
git push -u origin main
```

### Step 6.3: Configure GitHub Settings

1. **Enable GitHub Pages**:
   - Go to Settings > Pages
   - Source: "GitHub Actions"
   - Save

2. **Enable GitHub Packages** (automatic, no config needed)

3. **Set Repository Description and Topics**:
   - Description: "Production-ready multiplayer game template using Bevy + Lightyear with Docker and CI/CD"
   - Topics: `bevy`, `rust`, `game-development`, `multiplayer`, `lightyear`, `template`, `docker`, `cicd`

4. **Branch Protection** (optional but recommended):
   - Settings > Branches > Add rule
   - Branch name pattern: `main`
   - Enable: "Require status checks to pass"
   - Select: CI workflow

### Step 6.4: Create Initial Release Tag

```bash
git tag -a v0.1.0 -m "Initial release: Multiplayer Template"
git push origin v0.1.0
```

This triggers the release workflow and creates downloadable binaries.

---

## Phase 7: Verification

### Step 7.1: Verify CI Workflow

1. Go to GitHub Actions tab
2. Check that "CI" workflow runs and passes
3. Verify on all platforms (Linux, macOS, Windows)

### Step 7.2: Verify Docker Images

1. Check "Docker Server" workflow passes
2. Go to Packages tab on GitHub
3. Verify `multiplayer-template-server` image exists
4. Pull and test:
   ```bash
   docker pull ghcr.io/YOUR_USERNAME/multiplayer-template-server:latest
   docker run -p 5000:5000 -v ./certificates:/certificates ghcr.io/YOUR_USERNAME/multiplayer-template-server:latest
   ```

### Step 7.3: Verify Web Deployment

1. Check "Deploy Web" workflow passes
2. Visit: `https://YOUR_USERNAME.github.io/Multiplayer-Template/`
3. Verify web client loads and connects to server

### Step 7.4: Verify Release Assets

1. Go to Releases tab
2. Check v0.1.0 release exists
3. Verify downloadable binaries for:
   - Linux server and client
   - macOS server and client
   - Windows server and client
   - Web client (tar.gz)

### Step 7.5: Update README Badges

After first successful workflow runs, update badges in README.md with actual links.

---

## Implementation Checklist

Track progress through implementation:

### Phase 1: Repository Preparation
- [ ] Update Cargo.toml workspace metadata
- [ ] Update root README.md title and content
- [ ] Update app READMEs
- [ ] Create .env.example with all configuration variables
- [ ] Add .env to .gitignore
- [ ] Remove standalone_avian_3d_character/
- [ ] Update .gitignore

### Phase 2: Docker Setup
- [ ] Create Dockerfile.server
- [ ] Create Dockerfile.web
- [ ] Create docker-compose.yml
- [ ] Create .dockerignore

### Phase 3: GitHub Actions
- [ ] Create .github/workflows/ directory
- [ ] Create ci.yml
- [ ] Create docker-server.yml
- [ ] Create deploy-web.yml
- [ ] Create release.yml

### Phase 4: Additional Config
- [ ] Create .github/dependabot.yml
- [ ] Create DEPLOYMENT.md
- [ ] Create CONTRIBUTING.md

### Phase 5: Local Testing
- [ ] Test docker build (server)
- [ ] Test docker build (web)
- [ ] Test docker-compose up
- [ ] Verify all new files exist

### Phase 6: GitHub Setup
- [ ] Create GitHub repository
- [ ] Push code to GitHub
- [ ] Enable GitHub Pages
- [ ] Configure repository settings
- [ ] Create v0.1.0 release tag

### Phase 7: Verification
- [ ] Verify CI workflow passes
- [ ] Verify Docker images published
- [ ] Verify web client deployed
- [ ] Verify release assets created
- [ ] Update README badges

---

## Troubleshooting

### Docker Build Fails

**Issue**: `cargo build` fails in Docker

**Solution**:
- Ensure Cargo.lock is committed
- Check .dockerignore doesn't exclude needed files
- Try building locally first: `cargo build --release -p server`

### GitHub Actions: Permission Denied

**Issue**: Cannot push to ghcr.io

**Solution**:
- Ensure `packages: write` permission in workflow
- Check repository settings allow GitHub Actions to create packages

### Web Deployment: 404 Page

**Issue**: GitHub Pages shows 404

**Solution**:
- Verify GitHub Pages is enabled
- Check deploy-web.yml ran successfully
- Ensure index.html is in artifact root

### Docker Image: Certificate Not Found

**Issue**: Server fails with certificate error

**Solution**:
- Mount certificates volume: `-v ./certificates:/certificates`
- Generate certificates if missing (see DEPLOYMENT.md)
- Update certificate paths in game-core

### Release Workflow: Artifacts Missing

**Issue**: Release has no files

**Solution**:
- Check build-binaries and build-web jobs succeeded
- Verify artifact upload steps ran
- Check artifact paths are correct

### Trunk Build: Out of Memory

**Issue**: WASM build fails with OOM

**Solution**:
- Increase Docker memory limit (8GB+)
- Use `--release` flag (already in workflow)
- Split build into multiple stages

---

## Next Steps After Implementation

1. **Customize for your game**:
   - Update game logic
   - Add custom assets
   - Modify player controls

2. **Add monitoring**:
   - Prometheus metrics
   - Grafana dashboards
   - Error tracking (Sentry)

3. **Scaling**:
   - Kubernetes manifests
   - Load balancing
   - Database integration

4. **Documentation**:
   - Gameplay guide
   - API documentation
   - Architecture diagrams

5. **Community**:
   - Add issue templates
   - Create discussions
   - Write contributor guide

---

## Resources

- [Bevy Documentation](https://bevyengine.org/learn/)
- [Lightyear Documentation](https://github.com/cBournhonesque/lightyear)
- [Docker Documentation](https://docs.docker.com/)
- [GitHub Actions Documentation](https://docs.github.com/en/actions)
- [Trunk Documentation](https://trunkrs.dev/)

---

## Support

If you encounter issues during implementation:

1. Check troubleshooting section above
2. Search existing GitHub issues
3. Open a new issue with:
   - Step where you're stuck
   - Error messages
   - What you've tried

---

**Implementation Time Estimate**: 2-3 hours
**Difficulty**: Intermediate
**Prerequisites**: Docker, GitHub, basic CI/CD knowledge

Good luck! 🚀
