# Component conformance playbook: shadcn Select

Goal: do not snapshot every internal state. Gate a small set of stable invariants and make failures self-diagnosing via evidence.

## What to gate (invariants)

- Open/close lifecycle:
  - trigger opens overlay content,
  - trigger toggles close,
  - outside press / Escape dismisses,
  - close restores focus predictably.
- Selection outcome:
  - click/keyboard commit updates the trigger label,
  - disabled items do not commit.
- Routing correctness:
  - injected pointer lands on the intended target (or produces an explainable hit-test trace),
  - capture/occlusion never “mysteriously” blocks underlay without evidence.
- Placement sanity (geometry-first):
  - overlay stays within window/viewport (use `bounds_within_window`),
  - placement decisions are explainable (use `overlay_placement_trace`).
- Scroll/virtualization stability (when applicable):
  - wheel/key scroll affects content (post-run gate like `--check-wheel-scroll-hit-changes`),
  - `click_stable` avoids stale-coordinate clicks on jumping bounds.

## Script authoring tips

- Prefer stable `test_id` at the recipe/component layer (trigger/content/items/viewport).
- Replace sleeps with intent steps:
  - `click_stable` for jittery targets,
  - `wait_bounds_stable` for “overlay settled” phases (flip/shift, estimate→measured).
- Put one `capture_bundle` near the “interesting” step so failures are explainable without rerunning.

## Run the suite

- `cargo run -p fretboard -- diag suite ui-gallery-select --launch -- cargo run -p fret-ui-gallery --release`

## Typed script template (optional)

If you want a typed Rust source-of-truth for the suite scripts (while keeping JSON as the portable artifact):

- Check that all suite scripts match their templates:
  - `cargo run -p fret-diag-scriptgen -- check-suite ui-gallery-select`
- Optional: inspect or check an individual template:
  - `cargo run -p fret-diag-scriptgen -- list`
  - `cargo run -p fret-diag-scriptgen -- print ui-gallery-select-open-jitter-click-stable-v2`
  - `cargo run -p fret-diag-scriptgen -- check ui-gallery-select-open-jitter-click-stable-v2 tools/diag-scripts/ui-gallery-select-open-jitter-click-stable-v2.json`

Notes:

- `diag suite` runs `diag lint` by default; disable with `--no-lint`.
- On failures, read `script.result.json` first, then jump to `hit_test_trace` / `overlay_placement_trace` / `bounds_stable_trace`.
