# UI diagnostics timebase decoupling (v1) — TODO

Scope: eliminate “no redraw callbacks → script never finishes → tooling timeout waiting for `script.result`” classes of
failures, with a focus on multi-window docking and other occlusion-heavy interactions.

## Priority (proposed)

P0 (blocker for docking automation):

- M1 Pending-script liveness: make pending scripts start deterministically even if the app goes idle between runs.
- M2 Timeout semantics contract: document and lock how `timeout_frames` behaves when frames are not advancing.

P1:

- Add a small “no-frame liveness” regression suite:
  - force an occlusion/idle scenario,
  - verify `timeout.no_frames` appears (not a tooling timeout),
  - verify at least one bounded evidence bundle is captured.

P2:

- Consider schema evolution (`wait_ms`, `timeout_ms`) once we have enough evidence from real failures.

## Immediate TODOs (next)

- Write a short contract note (1–2 pages):
  - when timers are armed (pending vs active),
  - which steps are allowed to advance without frames,
  - and what the expected failure modes are (`timeout.no_frames` vs `timeout`).
- Decide whether tooling should enable `script_keepalive` in `--launch` config for script-driven runs (if useful for M1).
- Add a minimal docking-adjacent repro that intentionally occludes the main window for > 1s and still completes or fails
  deterministically (bundle + `reason_code`).

## Backlog

- Improve evidence on no-frame failure:
  - last known `step_kind`,
  - last known window snapshot age at failure,
  - whether the no-frame driver advanced anything (count + last advanced step).
- Make `timeout.no_frames` show up as a first-class triage hint in tooling summaries.

