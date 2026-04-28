# ImUi ID Stack Diagnostics v1 - Milestones

Status: active
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

## M3 - Follow-on Decision

Exit criteria:

- final gates are recorded,
- `WORKSTREAM.json` reflects current lane state,
- and deferred work is split into narrower lanes instead of widening this one by default.

Current deferred candidates:

- full interactive ID-stack browser,
- `fretboard diag query` support for identity warnings,
- IMUI `for_each_keyed` duplicate-key authoring proof,
- label-to-`test_id` inference,
- table column identity.
