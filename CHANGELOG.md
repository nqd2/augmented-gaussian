# Changelog

## Unreleased

### Added

- Added reconstruction method selection with native `voxel` and external `sugar` / `poisson` adapter modes.
- Added automatic timestamped output subdirectories for the default export root.
- Added WebAR `model-viewer` AR preview support for `occlusion.glb`.

### Changed

- Manifest JSON includes `schemaVersion = 2`, `outputDir`, and reconstruction metrics.

### BREAKING CHANGE

- Tools that parse `manifest.json` need schema version 2 support, including top-level `outputDir` and `metrics.reconstruction*` fields.
