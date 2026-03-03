# How to debug layout (semantics-first + sidecars)

This is the recommended workflow to debug layout regressions without adding ad-hoc debug UI to demo apps.

The key idea: use **semantics** as the stable "what" (selectors + bounds predicates), and use a **layout sidecar**
as the deep "why" (Taffy subtree dump) when needed.

## 1) Start with a semantics-first repro (portable)

Prefer a deterministic script that:

- navigates to the relevant page/state,
- asserts the expected geometry using bounds predicates,
- dumps a bundle on failure (and optionally always dumps a bundle at the end).

Useful building blocks:

- Pick stable targets with `test_id` (inspect/pick), then update scripts via `diag pick-apply`.
- Use `bounds_within_window` / `bounds_equals` style predicates instead of screenshots.

## 2) Attach a layout sidecar when "why" matters (native only, best-effort)

If a layout assertion fails and you need the exact layout engine state, request a bundle-scoped sidecar:

- Script step: `capture_layout_sidecar` (schema v2; see `LAYOUT_SIDECARS_V1.md`)
- Output: `<bundle_dir>/layout.taffy.v1.json`

Notes:

- Sidecars are intentionally bounded and clipped; treat them as explainability artifacts.
- Sidecars are not a substitute for semantics-based gates; they exist to accelerate diagnosis.

## 3) View the sidecar (CLI)

Given a bundle directory (or a bundle artifact path), open the sidecar viewer:

- `cargo run -p fretboard -- diag layout-sidecar <bundle_dir>`

If you only need the resolved sidecar path:

- `cargo run -p fretboard -- diag layout-sidecar <bundle_dir> --print-path`

## 4) When it is perf, not correctness

If the issue is "layout got slower" rather than "layout is wrong":

1. Run a perf suite that is layout-heavy:
   - `cargo run -p fretboard -- diag perf ui-gallery-layout-steady --repeat 7 --warmup-frames 5 --sort time --json`
2. For the worst bundle, generate a layout perf summary:
   - `cargo run -p fretboard -- diag layout-perf-summary <bundle_dir>`

The perf harness also best-effort attaches a bounded layout perf summary to:

- `<out_dir>/check.perf_thresholds.json`
- `<out_dir>/check.perf_hints.json`

## 5) Optional escape hatch: ad-hoc Taffy dumps

When you need immediate local inspection (not CI artifacts), enable ad-hoc dumps:

- `FRET_TAFFY_DUMP=1`
- Optional filters:
  - `FRET_TAFFY_DUMP_ROOT=<id>`
  - `FRET_TAFFY_DUMP_ROOT_LABEL=<label>`
  - `FRET_TAFFY_DUMP_DIR=.fret/taffy-dumps`

This is not a replacement for sidecars; prefer sidecars for reproducible bug reports.
