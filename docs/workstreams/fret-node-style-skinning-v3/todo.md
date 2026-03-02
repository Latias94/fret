# fret-node-style-skinning-v3 TODO

## Design checks

- [x] Verify paint-only invariants (no geometry/hit-test dependencies on paint overrides)
- [x] Decide the minimal v1 override structs:
  - Edge: stroke paint/width mul/dash
  - Node: background/border paint (and optional header paint)
- [x] Define deterministic normalization rules for override structs (finite checks; clamping)

## Implementation tasks

- [x] Add `NodeGraphPaintOverrides` + `NodeGraphPaintOverridesMap` in `ecosystem/fret-node`
- [x] Thread overrides handle through `NodeGraphCanvas` builder
- [x] Update paint cache keys to include paint override revision
- [x] Add conformance tests (invalidation + invariants)
- [x] Apply per-edge paint overrides to emitted wire `SceneOp::Path.paint`
- [x] Apply per-node paint overrides to emitted node body/background paint
- [x] Conformance: paint overrides do not mutate serialized `Graph`

## Optional (nice-to-have)

- [ ] Add debug overlay counters for “paint caches rebuilt due to overrides”
- [x] Add a small JSON preset format for theme presets (paint-only; do not couple to `Graph`)
- [x] Capture a XyFlow parity crosswalk note (`xyflow-style-parity.md`)
