# Deployment Guide

This guide covers deploying Multiplayer-Template to production environments.

## Docker Deployment

### Prerequisites

- Docker 20.10+
- Docker Compose 2.0+ (optional)
- Valid SSL/TLS certificates for production

### Quick Start with Docker Compose

1. **Generate production certificates** (if using WebTransport):

```bash
# Generate production certificates with your domain
openssl req -x509 -newkey rsa:4096 -keyout certificates/key.pem \
    -out certificates/cert.pem -sha256 -days 365 -nodes \
    -subj "/CN=yourdomain.com"

# Generate digest for web clients
openssl x509 -in certificates/cert.pem -outform der | \
    openssl dgst -sha256 -binary | base64 > certificates/digest.txt
```

2. **Pull and run with Docker Compose**:

```bash
docker-compose up -d
```

3. **Access**:
   - Server: Port 5888
   - Web Client: http://localhost:8080

### Using Pre-built Images

#### Server

```bash
docker pull ghcr.io/BenDavies1218/multiplayer-template-server:latest

docker run -d \
  --name multiplayer-server \
  -p 5888:5888 \
  -v $(pwd)/certificates:/certificates:ro \
  -e RUST_LOG=info \
  --restart unless-stopped \
  ghcr.io/BenDavies1218/multiplayer-template-server:latest
```

#### Web Client

```bash
docker pull ghcr.io/BenDavies1218/multiplayer-template-web:latest

docker run -d \
  --name multiplayer-web \
  -p 8080:80 \
  --restart unless-stopped \
  ghcr.io/BenDavies1218/multiplayer-template-web:latest
```

## Environment Variables

### Configuration File

Copy `.env.example` to `.env` and customize:

```bash
cp .env.example .env
```

### Available Variables

#### Server Configuration

| Variable | Description | Default | Required |
|----------|-------------|---------|----------|
| `SERVER_HOST` | Server bind address | `127.0.0.1` | No |
| `SERVER_PORT` | Server port | `5888` | No |
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

For production, update server configuration to listen on all interfaces:

```bash
SERVER_HOST=0.0.0.0  # Listen on all interfaces
```

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
- Verify port 5888 is not in use
- Check logs: `docker logs multiplayer-server`

### Web client can't connect
- Verify server is running
- Check firewall rules allow port 5888
- For WebTransport, ensure certificate digest matches

### Performance issues
- Use release builds (automatically in Docker)
- Increase resource limits
- Monitor with `docker stats`
