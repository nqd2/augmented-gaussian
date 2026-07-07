# augmented-gaussian Project Summary

## Goal

`augmented-gaussian` processes 3D Gaussian Splatting data for AR systems. It converts splats into structured geometry so virtual objects can collide with scanned obstacles and use generated terrain/navigation data.

## Stack

- **Tauri app** for the desktop shell
- **Rust backend** for parsing, transforms, processing, and exports
- **`wgpu`** for GPU voxelization and carving
- **React, Vite, TypeScript** for the GUI
- **PlayCanvas** for 3DGS preview, calibration, and gizmos
- **`rerecast`** in `core-lib` for navmesh baking

## Pipeline

### Inputs

The tool supports `.ply`, `.splat`, `.sog`, and `meta.json`.

### Data Model

`SplatTable` stores splats in columnar vectors:

- Positions: `x`, `y`, `z`
- Scales: `scale_0`, `scale_1`, `scale_2` as log scale `ln(s)`
- Opacity: `opacity` as logit scale `ln(alpha / (1 - alpha))`
- Spherical harmonics: `f_dc_0..2` and optional `f_rest_0..44`
- Rotation: `rot_0..3` as normalized quaternion

### Processing Steps

1. The frontend records edits as JSON recipes.
2. The backend replays recipes on the source data.
3. Alignment applies floor normal, up-axis, scale, and origin.
4. Filters remove invalid, low-opacity, masked, disconnected, or non-contributing splats.
5. CPU or GPU voxelization evaluates Gaussian opacity at voxel centers.
6. Fill and carve stages produce the occupied region used for geometry.
7. Mesh extraction emits either exposed voxel faces or a smoothed mesh.
8. `rerecast` bakes the navmesh from the final collision mesh.

## Export Bundle

The WebAR bundle contains:

- `index.html`: local PlayCanvas viewer
- `assets/js/playcanvas.min.js`: bundled PlayCanvas runtime
- `scene.sog`: processed SOG V2 splat bundle
- `occlusion.glb`: collision/occlusion geometry
- `navmesh.glb` and `navmesh.bin`: present when `rerecast` finds walkable polygons
- `manifest.json`: transform, bounds, parameters, metrics, and artifact paths

## Development Notes

`AGENTS.md` captures the project instructions:

- Keep edits small and goal-driven.
- Use direct prose in docs and UI copy.
- Keep CLI and GUI outputs identical for the same input, config, and recipe.
- Test the Rust core before relying on GUI behavior.
