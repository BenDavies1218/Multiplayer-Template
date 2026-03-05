# Web Client (WASM)

The web browser client for the multiplayer Bevy game. This binary compiles to WebAssembly (WASM) and runs directly in modern web browsers.

## Purpose

The web client provides:
- Browser-based gameplay without installation
- Same features as native client (prediction, rendering, interpolation)
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

**Note**: Production builds are significantly larger in disk size during compilation but result in smaller deployed files.

## Running Locally

### Development Server

```bash
cd apps/web
trunk serve
```

Open http://localhost:8080 in your browser.

**Features**:
- Auto-reload on code changes
- Fast incremental builds
- Source maps for debugging

### Release Server

```bash
cd apps/web
trunk serve --release
```

Opens at http://localhost:8080 with optimized WASM.

### Custom Port

```bash
trunk serve --port 3000
```

## Deployment

### Build for Production

```bash
cd apps/web
trunk build --release
```

### Deploy the `dist/` Folder

Upload the contents of `apps/web/dist/` to your web server or CDN:

```
dist/
├── index.html          # Entry point
├── web-*.wasm          # Compiled game (large file)
├── web-*.js            # WASM loader
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

    root /var/www/Multiplayer-Template/dist;
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

### CORS and Headers

For WebTransport, you may need proper CORS headers. Most static hosts handle this automatically.

## Browser Compatibility

### Minimum Requirements

- **WebGL2** support (most browsers 2017+)
- **WebAssembly** support (all modern browsers)
- **WebTransport** or **WebSocket** support

### Tested Browsers

- Chrome/Edge 90+
- Firefox 90+
- Safari 15.4+

**Note**: Safari has limited WebTransport support. The game automatically falls back to WebSocket if needed.

### Mobile Browsers

The game technically works on mobile browsers but is not optimized for touch controls.

## Configuration

### Trunk.toml

The `Trunk.toml` file configures the WASM build:

```toml
[build]
target = "index.html"           # Entry point
no_default_features = true      # Disable server features
features = ["client", "netcode"] # Enable only client
rustflags = ["--cfg", "getrandom_backend=\"wasm_js\""]

[watch]
ignore = ["dist/", "target/"]   # Don't rebuild on these changes
```

**Important**: `no_default_features = true` prevents server-only code from being compiled into WASM.

### index.html

Minimal HTML file that loads the WASM binary:

```html
<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="utf-8"/>
    <meta name="viewport" content="width=device-width, initial-scale=1">
    <title>Bevy game</title>
    <link data-trunk rel="rust"/>  <!-- Trunk loads WASM here -->
</head>
</html>
```

### Random Client ID

Unlike native clients, web clients generate a random client ID automatically (apps/web/src/main.rs):

```rust
pub fn cli() -> Cli {
    cfg_if::cfg_if! {
        if #[cfg(target_family = "wasm")] {
            let client_id = rand::random::<u64>();  // Random ID for browser
            Cli {
                mode: Some(Mode::Client {
                    client_id: Some(client_id),
                })
            }
        } else {
            Cli::parse()  // Parse from command line
        }
    }
}
```

## Network Configuration

### Transport Selection

Edit `crates/game-core/src/common/cli.rs` to change transport:

```rust
// WebTransport (default, requires HTTPS in production)
transport: ClientTransports::WebTransport,

// WebSocket (better browser compatibility)
transport: ClientTransports::WebSocket,
```

**Note**: UDP is not available for WASM.

### WebTransport Certificates

For WebTransport to work, the web client needs the server's certificate digest.

In `crates/game-core/src/common/client.rs:97`, the digest is embedded at compile time:

```rust
#[cfg(target_family = "wasm")]
{
    include_str!("../../../../certificates/digest.txt").to_string()
}
```

Ensure `certificates/digest.txt` exists before building for WebTransport.

### Server Address

The server address is compiled into the WASM. Edit `crates/game-core/src/common/shared.rs`:

```rust
pub const SERVER_ADDR: SocketAddr = SocketAddr::new(
    IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)),  // Change for production
    SERVER_PORT
);
```

**For production**, set this to your server's public IP or domain.

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

#### "Server Feature in WASM" Error

If server code is trying to compile for WASM:

1. Check `Trunk.toml` has:
   ```toml
   no_default_features = true
   features = ["client", "netcode"]
   ```

2. Ensure no `#[cfg(feature = "server")]` code is accidentally included

### Runtime Errors

#### "Failed to Fetch WASM"

Check browser console. Ensure:
- Server is running (`trunk serve`)
- Port is correct (default: 8080)
- No CORS issues (shouldn't occur with trunk serve)

#### Connection Failed

If the game loads but can't connect:

1. Ensure server is running: `cargo run -p server -- server`
2. Check browser console for WebTransport/WebSocket errors
3. Verify server address is correct (localhost:5000 for development)
4. Try WebSocket transport if WebTransport fails

#### Slow Performance

If the game runs slowly in browser:

1. Use **release build**: `trunk serve --release`
2. Close other browser tabs (WASM uses significant memory)
3. Use a Chromium-based browser (better WASM performance than Firefox/Safari)
4. Check browser console for errors
5. Reduce graphics quality if needed

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

## Controls

Same as native client:

- **W/A/S/D**: Move
- **Space**: Jump
- **Left Mouse Click**: Shoot

## Advanced Topics

### Custom Build Script

Create a build script for CI/CD:

```bash
#!/bin/bash
cd apps/web
trunk build --release
# Upload dist/ to S3, Netlify, etc.
```

### Environment-Specific Builds

Use build.rs to inject environment variables into the build.

### Compression

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
