# Mutation Toast Feedback Golden Path v1 - Milestones

Status: active execution lane
Last updated: 2026-04-14

## M0 - Lane and owner freeze

Exit criteria:

- the repo explicitly states why this is a new narrow follow-on,
- the closed executor-backed mutation lane remains the contract background rather than the active
  implementation surface,
- and the first realistic proof target is named.

Primary evidence:

- `DESIGN.md`
- `WORKSTREAM.json`
- `docs/workstreams/executor-backed-mutation-surface-v1/CLOSEOUT_AUDIT_2026-04-14.md`

Current status:

- Closed on 2026-04-14 via the lane-opening docs.

## M1 - Copyable teaching surface lands

Exit criteria:

- the repo has one small first-party example that composes mutation ownership with toast feedback,
- the example avoids raw inbox/executor and `toast_promise_async*`,
- and cookbook/source-policy gates freeze the split.

Primary evidence:

- `apps/fret-cookbook/examples/mutation_toast_feedback_basics.rs`
- `apps/fret-cookbook/src/lib.rs`
- `apps/fret-cookbook/Cargo.toml`
- `apps/fret-cookbook/README.md`
- `apps/fret-cookbook/EXAMPLES.md`
- `apps/fretboard/src/demos.rs`
- `docs/examples/README.md`
- `docs/crate-usage-guide.md`
- `docs/integrating-sqlite-and-sqlx.md`

Current status:

- Closed on 2026-04-14.
- The repo now has a first-party Postman-style request preset save example that keeps:
  - `fret-mutation` authoritative,
  - app-owned locals as the durable projection,
  - and Sonner as feedback only.

## M2 - Diagnostics proof floor

Exit criteria:

- a dedicated diag script captures at least one success-path screenshot and bundle,
- and the script is discoverable from the cookbook example index.

Primary evidence:

- `tools/diag-scripts/cookbook/mutation-toast-feedback-basics/cookbook-mutation-toast-feedback-basics-smoke.json`
- `tools/diag-scripts/suites/cookbook-mutation-toast-feedback-basics/suite.json`
- `apps/fret-cookbook/EXAMPLES.md`
- future run artifact under `target/fret-diag/mutation-toast-feedback-basics/`

Current status:

- In progress.
- The script and suite are now landed.
- The first bounded run artifact is now captured under:
  - `target/fret-diag/mutation-toast-feedback-basics/sessions/1776183542106-15688/screenshots/1776183582375-cookbook-mutation-toast-feedback-basics-smoke/window-4294967297-tick-46-frame-46.png`
  - `target/fret-diag/mutation-toast-feedback-basics/sessions/1776183542106-15688/1776183582450-cookbook-mutation-toast-feedback-basics-smoke/`

## M3 - Closure or split

Exit criteria:

- the lane either closes with a stable cookbook/doc/diag golden path,
- or splits again because the remaining pressure is really about another owner or another product
  surface.

Primary evidence:

- `WORKSTREAM.json`
- `TODO.md`
- `EVIDENCE_AND_GATES.md`
- future closeout note

Current status:

- In progress.
- No mechanism widening is warranted yet; the remaining work is purely evidence capture and closure
  discipline.
