# Mutation Toast Feedback Golden Path v1 - Evidence & Gates

Goal: keep the mutation-plus-feedback teaching lane tied to one cookbook example, one screenshot
smoke, and one focused source-policy gate instead of letting it become another vague async backlog.

## Evidence anchors (current)

- `docs/workstreams/mutation-toast-feedback-golden-path-v1/DESIGN.md`
- `docs/workstreams/mutation-toast-feedback-golden-path-v1/TODO.md`
- `docs/workstreams/mutation-toast-feedback-golden-path-v1/MILESTONES.md`
- `docs/workstreams/mutation-toast-feedback-golden-path-v1/WORKSTREAM.json`
- `docs/workstreams/executor-backed-mutation-surface-v1/CLOSEOUT_AUDIT_2026-04-14.md`
- `docs/adr/0326-query-vs-mutation-read-vs-submit-default-app-lane-v1.md`
- `apps/fret-cookbook/Cargo.toml`
- `apps/fret-cookbook/examples/mutation_toast_feedback_basics.rs`
- `apps/fret-cookbook/src/lib.rs`
- `apps/fret-cookbook/README.md`
- `apps/fret-cookbook/EXAMPLES.md`
- `apps/fretboard/src/demos.rs`
- `tools/diag-scripts/cookbook/mutation-toast-feedback-basics/cookbook-mutation-toast-feedback-basics-smoke.json`
- `tools/diag-scripts/suites/cookbook-mutation-toast-feedback-basics/suite.json`
- `docs/examples/README.md`
- `docs/crate-usage-guide.md`
- `docs/integrating-sqlite-and-sqlx.md`

## First-open repro surfaces

Use these before reading older historical mutation-lane notes in depth:

1. Cookbook example (human-facing)
   - `cargo run -p fretboard-dev -- dev native --example mutation_toast_feedback_basics`
2. Focused source-policy proof
   - `cargo nextest run -p fret-cookbook mutation_toast_feedback_example_keeps_submit_and_feedback_projection_split`
3. Screenshot smoke
   - `FRET_DIAG=1 cargo run -p fretboard-dev -- diag run tools/diag-scripts/cookbook/mutation-toast-feedback-basics/cookbook-mutation-toast-feedback-basics-smoke.json --dir target/fret-diag/mutation-toast-feedback-basics --session-auto --launch -- cargo run -p fret-cookbook --features cookbook-mutation,cookbook-diag --example mutation_toast_feedback_basics`

## Current focused gates

### Cookbook source-policy gate

- `cargo nextest run -p fret-cookbook mutation_toast_feedback_example_keeps_submit_and_feedback_projection_split`

This gate currently proves:

- the example stays on `cx.data().mutation_async(...)`,
- submit/retry stay on `handle.submit(...)` / `handle.retry_last(...)`,
- `cx.data().update_after_mutation_completion(...)` remains the projection lane,
- Sonner stays on `toast_success_message(...)` / `toast_error_message(...)`,
- and the example does not drift back to `toast_promise_async*` or raw inbox/executor teaching.

### Example compile gate

- `cargo check -p fret-cookbook --features cookbook-mutation --example mutation_toast_feedback_basics`

This gate currently proves:

- the feature wiring stays minimal and explicit,
- the example still compiles without accidentally widening the default cookbook dependency set,
- and `fretboard-dev` can keep offering a stable feature hint.

### Screenshot smoke gate

- `FRET_DIAG=1 cargo run -p fretboard-dev -- diag run tools/diag-scripts/cookbook/mutation-toast-feedback-basics/cookbook-mutation-toast-feedback-basics-smoke.json --dir target/fret-diag/mutation-toast-feedback-basics --session-auto --launch -- cargo run -p fret-cookbook --features cookbook-mutation,cookbook-diag --example mutation_toast_feedback_basics`

This gate currently proves:

- the example boots under diagnostics,
- the success-path save still emits a toast,
- and the repo can keep one screenshot/bundle artifact for visual review.

Most recent successful run on 2026-04-14:

- Screenshot:
  `target/fret-diag/mutation-toast-feedback-basics/sessions/1776183542106-15688/screenshots/1776183582375-cookbook-mutation-toast-feedback-basics-smoke/window-4294967297-tick-46-frame-46.png`
- Bundle:
  `target/fret-diag/mutation-toast-feedback-basics/sessions/1776183542106-15688/1776183582450-cookbook-mutation-toast-feedback-basics-smoke/`

### Lane hygiene gates

- `python3 tools/check_workstream_catalog.py`
- `python3 .agents/skills/fret_skills.py validate --strict --check-anchors --check-symbols`
- `python3 -m json.tool docs/workstreams/mutation-toast-feedback-golden-path-v1/WORKSTREAM.json > /dev/null`
- `git diff --check`

## Missing evidence before closure

Before closing this lane, leave:

- one successful screenshot smoke artifact path,
- and one short closeout note that says this lane is now the first-open teaching answer while the
  closed executor-backed lane remains only the owner-split background.

Do not respond to future drift here by widening Sonner into a submit owner or by reopening the
shared mutation contract without a different narrow follow-on.
