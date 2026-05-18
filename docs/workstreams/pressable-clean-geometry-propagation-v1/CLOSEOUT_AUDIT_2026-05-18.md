# Pressable Clean Geometry Propagation v1 - Closeout Audit - 2026-05-18

## Verdict

Close this lane.

`Pressable` is safe for the targeted clean-geometry propagation path after the recorded source audit,
RED proof, minimal runtime slice, interaction gates, and fresh UI Gallery resize-jitter confirmation.

## Shipped Runtime Decision

- `ElementInstance::Pressable(_)` is included in
  `clean_engine_geometry_propagation_supported_element(...)`.
- The change stays in `crates/fret-ui`; no component policy or ecosystem recipe behavior moved into
  the runtime.
- `Scroll`, `ViewCache`, text wrapping, and broader clean-geometry expansion remain out of scope.

## Evidence

Focused proof:

- `cargo nextest run -p fret-ui clean_geometry_small_resize_propagates_through_pressable_wrapper --no-fail-fast`
- `cargo nextest run -p fret-ui layout_engine pressable --no-fail-fast`
- `cargo nextest run -p fret-ui pressable_on_activate_hook_runs_on_pointer_activation pressable_on_hover_change_hook_runs_on_pointer_move pressable_clears_pressed_and_releases_capture_on_move_without_buttons --no-fail-fast`

Fresh local perf confirmation:

- Bundle:
  `target/fret-diag/pressable-clean-geometry-propagation-v1-pgp050-after/1779088062238/bundle.schema2.json`
- Layout summary:
  `target/fret-diag/pressable-clean-geometry-propagation-v1-pgp050-after/layout.perf.summary.v1.json`
- Stats command:
  `target/release/fretboard-dev diag stats target/fret-diag/pressable-clean-geometry-propagation-v1-pgp050-after/1779088062238/bundle.schema2.json --sort time --top 20`

Fresh worst-frame layout hotspots:

```text
ViewCache layout_us=380 inclusive_us=723
Scroll    layout_us=205 inclusive_us=331
Flex      layout_us=83  inclusive_us=122
```

Historical RLO-030 after hotspots:

```text
Pressable layout_us=308 inclusive_us=684
Scroll    layout_us=199 inclusive_us=287
ViewCache layout_us=76  inclusive_us=375
```

## Interpretation

- The local `Pressable` hotspot moved after PGP-040; it is no longer in the worst-frame layout
  hotspot list for the shared resize-jitter repro.
- The single-run tail frame is roughly flat against the RLO-030 after bundle:
  `p95.total_time_us=1442 -> 1477`, `p95.layout_time_us=885 -> 930`, and
  `p95.layout_engine_solve_time_us=214 -> 215`.
- This closeout proves owner movement and mechanism correctness. It does not claim a universal
  frame-time improvement.

## Follow-Ons

- `ViewCache` should be its own lane if it remains the top local owner after another fresh capture.
- `Scroll` should be its own lane because it owns viewport, clipping, offset, and input semantics.
- Broader clean-geometry expansion should stay evidence-led and owner-specific.

## Final Status

Closed on 2026-05-18.
