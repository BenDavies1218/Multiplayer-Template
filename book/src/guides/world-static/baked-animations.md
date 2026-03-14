# Baked Animations Guide

How to add looping ambient animations to the world — baked in Blender, auto-played in Bevy.

---

## How It Works

Animations baked into the world visual GLB file are automatically detected and played on loop when the scene loads. This is client-only — the server does not process animations.

When Bevy's glTF loader finds animations in a GLB, it spawns `AnimationPlayer` and `AnimationGraphHandle` components on the animated entities. A system in `game-client` detects these and calls `.play().repeat()` on every clip.

**Key file:** `crates/game-client/src/renderer.rs` (`auto_play_gltf_animations` system)

---

## What's Supported

| Feature                    | Supported | Notes                                              |
| -------------------------- | --------- | -------------------------------------------------- |
| Object transform animations | Yes     | Location, rotation, scale keyframes                |
| Shape key animations       | Yes       | Morph target animations via glTF                   |
| Looping animations         | Yes       | All animations loop automatically                  |
| Multiple animations        | Yes       | All clips in the GLB play simultaneously           |
| Skeletal animations        | Yes       | Armature-driven animations export via glTF         |
| NLA strips                 | Partial   | Merged into a single action on export — use "Stash" or bake first |
| F-curve modifiers          | No        | Blender-specific — bake to keyframes before export |
| Constraints                | No        | Blender-specific — bake to keyframes before export |
| Interactive/triggered      | No        | All animations auto-play — no trigger support yet  |

---

## Blender Workflow

### 1. Create the Animation

- Select the object to animate
- Open the Timeline or Dope Sheet
- Insert keyframes (`I` key) for Location, Rotation, or Scale
- Set the animation range in the Timeline (Start/End frames)

### 2. Make It Loop

For seamless looping, ensure the first and last keyframes match:

- Copy the first keyframe and paste at the last frame
- Or use a cyclic F-curve modifier (Blender-only, must bake before export)

To bake F-curve modifiers: select the object, Graph Editor, Channel, Bake Action.

### 3. Export Settings

Use the standard visual mesh export settings with these additions:

```
Include:
  Selected Objects
  Punctual Lights    (if used)

Animation:
  Animations
  Always Sample Animations

(all other settings same as [Meshes Guide](meshes.md))
```

**Important:** Make sure animated objects are **selected** when exporting with "Selected Objects" enabled.

### 4. Multiple Animations

If you have multiple animated objects (e.g., a fan and a floating crystal), each object's animation is exported as a separate clip. All clips auto-play simultaneously in Bevy.

---

## Best Practices

- **Keep animations simple** — transform keyframes (location/rotation/scale) are cheapest
- **Bake constraints and modifiers** — only raw keyframes export to glTF
- **Test in world-viewer first** — `cargo dev-viewer` shows animations without networking overhead
- **Match frame rate** — Blender defaults to 24fps, which is fine for ambient loops
- **Avoid very long animations** — keep loops short (2-10 seconds) for ambient effects

---

## Troubleshooting

| Problem                       | Solution                                                          |
| ----------------------------- | ----------------------------------------------------------------- |
| Animation doesn't play        | Check Animation checkbox in export settings                       |
| Animation plays but wrong     | Bake constraints/modifiers before export                          |
| Animation doesn't loop        | Ensure first and last keyframes match                             |
| Object jumps at loop point    | Remove the duplicate last keyframe (off-by-one) or adjust timing  |
| NLA strips not exporting      | Stash/bake NLA strips to a single action before export            |
| Only some objects animate     | Make sure all animated objects are selected for export             |
| Animation too fast/slow       | Adjust keyframe timing in Blender — Bevy plays at exported speed  |
