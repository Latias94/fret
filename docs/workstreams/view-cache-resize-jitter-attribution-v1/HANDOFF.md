# ViewCache Resize-Jitter Attribution v1 - Handoff

Status: Active
Last updated: 2026-05-18

## Current State

The lane is open and scoped as a narrow follow-on after `pressable-clean-geometry-propagation-v1`.
No runtime code has been changed.

Starting evidence:

- `target/fret-diag/pressable-clean-geometry-propagation-v1-pgp050-after/1779088062238/bundle.schema2.json`
- `target/fret-diag/pressable-clean-geometry-propagation-v1-pgp050-after/layout.perf.summary.v1.json`

Starting hotspot verdict:

- `ViewCache layout_us=380 inclusive_us=723`
- `Scroll layout_us=205 inclusive_us=331`
- `Flex layout_us=83 inclusive_us=122`

## Next Task

Run VCRJ-020.

Goal:

- Audit `ViewCache` source ownership before touching runtime code.
- Decide whether the hotspot is a cache-root contained relayout, invalidation breadth, root-bound
  repair, scroll follow-up side effect, diagnostics attribution artifact, or demo composition issue.

Start with:

```bash
rg -n "ViewCache|view_cache|ViewBoundaryKind::ViewCacheRoot|contained_relayout|cache_root" \
  crates/fret-ui/src/element.rs \
  crates/fret-ui/src/elements/runtime.rs \
  crates/fret-ui/src/tree/view_boundary.rs \
  crates/fret-ui/src/tree/layout \
  -S
```

## Guardrails

- Do not add `ElementInstance::ViewCache(_)` to the clean-geometry allowlist as a first move.
- Keep `Scroll` as a separate possible follow-on.
- Keep UI Gallery recipe changes out unless evidence proves the demo composition owns the cost.
- Preserve cache-root liveness, state retention, boundary tracing, and scroll extent repair.
