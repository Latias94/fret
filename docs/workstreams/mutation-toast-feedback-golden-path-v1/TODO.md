# Mutation Toast Feedback Golden Path v1 - TODO

Status: closed closeout lane
Last updated: 2026-04-15

## Lane setup

- [x] Create a new narrow follow-on instead of reopening
      `executor-backed-mutation-surface-v1`.
- [x] Wire the new lane into `docs/roadmap.md`, `docs/todo-tracker.md`, and
      `docs/workstreams/README.md`.
- [x] Freeze that this lane does not change mutation ownership or Sonner ownership.

## M0 - Teaching-surface freeze

- [x] Name one realistic first-contact app slice instead of another generic Todo flow.
      Result: the new cookbook example models a Postman-style request preset save.
- [x] Freeze the teaching split:
      - mutation = authoritative submit owner,
      - app-owned locals/models = durable projection,
      - Sonner = feedback only.

## M1 - First implementation slice

- [x] Add a feature-gated cookbook example that teaches:
      - `cx.data().mutation_async(...)`,
      - `handle.submit(...)`,
      - `handle.retry_last(...)`,
      - and `cx.data().update_after_mutation_completion(...)`.
- [x] Keep the example off raw inbox/executor surfaces.
- [x] Add source-policy tests so the example cannot drift back to `toast_promise_async*` or raw
      executor/inbox plumbing.
- [x] Wire discoverability through:
      - `apps/fret-cookbook/README.md`,
      - `apps/fret-cookbook/EXAMPLES.md`,
      - `apps/fretboard/src/demos.rs`,
      - and `docs/examples/README.md`.
- [x] Add docs references from the general mutation docs to the new example.

## M2 - Diagnostics evidence

- [x] Add a screenshot-capable diag smoke for the success path.
- [x] Run the diag smoke once and leave a bounded artifact path in the lane notes / execution log.
      Result:
      `target/fret-diag/mutation-toast-feedback-basics/sessions/1776183542106-15688/screenshots/1776183582375-cookbook-mutation-toast-feedback-basics-smoke/window-4294967297-tick-46-frame-46.png`
      plus the paired bundle
      `target/fret-diag/mutation-toast-feedback-basics/sessions/1776183542106-15688/1776183582450-cookbook-mutation-toast-feedback-basics-smoke/`.

## M3 - Closeout decision

- [x] Decide whether the lane can close after the first screenshot smoke and source/doc gates stay
      green.
      Result: yes. `CLOSEOUT_AUDIT_2026-04-15.md` now records the shipped verdict and this lane is
      closed.
- [x] Start a different narrow follow-on instead of widening this lane if future pressure becomes:
      - query invalidation teaching depth,
      - Sonner API redesign,
      - or another broader async surface rename debate.
      Result: `WORKSTREAM.json` now freezes `start_follow_on` as the default future action.
