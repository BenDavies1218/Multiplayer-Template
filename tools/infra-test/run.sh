#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
cd "$SCRIPT_DIR"

NUM_CLIENTS=1
ACTION="up"

usage() {
    echo "Usage: $0 [OPTIONS]"
    echo ""
    echo "Infrastructure load testing for the multiplayer server."
    echo ""
    echo "Options:"
    echo "  --clients N   Number of headless clients to spawn (default: 1)"
    echo "  --down        Tear down all containers and volumes"
    echo "  --logs        Follow container logs"
    echo "  --help        Show this help message"
    echo ""
    echo "Examples:"
    echo "  $0                  # Start with 1 client"
    echo "  $0 --clients 10    # Start with 10 clients"
    echo "  $0 --down          # Stop everything"
    echo "  $0 --logs          # Tail logs"
}

while [[ $# -gt 0 ]]; do
    case "$1" in
        --clients)
            NUM_CLIENTS="$2"
            shift 2
            ;;
        --down)
            ACTION="down"
            shift
            ;;
        --logs)
            ACTION="logs"
            shift
            ;;
        --help)
            usage
            exit 0
            ;;
        *)
            echo "Unknown option: $1"
            usage
            exit 1
            ;;
    esac
done

case "$ACTION" in
    up)
        echo "==> Pulling latest images..."
        docker compose pull

        echo "==> Starting monitoring stack (cadvisor, prometheus, grafana)..."
        docker compose up -d cadvisor prometheus grafana

        echo "==> Starting server..."
        docker compose up -d server

        echo "==> Waiting for server to be healthy..."
        docker compose up -d --wait server

        echo "==> Scaling to ${NUM_CLIENTS} client(s)..."
        NUM_CLIENTS="$NUM_CLIENTS" docker compose up -d --scale client="$NUM_CLIENTS" client

        echo ""
        echo "========================================"
        echo "  Infrastructure test stack is running"
        echo "========================================"
        echo ""
        echo "  Grafana:    http://localhost:3000"
        echo "  Prometheus: http://localhost:9090"
        echo "  cAdvisor:   http://localhost:8080"
        echo ""
        echo "  Server:     localhost:5888"
        echo "  Clients:    ${NUM_CLIENTS}"
        echo ""
        echo "  Dashboard:  http://localhost:3000/d/load-test/load-test-dashboard"
        echo ""
        echo "  Tear down:  $0 --down"
        echo "  View logs:  $0 --logs"
        echo "========================================"
        ;;
    down)
        echo "==> Tearing down all containers and volumes..."
        docker compose down -v
        echo "Done."
        ;;
    logs)
        docker compose logs -f
        ;;
esac
