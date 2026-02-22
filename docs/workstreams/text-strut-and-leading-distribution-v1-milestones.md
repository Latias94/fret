# Milestones: Text Strut + Leading Distribution v1

This is **non-normative** and tracks delivery progress.

Status key:
- Done: exit criteria met for v1 scope.
- Partial: core landed, follow-ups tracked in TODO.

## M0 — Draft surface + ownership clarified

Status: Partial.

Exit criteria:

- Mechanism vs ecosystem ownership is explicit (types live in `crates/fret-core`).
- Workstream has acceptance criteria and evidence anchors.

Evidence:
- `docs/workstreams/text-strut-and-leading-distribution-v1.md`

## M1 — Core mechanism types exist

Status: Done.

Exit criteria:

- `crates/fret-core` exports the v1 types.
- Types are wired into `TextStyle`/paragraph style without breaking existing callsites.

Evidence:
- `crates/fret-core/src/text/mod.rs`
- `crates/fret-core/src/lib.rs`

## M2 — Render-text implementation complete

Status: Partial.

Exit criteria:

- Parley shaping/layout enforces strut metrics when enabled.
- Cache keys include strut fields.

Evidence:
- `crates/fret-render-text/src/parley_shaper.rs`
- `crates/fret-render-text/src/cache_keys.rs`
- `crates/fret-render-text/src/measure.rs`

## M3 — Regression gates in place

Status: Done.

Exit criteria:

- A bundled-font regression test covers multiline stability with emoji/fallback runs.

Evidence:
- `crates/fret-render-text/src/wrapper.rs` (`strut_force_keeps_multiline_baseline_stable_across_fallback_glyphs`)

## M4 — Ecosystem opt-in adopted for text areas

Status: Done.

Exit criteria:

- `fret-ui-kit::typography` exposes a clear opt-in for multiline stable line boxes.
- At least one real surface (text area / form) adopts it.

Evidence:
- `ecosystem/fret-ui-shadcn/src/textarea.rs`
