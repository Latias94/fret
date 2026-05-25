# Fret Node Paint Root Cache Plan Adapter v1 - Evidence And Gates

Status: Active
Last updated: 2026-05-25

## Smallest Current Repro

```bash
cargo check -p fret-node --features compat-retained-canvas
```

## Gate Set

### Scope Gate

```bash
python3 -m json.tool docs/workstreams/fret-node-paint-root-cache-plan-adapter-v1/WORKSTREAM.json
python3 tools/check_workstream_catalog.py
```

### Targeted Iteration Gate

```bash
cargo test -p fret-node --features compat-retained-canvas paint_root_cache_plan_adapter
```

### Package Gates

```bash
cargo check -p fret-node
cargo check -p fret-node --features compat-retained-canvas
```

### Boundary Gate

```bash
python3 tools/check_layering.py
git diff --check
```

## Evidence Anchors

- `docs/workstreams/fret-node-paint-prepaint-adapter-v1/CLOSEOUT_AUDIT_2026-05-25.md`
- `docs/workstreams/fret-node-paint-prepaint-adapter-v1/PAINT_ROOT_SCOPE_AUDIT_2026-05-25.md`
- `ecosystem/fret-node/src/ui/canvas/widget/paint_root/cache_plan.rs`
- `ecosystem/fret-node/src/ui/canvas/widget/paint_root/cache_plan_adapter.rs`
- `ecosystem/fret-node/src/ui/canvas/widget/paint_root/cache_plan_retained_cx.rs`
- `ecosystem/fret-node/src/lib.rs`

## Initial Scope Evidence - 2026-05-25

Claim:

- Cache-plan preparation is the smallest coherent paint-root adapter follow-on.
- The first implementation slice should isolate only host access, bounds, and scale factor.

Fresh validation:

- Pending first implementation slice.

Notes:

- Do not include frame setup, static layer replay/store, cached/immediate scene passes, or tail
  cleanup in CPA-020.

## CPA-010/CPA-020 Implementation Evidence - 2026-05-25

Claim:

- Cache-plan adapter scope is frozen to host access, bounds, and scale factor.
- `prepare_paint_root_cache_plan` now depends on a named retained-agnostic adapter seam instead of
  reading retained `PaintCx` fields directly.
- The retained `PaintCx` binding is isolated to `cache_plan_retained_cx.rs`.

Evidence:

- `cache_plan_adapter.rs` defines `PaintRootCachePlanCx`.
- `cache_plan_retained_cx.rs` implements `PaintRootCachePlanCx` for `PaintCx`.
- `cache_plan.rs` uses `paint_root_cache_plan_host`, `paint_root_cache_plan_bounds`, and
  `paint_root_cache_plan_scale_factor` for derived output, static-cache eligibility, tile sizing,
  style keys, and geometry key fallback.
- `surface_policy_tests::paint_root_cache_plan_adapter_keeps_route_inputs_off_retained_cx` keeps the
  adapter and cache-plan helper source off retained Cx names.

Fresh validation:

- `cargo test -p fret-node --features compat-retained-canvas paint_root_cache_plan_adapter` -
  passed; 1 test passed, 1162 filtered out.
- `cargo check -p fret-node` - passed.
- `cargo check -p fret-node --features compat-retained-canvas` - passed.
- `python3 -m json.tool docs/workstreams/fret-node-paint-root-cache-plan-adapter-v1/WORKSTREAM.json` -
  passed.
- `python3 tools/check_layering.py` - passed.
- `python3 tools/check_workstream_catalog.py` - passed; validated 446 dedicated directories and 47
  standalone markdown files.
- `git diff --check` - passed.
