# ImUi ID Stack Diagnostics v1 - Milestones

Status: closed
Last updated: 2026-04-28

## M0 - Tracking

Exit criteria:

- the lane is documented as a narrow follow-on of the closed label identity work,
- repo-level indexes point to the new workstream,
- and the first slice is limited to structured diagnostics, not public identity API growth.

## M1 - Structured Runtime Diagnostics

Result:

- duplicate keyed-list hash warnings are stored in the diagnostics snapshot,
- unkeyed reorder warnings are stored in the diagnostics snapshot,
- source location and debug path evidence are present when diagnostics are enabled,
- and existing tracing warnings remain intact.

## M2 - IMUI Authoring Proof

Result:

- `fret-imui` can run diagnostics-enabled tests,
- `ui.for_each_unkeyed` reorder reaches the runtime structured warning,
- and the proof explains that dynamic collections should move to `ui.for_each_keyed` / `ui.id`.

## M3 - Diagnostics Query Surface

Result:

- `fretboard diag query identity-warnings` reads existing schema2 bundle snapshots and extracts
  `debug.element_runtime.identity_warnings`,
- the query supports bounded filters for kind, window, element, list id, element path, and source
  file,
- default output de-duplicates repeated snapshot observations while `--timeline` preserves the full
  snapshot history,
- and the surface is covered by handler, clap contract, and cutover conversion tests.

## M4 - IMUI Duplicate-Key Authoring Proof

Result:

- `ImUi::for_each_keyed` delegates to `ElementContext::for_each_keyed` so keyed-list identity uses
  the same runtime list scope and diagnostics recorder as declarative UI,
- the IMUI API now mirrors the runtime shape with `items: &[T]`, `key(&T) -> K`, and
  `f(ui, index, &T)`,
- duplicate keyed-list hashes are observable from the IMUI authoring callsite,
- and explicit subtree identity still uses `ui.id(...)` / `ui.push_id(...)`.

## M5 - Follow-on Decision

Result:

- final gates are recorded,
- `WORKSTREAM.json` reflects current lane state,
- and deferred work is split into narrower lanes instead of widening this one by default.

Current deferred candidates:

- full interactive ID-stack browser,
- label-to-`test_id` inference,
- table column identity.
