# Web Client (WASM)

The web browser client for the multiplayer Bevy game. This binary compiles to WebAssembly (WASM) and runs directly in modern web browsers.

## Purpose

The web client provides:

- Browser-based gameplay without installation
- Same features as native client (prediction, rendering, interpolation, dynamic objects)
- Automatic client ID generation
- WebTransport or WebSocket connectivity

## Features

- **WebAssembly**: Runs natively in the browser
- **Client-Side Prediction**: Instant local response
- **100ms Interpolation Buffer**: Smooth remote player movement
- **3D Graphics**: WebGL2-based rendering
- **No Installation**: Play directly in the browser

## Prerequisites

Install [Trunk](https://trunkrs.dev/), the WASM build tool:

```bash
cargo install trunk
```

Or using Homebrew (macOS/Linux):

```bash
brew install trunk
```

## Building

### Development Build

```bash
cd apps/web
trunk build
```

Output: `dist/` directory with HTML, WASM, and JS files.

### Production Build (Optimized)

```bash
cd apps/web
trunk build --release
```

This enables:

- WASM optimization (wasm-opt)
- Smaller file sizes
- Better runtime performance

## Running Locally

### Development Server

```bash
cd apps/web
trunk serve
```

Open <http://localhost:8080> in your browser.

Features:

- Auto-reload on code changes
- Fast incremental builds
- Source maps for debugging

### Release Server

```bash
cd apps/web
trunk serve --release
```

Opens at <http://localhost:8080> with optimized WASM.

### Custom Port

```bash
trunk serve --port 3000
```

## Controls

- **W/A/S/D**: Move
- **Space**: Jump
- **Left Shift**: Sprint
- **C**: Crouch
- **Q**: Shoot
- **Left Click**: Grab cursor
- **Escape**: Release cursor

## Deployment

### Build for Production

```bash
cd apps/web
trunk build --release
```

### Deploy the `dist/` Folder

Upload the contents of `apps/web/dist/` to your web server or CDN:

```text
dist/
├── index.html          # Entry point
├── web-*.wasm          # Compiled game (large file)
├── web-*.js            # WASM loader
├── assets/             # Game assets (copied from workspace root)
└── web-*.css           # Styles (if any)
```

### Hosting Recommendations

#### Static Hosting (Easiest)

- **Netlify**: Drag and drop `dist/` folder
- **Vercel**: Connect git repo, set build command to `cd apps/web && trunk build --release`
- **GitHub Pages**: Push `dist/` to gh-pages branch
- **Cloudflare Pages**: Connect repo and configure build

#### Self-Hosting (Nginx)

```nginx
server {
    listen 80;
    server_name yourgame.com;

    root /var/www/multiplayer-bevy/dist;
    index index.html;

    # Enable gzip compression for WASM
    gzip on;
    gzip_types application/wasm application/javascript;

    location / {
        try_files $uri $uri/ /index.html;
    }

    # Cache WASM files
    location ~* \.wasm$ {
        expires 1y;
        add_header Cache-Control "public, immutable";
    }
}
```

## Browser Compatibility

### Minimum Requirements

- **WebGL2** support (most browsers 2017+)
- **WebAssembly** support (all modern browsers)
- **WebTransport** or **WebSocket** support

### Tested Browsers

- Chrome/Edge 90+
- Firefox 90+
- Safari 15.4+

Safari has limited WebTransport support. The game automatically falls back to WebSocket if needed.

### Mobile Browsers

The game technically works on mobile browsers but is not optimized for touch controls.

## Configuration

### Trunk.toml

The `Trunk.toml` file configures the WASM build:

```toml
[build]
target = "index.html"
rustflags = ["--cfg", "getrandom_backend=\"wasm_js\""]

[watch]
ignore = ["dist/"]
```

### index.html

The HTML entry point loads the WASM binary and copies the assets directory:

```html
<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="utf-8"/>
    <meta name="viewport" content="width=device-width, initial-scale=1, user-scalable=no">
    <title>Bevy game</title>
    <link data-trunk rel="rust"/>
    <link data-trunk rel="copy-dir" href="../../assets"/>
</head>
</html>
```

The `copy-dir` directive copies the workspace `assets/` directory (including config files) into the `dist/` output.

### Random Client ID

Unlike native clients, web clients generate a random client ID automatically:

```rust
cfg_if::cfg_if! {
    if #[cfg(target_family = "wasm")] {
        let client_id = rand::random::<u64>();
        Cli {
            mode: Some(Mode::Client {
                client_id: Some(client_id),
            })
        }
    } else {
        Cli::parse()
    }
}
```

## Network Configuration

### Transport

The web client uses **WebTransport** by default. Available transports for WASM:

- **WebTransport** (default, QUIC-based)
- **WebSocket** (better browser compatibility)

UDP is **not available** for WASM.

### WebTransport Certificates

For WebTransport to work, the web client needs the server's certificate digest.

The digest is embedded at compile time from `certificates/digest.txt` (in `crates/game-client/src/transport.rs`):

```rust
#[cfg(target_family = "wasm")]
{
    include_str!("../../../certificates/digest.txt")
        .trim()
        .replace(':', "")
}
```

Ensure `certificates/digest.txt` exists before building for WebTransport.

### Server Address

The server address is configured in `assets/config/game_core_config.json`:

```json
{
  "networking": {
    "server_host": "127.0.0.1",
    "server_port": 5888
  }
}
```

For production, change these to your server's public IP or domain.

## Performance

### File Sizes

Typical WASM build sizes:

- **Development**: ~50-100 MB (unoptimized)
- **Release**: ~15-30 MB (optimized with wasm-opt)

### Loading Time

First load:

- **Development**: 5-15 seconds
- **Release**: 2-5 seconds (depending on connection)

Browser caches WASM after first load, making subsequent loads instant.

### Runtime Performance

- **Frame Rate**: 60 FPS (vsync limited in most browsers)
- **Physics**: 64Hz fixed timestep
- **Network**: 64Hz send rate

## Troubleshooting

### Build Errors

#### "getrandom" Error

If you see getrandom-related errors:

1. Ensure `Trunk.toml` has the rustflags:

   ```toml
   rustflags = ["--cfg", "getrandom_backend=\"wasm_js\""]
   ```

2. Verify `Cargo.toml` has WASM dependencies:

   ```toml
   [target."cfg(target_family = \"wasm\")".dependencies]
   getrandom = { version = "0.2", features = ["js"] }
   ```

### Runtime Errors

#### Connection Failed

If the game loads but can't connect:

1. Ensure server is running: `cargo run -p server -- server`
2. Check browser console for WebTransport/WebSocket errors
3. Verify server address is correct (`localhost:5888` for development)
4. Try WebSocket transport if WebTransport fails

#### Slow Performance

If the game runs slowly in browser:

1. Use **release build**: `trunk serve --release`
2. Close other browser tabs (WASM uses significant memory)
3. Use a Chromium-based browser (better WASM performance than Firefox/Safari)
4. Check browser console for errors

### Certificate Issues

For WebTransport, ensure:

```bash
# Certificate digest must exist
ls certificates/digest.txt

# Rebuild after certificate changes
cd apps/web
trunk build
```

## Development Workflow

### Typical Development Loop

```bash
# Terminal 1: Run server
cargo run -p server -- server

# Terminal 2: Run web client
cd apps/web
trunk serve

# Edit code, see changes automatically reload in browser
```

### Browser DevTools

- **F12**: Open browser console to see logs
- **Network tab**: Monitor WASM and asset loading
- **Performance tab**: Profile frame rate and bottlenecks

## Compression

Enable Brotli/Gzip compression on your web server for faster loading:

```nginx
# Nginx example
gzip on;
gzip_types application/wasm application/javascript;
brotli on;
brotli_types application/wasm application/javascript;
```

## Related Documentation

- [Root README](../../README.md)
- [Server README](../server/README.md)
- [Native Client README](../native/README.md)
- [Trunk Documentation](https://trunkrs.dev/)
