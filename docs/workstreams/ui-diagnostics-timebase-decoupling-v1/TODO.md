# UI diagnostics timebase decoupling (v1) — TODO

Scope: eliminate “no redraw callbacks → script never finishes → tooling timeout waiting for `script.result`” classes of
failures, with a focus on multi-window docking and other occlusion-heavy interactions.

## Priority (proposed)

P0 (blocker for docking automation):

- Adopt at least one no-frame liveness regression gate in a suite/CI-like entrypoint (so we do not regress to tooling
  timeouts).
- Audit and document which steps are allowed to advance on the no-frame path (keep it conservative and bounded).

P1:

- Improve evidence on no-frame failures (so triage does not require raw bundle inspection).
- Maintain “shutdown outcome” evidence for tool-launched runs (shipped):
  - write `resource.footprint.json` in the out dir (tooling-owned),
  - use `killed=true` as a first-class triage hint for “exit trigger not observed / deadlock / no-frame stall”.

P2:

- Consider schema evolution (`wait_ms`, `timeout_ms`) once we have enough evidence from real failures.

## Immediate TODOs (next)

- Wire `tools/diag-scripts/diag/no-frame/diag-no-frame-timeout-no-frames.json` into a minimal regression entrypoint
  (suite/smoke gate) and document how to run it locally.
- Add a docking-adjacent repro that intentionally occludes a relevant window for > 1s and still completes or fails
  deterministically (bounded bundle + stable `reason_code`).
- Update docs when we change the no-frame allowed-step set (to keep the contract honest).

## Backlog

- Improve evidence on no-frame failure:
  - last known `step_kind`,
  - last known window snapshot age at failure,
  - whether the no-frame driver advanced anything (count + last advanced step).
- Make `timeout.no_frames` show up as a first-class triage hint in tooling summaries.
