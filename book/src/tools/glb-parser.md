# glb-parser

A CLI tool that reads a GLB file, detects each node's entity type, extracts light metadata from the `KHR_lights_punctual` extension, and generates a [`dynamic_objects_config.json`](../guides/world-dynamic/dynamic-objects.md) template.

## Why not the `gltf` crate?

The parser reads the GLB binary format directly — it extracts the JSON chunk at offset 20 and deserializes it with serde. This avoids the `gltf` crate's strict extension validation, which can reject files that use custom or draft extensions from Blender.

## Installation

The tool is part of the workspace — no separate install needed.

```sh
cargo build -p glb-parser
```

## Usage

```sh
# Print config template to stdout
cargo run -p glb-parser -- <path-to.glb>

# Write to file (creates new or merges with existing)
cargo run -p glb-parser -- <path-to.glb> -o <output.json>
```

### Example

```sh
# Generate a fresh config
cargo run -p glb-parser -- assets/models/world_dynamic.glb -o assets/config/dynamic_objects_config.json
```

## Type Detection

The parser inspects each named node in the glTF node array and classifies it:

| Type | Condition |
|------|-----------|
| `light` | Node has a `KHR_lights_punctual` extension reference |
| `mesh` | Node has a `mesh` index |
| `camera` | Node has a `camera` index |
| `empty` | None of the above (e.g., an Empty in Blender) |

For `light` nodes, the parser also extracts a `light_info` block:

```json
{
  "light_type": "point",
  "color": [1.0, 0.8, 0.3],
  "intensity": 800.0
}
```

Light type, color, and intensity come from the `KHR_lights_punctual` root extension. Defaults are `"point"`, `[1.0, 1.0, 1.0]`, and `1.0` if omitted.

## Output Format

The output is a JSON object with a `nodes` map. Each key is the Blender node name:

```json
{
  "nodes": {
    "torch_light_01": {
      "type": "light",
      "light_info": {
        "light_type": "point",
        "color": [1.0, 0.8, 0.3],
        "intensity": 800.0
      },
      "triggers": []
    },
    "door_01": {
      "type": "mesh",
      "triggers": []
    },
    "zone_entrance": {
      "type": "empty",
      "triggers": []
    }
  }
}
```

Nodes are sorted by type then name. Unnamed nodes are excluded.

## Merge Mode

When the `-o` output file already exists, the parser merges instead of overwriting:

- **Existing nodes** keep their triggers, state, and custom config untouched
- **New nodes** from the GLB are added with empty triggers
- **Missing nodes** (in config but not in GLB) produce a stderr warning
- **Type upgrades** — existing nodes typed as `"empty"` are updated if the GLB now has better type info (e.g., a light was added in Blender)

This makes it safe to re-run the parser after adding objects in Blender without losing your trigger configuration.

## Status Output

The parser prints a summary to stderr (never mixed into the JSON on stdout):

```
Merging with existing config (5 nodes)
GLB nodes: 8, new: 3, updated: 1, total config nodes: 10
  empty: 1
  light: 4
  mesh: 3
Written to assets/config/dynamic_objects_config.json
```

## Typical Workflow

1. Model objects in Blender and export to GLB
2. Run `cargo run -p glb-parser -- world_dynamic.glb -o dynamic_objects_config.json`
3. Edit the config to add triggers, actions, and state
4. Re-export from Blender after changes, re-run the parser — new nodes are merged in

See the [Dynamic Objects guide](../guides/world-dynamic/dynamic-objects.md) for full details on triggers, actions, state, and the Blender export settings.
