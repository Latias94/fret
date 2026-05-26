# Paint Root Scope Audit - 2026-05-25

Status: NPA-030 complete

## Verdict

Do not implement a broad paint-root adapter in this lane.

`paint_retained_widget` is small, but the call below it (`canvas.paint_root(cx)`) crosses multiple
operation families that still depend on `PaintCx` directly. Treating the whole paint root as one
adapter would create a large trait that hides the retained dependency instead of isolating it.

## Evidence

- `retained_widget_runtime_paint.rs` owns only lifecycle theme sync and root paint dispatch.
- `paint_root/cached.rs` combines model observation, view-state sync, frame preparation, cache-plan
  preparation, cached pass selection, immediate pass fallback, and tail cleanup.
- `paint_root/frame.rs` combines cache-frame bookkeeping, viewport math, clip emission, background
  paint, and grid paint.
- `paint_root/cache_plan.rs` computes derived geometry/index output, publishes internals, derives
  static-cache eligibility, computes tile rects, and builds style/geometry cache keys.
- Cached groups/nodes/edges/labels paths use scene mutation, static scene cache replay/store,
  paint-cache touch paths, text blob touch paths, services, scale factor, and host reads.

## Operation Families

| Family | Primary files | Retained context use | Split decision |
| --- | --- | --- | --- |
| Paint lifecycle dispatch | `retained_widget_runtime_paint.rs` | `theme()`, `services`, root dispatch | Too small alone to justify this lane; can be folded into a later runtime cleanup. |
| Root route preparation | `paint_root/cached.rs` | `observe_model`, `app` for view-state sync | Separate candidate if observation/view sync needs an adapter. |
| Frame scene setup | `paint_root/frame.rs`, `paint_root/frame/*` | `bounds`, `scene`, `window`, `node`, `app`, `services` | Split after cache-plan proof; includes scene mutation and diagnostics. |
| Cache-plan preparation | `paint_root/cache_plan.rs`, `paint_root/cache_plan/*` | `app`, `bounds`, `scale_factor` | Best next paint adapter proof: coherent, no direct scene mutation. |
| Static layer replay/store | `paint_root/static_layer.rs`, `paint_root/static_cache.rs` | `scene`, cache stores, paint cache touch callbacks | Split separately; cache replay and scene emission are tightly coupled. |
| Cached/immediate passes | `paint_root/cached_pass.rs`, `paint_root/immediate*.rs`, `paint_root/cached_*` | `scene`, `services`, `scale_factor`, `app` | Too broad for one adapter; split by layer family. |
| Tail cleanup | `paint_root/tail.rs`, `paint_root/prune.rs` | `services`, `scene` | Split only after scene/cache adapters exist. |

## Follow-On Recommendation

Open a narrower follow-on for a paint-root cache-plan adapter proof.

Suggested slug:

- `fret-node-paint-root-cache-plan-adapter-v1`

Suggested first task:

- Introduce a retained-agnostic cache-plan context seam for host access, bounds, and scale factor.
- Move `prepare_paint_root_cache_plan` route inputs behind that seam.
- Keep frame setup, scene emission, static layer replay/store, cached/immediate passes, and tail
  cleanup out of scope.

Suggested gates:

- `cargo test -p fret-node --features compat-retained-canvas paint_root_cache_plan_adapter`
- `cargo check -p fret-node`
- `cargo check -p fret-node --features compat-retained-canvas`
- `python3 tools/check_layering.py`
- `python3 tools/check_workstream_catalog.py`
- `git diff --check`
