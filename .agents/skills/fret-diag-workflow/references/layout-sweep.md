# Layout sweep playbook (UI gallery)

Goal: catch “self-drawn layout correctness” regressions (overflow, clipping, bounds outside the window, zero-size
targets) without relying on pixel diffs.

## What to gate

Prefer stable, explainable invariants:

- no error-level `diag lint` findings (duplicate `test_id`, focused out-of-window, empty bounds on targeted nodes),
- key roots stay within the window bounds (`bounds_within_window` / bundle lint),
- no obvious hover-layout thrash (optional: `--check-hover-layout` gates).

Use screenshots only as supporting evidence (for UI review or when a rendering bug can’t be explained from geometry).

## How to run

Recommended (native, launches a fresh process):

- `cargo run -p fretboard -- diag suite ui-gallery-layout --launch -- cargo run -p fret-ui-gallery --release`

Run a single sweep script:

- `cargo run -p fretboard -- diag run tools/diag-scripts/ui-gallery-layout-sweep-core.json --launch -- cargo run -p fret-ui-gallery --release`

## What to read when it fails

1. `script.result.json` (reason code + last bundle dir).
2. `check.lint.json` (error-level findings should be actionable without screenshots).
3. `triage.json` / `check.triage.json` (high-level summary, useful for AI triage).
4. Only then: screenshots (if enabled) to confirm the visual symptom.

Tips:

- Use `diag lint --all-test-ids` when you want window-boundary hints for all targeted nodes, not only focused ones.
- If a failure is “jitter”, prefer `click_stable` / `wait_bounds_stable` in the script rather than adding sleeps.

