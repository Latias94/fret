# UI Gallery view-cache web perf stabilization (v1) — Milestones

## M1 — Reproducible configuration (landed)

- [x] Web URL flags for UI Gallery view-cache configuration on `wasm32`.
  - Evidence:
    - `apps/fret-ui-gallery/src/driver/legacy.rs` (`bool_from_window_query`, `config_bool`)
    - `apps/fret-ui-gallery/Cargo.toml` (enable `web-sys/UrlSearchParams`)

## M2 — Evidence capture (pending)

- [ ] Baseline bundle (web, view-cache disabled):
  - Evidence: `.fret/diag/exports/<ts>-bundle` (path only)
- [ ] Experimental bundle (web, view-cache enabled):
  - Evidence: `.fret/diag/exports/<ts>-bundle` (path only)

## M3 — Decision (pending)

- [ ] Decide whether web perf evidence URLs should default to enabling view-cache (policy decision for harness).
- [ ] If enabling, document the recommended URL flags in the renderer vnext workstream as “evidence harness defaults”.

