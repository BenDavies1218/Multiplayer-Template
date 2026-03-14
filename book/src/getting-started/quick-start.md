# Quick Start

Get a server and client running in under 2 minutes.

## 1. Start the Server

```bash
cargo dev-server
```

The server listens on port **5888** with UDP by default.

## 2. Start a Client

In a new terminal:

```bash
cargo dev-native
```

This launches a native client with client ID 1.

## 3. Run Multiple Clients

Open additional terminals:

```bash
# Terminal 2
cargo run -p native -- client -c 2

# Terminal 3
cargo run -p native -- client -c 3
```

## 4. Web Client

In a new terminal:

```bash
cd apps/web
trunk serve
```

Open <http://localhost:8080> in your browser.

## Controls

| Key | Action |
|-----|--------|
| W/A/S/D | Move |
| Space | Jump |
| Left Shift | Sprint |
| C | Crouch |
| Q | Shoot |
| Left Click | Grab cursor |
| Escape | Release cursor |

## Docker Quick Start

```bash
docker compose up -d
```

This starts the server on port 5888 and the web client on port 8080.
