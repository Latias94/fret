# UI Gallery view-cache web perf stabilization (v1) — Milestones

## M1 — Reproducible configuration (landed)

- [x] Web URL flags for UI Gallery view-cache configuration on `wasm32`.
  - Evidence:
    - `apps/fret-ui-gallery/src/driver/legacy.rs` (`bool_from_window_query`, `config_bool`)
    - `apps/fret-ui-gallery/Cargo.toml` (enable `web-sys/UrlSearchParams`)
- [x] On `wasm32`, when view-cache is explicitly enabled, default shell view-cache on (unless overridden).
  - Evidence: `apps/fret-ui-gallery/src/driver/legacy.rs` (view-cache shell default)
- [x] Avoid per-frame model churn for undo/redo availability (keeps view-cache roots stable and reduces layout invalidations).
  - Evidence: `apps/fret-ui-gallery/src/driver/render_flow.rs` (write only on change)

## M2 — Evidence capture (pending)

- [x] Baseline bundle (web, view-cache disabled):
  - Evidence: `.fret/diag/exports/1771829809968-bundle`
- [x] Experimental bundle (web, view-cache enabled):
  - Evidence: `.fret/diag/exports/1771832191642-bundle` (view-cache active, but `view_cache_roots_total=0`)
- [ ] Post-fixes experimental bundle (web, view-cache enabled + shell default + churn fix):
  - Evidence: `.fret/diag/exports/<ts>-bundle` (path only)
- [x] Post-churn-fix experimental bundle (web, view-cache enabled):
  - Evidence: `.fret/diag/exports/1771835082078-bundle`
  - Notes: `model_changes=0`, `paint_cache_misses≈2`, but `view_cache_roots_total=0` still (shell roots not being mounted yet).

## M3 — Decision (pending)

- [ ] Decide whether web perf evidence URLs should default to enabling view-cache (policy decision for harness).
- [ ] If enabling, document the recommended URL flags in the renderer vnext workstream as “evidence harness defaults”.
