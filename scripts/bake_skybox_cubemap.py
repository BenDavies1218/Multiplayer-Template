#!/usr/bin/env python3
"""
Pre-bake an equirectangular EXR skybox into a stacked cubemap EXR.

Output format: face_size x (face_size * 6), with 6 cube faces stacked vertically.
Face order: +X, -X, +Y, -Y, +Z, -Z (standard cubemap convention).

The game renderer detects 1:6 aspect ratio and uses the fast
reinterpret_stacked_2d_as_array(6) path — no CPU conversion at runtime.

Usage:
    python3 scripts/bake_skybox_cubemap.py [--face-size 1024] [--input path] [--output path]

Requirements:
    pip install numpy openexr
"""

import argparse
import sys
import numpy as np
import OpenEXR


def load_exr(path: str) -> np.ndarray:
    """Load an EXR file, returning float32 RGBA array (H, W, 4)."""
    f = OpenEXR.File(path)
    channels = f.channels()

    # Handle different channel layouts
    if 'RGB' in channels:
        # Combined RGB channel
        rgb = channels['RGB'].pixels  # (H, W, 3)
        img = rgb.astype(np.float32)
    elif 'R' in channels and 'G' in channels and 'B' in channels:
        # Separate R, G, B channels
        r = channels['R'].pixels
        g = channels['G'].pixels
        b = channels['B'].pixels
        img = np.stack([r, g, b], axis=-1).astype(np.float32)
    else:
        print(f"ERROR: Unexpected channel layout: {list(channels.keys())}")
        sys.exit(1)

    # Add alpha channel if missing
    if img.shape[-1] == 3:
        alpha = np.ones((*img.shape[:2], 1), dtype=np.float32)
        img = np.concatenate([img, alpha], axis=-1)

    return img


def save_exr(path: str, img: np.ndarray):
    """Save a float32 RGBA array as EXR."""
    h, w, c = img.shape

    channel_names = ['R', 'G', 'B', 'A'][:c]
    channels = {}
    for i, name in enumerate(channel_names):
        channels[name] = img[:, :, i].astype(np.float32)

    header = {
        'compression': OpenEXR.PIZ_COMPRESSION,
        'type': OpenEXR.scanlineimage,
    }
    out = OpenEXR.File(header, channels)
    out.write(path)


# Cube face direction functions.
# Each returns (x, y, z) direction for a given (u, v) in [-1, 1].
FACE_DIRS = [
    lambda u, v: ( 1.0,   -v,   -u),   # +X
    lambda u, v: (-1.0,   -v,    u),   # -X
    lambda u, v: (   u,  1.0,    v),   # +Y
    lambda u, v: (   u, -1.0,   -v),   # -Y
    lambda u, v: (   u,   -v,  1.0),   # +Z
    lambda u, v: (  -u,   -v, -1.0),   # -Z
]


def equirect_to_cubemap(src: np.ndarray, face_size: int) -> np.ndarray:
    """
    Convert equirectangular image to stacked cubemap using vectorized numpy.

    Returns array of shape (face_size * 6, face_size, channels).
    """
    src_h, src_w, channels = src.shape

    # Create UV grid for one face: [-1, 1]
    pixel_coords = np.arange(face_size, dtype=np.float32) + 0.5
    u_grid = pixel_coords / face_size * 2.0 - 1.0
    v_grid = pixel_coords / face_size * 2.0 - 1.0
    uu, vv = np.meshgrid(u_grid, v_grid)  # (face_size, face_size)

    out = np.zeros((face_size * 6, face_size, channels), dtype=np.float32)

    for face_idx, face_fn in enumerate(FACE_DIRS):
        print(f"  Processing face {face_idx}/5...", end=" ", flush=True)

        # Get 3D direction for each pixel
        dx, dy, dz = face_fn(uu, vv)
        dx = np.asarray(dx, dtype=np.float32)
        dy = np.asarray(dy, dtype=np.float32)
        dz = np.asarray(dz, dtype=np.float32)

        # Normalize
        length = np.sqrt(dx*dx + dy*dy + dz*dz)
        dx = dx / length
        dy = dy / length
        dz = dz / length

        # Convert to equirectangular UV
        theta = np.arctan2(dx, dz)
        phi = np.arcsin(np.clip(dy, -1.0, 1.0))
        eq_u = 0.5 + theta / (2.0 * np.pi)
        eq_v = 0.5 - phi / np.pi

        # Bilinear interpolation coordinates
        fx = eq_u * src_w - 0.5
        fy = eq_v * src_h - 0.5

        x0 = np.floor(fx).astype(np.int64) % src_w
        y0 = np.clip(np.floor(fy).astype(np.int64), 0, src_h - 1)
        x1 = (x0 + 1) % src_w
        y1 = np.clip(y0 + 1, 0, src_h - 1)

        frac_x = (fx - np.floor(fx))[..., np.newaxis]  # (face, face, 1)
        frac_y = (fy - np.floor(fy))[..., np.newaxis]

        # Sample 4 corners
        s00 = src[y0, x0]  # (face_size, face_size, channels)
        s10 = src[y0, x1]
        s01 = src[y1, x0]
        s11 = src[y1, x1]

        # Bilinear blend
        top = s00 + (s10 - s00) * frac_x
        bot = s01 + (s11 - s01) * frac_x
        result = top + (bot - top) * frac_y

        y_start = face_idx * face_size
        out[y_start:y_start + face_size] = result
        print("done", flush=True)

    return out


def main():
    parser = argparse.ArgumentParser(
        description="Pre-bake equirectangular EXR skybox to stacked cubemap EXR"
    )
    parser.add_argument(
        "--input", "-i",
        default="assets/sunset_sky_hdr.exr",
        help="Input equirectangular EXR path (default: assets/sunset_sky_hdr.exr)"
    )
    parser.add_argument(
        "--output", "-o",
        default=None,
        help="Output stacked cubemap EXR path (default: <input>_cubemap.exr)"
    )
    parser.add_argument(
        "--face-size", "-s",
        type=int,
        default=1024,
        help="Cube face size in pixels (default: 1024)"
    )
    args = parser.parse_args()

    if args.output is None:
        base = args.input.rsplit(".", 1)[0]
        args.output = f"{base}_cubemap.exr"

    print(f"Loading {args.input}...")
    src = load_exr(args.input)
    print(f"  Source: {src.shape[1]}x{src.shape[0]}, {src.shape[2]} channels, dtype={src.dtype}")

    ratio = src.shape[1] / src.shape[0]
    if abs(ratio - 2.0) > 0.1:
        print(f"WARNING: Expected 2:1 aspect ratio (equirectangular), got {ratio:.2f}:1")

    print(f"Converting to cubemap (face size: {args.face_size})...")
    cubemap = equirect_to_cubemap(src, args.face_size)
    print(f"  Output: {cubemap.shape[1]}x{cubemap.shape[0]} (6 faces stacked)")

    print(f"Saving {args.output}...")
    save_exr(args.output, cubemap)
    print(f"Done! Update game_core_config.json skybox_path to point to the new file.")


if __name__ == "__main__":
    main()
