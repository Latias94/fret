# fret-node-style-skinning-v3 TODO

## Design checks

- [ ] Verify paint-only invariants (no geometry/hit-test dependencies on paint overrides)
- [ ] Decide the minimal v1 override structs:
  - Edge: stroke paint/width mul/dash
  - Node: background/border paint (and optional header paint)
- [ ] Define deterministic normalization rules for override structs (finite checks; clamping)

## Implementation tasks

- [x] Add `NodeGraphPaintOverrides` + `NodeGraphPaintOverridesMap` in `ecosystem/fret-node`
- [x] Thread overrides handle through `NodeGraphCanvas` builder
- [x] Update paint cache keys to include paint override revision
- [x] Add conformance tests (invalidation + invariants)

## Optional (nice-to-have)

- [ ] Add debug overlay counters for “paint caches rebuilt due to overrides”
- [ ] Add a small JSON preset format for edge paint recipes (optional; do not couple to `Graph`)
