# Paint Root Frame Setup Scope Audit - 2026-05-25

Status: FSA-020 complete

## Verdict

Do not implement a broad frame setup adapter yet.

`prepare_paint_root_frame` mixes several operation families. The first implementation candidate
should be bounds/viewport route inputs because it can remove direct `PaintCx::bounds` reads without
touching scene mutation, diagnostics registry writes, skin graph reads, or grid rendering.

## Evidence

- `paint_root/frame.rs` begins paint caches, records path-cache diagnostics, computes viewport and
  render-cull geometry from `cx.bounds`, emits a clip scene op, paints the background, then paints
  the grid.
- `paint_root/frame/cache.rs` has two families: cache begin with no retained context, and path-cache
  diagnostics using `cx.window`, `cx.node`, `cx.app.frame_id()`, and
  `cx.app.with_global_mut(CanvasCacheStatsRegistry::default, ...)`.
- `paint_root/frame/background.rs` resolves canvas chrome through
  `paint_grid_plan_support::resolve_canvas_chrome_hint`, which reads the graph through `cx.app`,
  then emits a background quad to `cx.scene`.
- `paint_grid.rs` starts the grid cache frame, prepares grid paint, warms grid tiles, and records
  grid tile stats; this remains a separate grid paint family.

## Operation Families

| Family | Primary files | Retained context use | Split decision |
| --- | --- | --- | --- |
| Cache frame begin | `paint_root/frame/cache.rs` | None | No adapter needed; keep as direct canvas state bookkeeping. |
| Path-cache diagnostics | `paint_root/frame/cache.rs` | `window`, `node`, `app.frame_id`, `app.with_global_mut` | Separate candidate if diagnostics context seams become a priority. |
| Bounds/viewport/render-cull | `paint_root/frame.rs`, `widget_surface/runtime/render.rs`, `view_math_viewport/viewport.rs` | `bounds` only at call site | Best first adapter proof; no scene mutation. |
| Clip scene emission | `paint_root/frame.rs` | `scene` | Defer; scene command emission should be split separately. |
| Background paint | `paint_root/frame/background.rs`, `paint_grid_plan_support/hint.rs` | `app` for skin graph read, `scene` for quad | Defer; combines policy lookup and scene emission. |
| Grid paint | `paint_grid.rs`, `paint_grid_plan.rs`, `paint_grid_cache.rs`, `paint_grid_stats.rs` | `PaintCx` through plan/cache/stats | Separate grid-paint adapter or audit lane. |

## Follow-On Recommendation

Keep this lane active for the first frame seam and implement a bounds/viewport adapter proof next.

Suggested next task:

- Add a named frame-viewport context seam exposing only paint bounds.
- Move viewport and render-cull computation behind that seam.
- Keep cache stats diagnostics, clip emission, background paint, and grid paint out of scope.

Suggested source-policy test:

- `cargo test -p fret-node --features compat-retained-canvas paint_root_frame_viewport_adapter`

Suggested gates:

- `cargo check -p fret-node`
- `cargo check -p fret-node --features compat-retained-canvas`
- `python3 tools/check_layering.py`
- `python3 tools/check_workstream_catalog.py`
- `git diff --check`
