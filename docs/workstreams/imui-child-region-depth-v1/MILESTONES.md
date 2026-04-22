# ImUi Child Region Depth v1 - Milestones

Status: closed closeout record
Last updated: 2026-04-22

Status note (2026-04-22): this file now records the closed child-region depth verdict only. Active
implementation should move to a different narrow lane if fresh first-party evidence exceeds this
closeout.

## M0 - Baseline and owner freeze

Exit criteria:

- the repo explicitly states why child-region depth is a new narrow follow-on instead of a reopened
  proof-breadth lane,
- the owner split is explicit enough to avoid runtime drift,
- and the lane names one current repro/gate/evidence package.

Primary evidence:

- `DESIGN.md`
- `M0_BASELINE_AUDIT_2026-04-22.md`
- `docs/workstreams/imui-collection-pane-proof-v1/CLOSEOUT_AUDIT_2026-04-21.md`
- `docs/workstreams/standalone/imui-imgui-parity-audit-v2.md`

Current status:

- Closed on 2026-04-22 via `M0_BASELINE_AUDIT_2026-04-22.md`.

## M1 - Target surface freeze

Exit criteria:

- the lane names which `BeginChild()`-scale concerns are real generic candidates,
- the lane explicitly records what remains deferred,
- and the first candidate slice is small enough to review independently.

Primary evidence:

- `DESIGN.md`
- `M1_TARGET_SURFACE_FREEZE_2026-04-22.md`
- `TODO.md`
- `EVIDENCE_AND_GATES.md`
- `ecosystem/fret-ui-kit/src/imui/child_region.rs`
- `ecosystem/fret-ui-kit/src/imui/options/containers.rs`
- `repo-ref/imgui/imgui.h`

Current status:

- Closed on 2026-04-22 via `M1_TARGET_SURFACE_FREEZE_2026-04-22.md`.
- M1 now freezes one explicit target surface:
  frame/padding posture is the first credible generic candidate.
- M1 explicitly keeps generic sizing on `LayoutRefinement` and defers axis resize, auto-resize,
  focus-boundary flattening, and begin-return posture.
- The expected next step is still a bounded M2 decision, not a wide flag clone.

## M2 - First bounded slice or no-new-surface verdict

Exit criteria:

- the lane either lands one bounded child-region surface addition or closes on a no-new-surface
  verdict,
- any helper growth is directly tied to first-party proof,
- and the resulting gate package stays reviewable.

Primary evidence:

- `M2_CHILD_REGION_CHROME_SLICE_2026-04-22.md`
- `WORKSTREAM.json`
- `TODO.md`
- `EVIDENCE_AND_GATES.md`
- `ecosystem/fret-imui/src/tests/composition.rs`
- `ecosystem/fret-ui-kit/src/imui/child_region.rs`

Current status:

- Closed on 2026-04-22 via `M2_CHILD_REGION_CHROME_SLICE_2026-04-22.md`.
- The lane now ships one bounded generic child-depth addition:
  `ChildRegionChrome::{Framed, Bare}`.
- Default `ChildRegionChrome::Framed` keeps the existing card-like posture, while
  `ChildRegionChrome::Bare` removes the built-in frame/padding chrome without changing the scroll
  substrate or the retained/declarative contract shape.
- M2 explicitly keeps sizing on `LayoutRefinement` and rejects broad child-flag growth beyond the
  landed chrome slice.

## M3 - Closeout or split again

Exit criteria:

- the lane either closes with explicit owner split and first verdict,
- or splits again because the remaining pressure clearly belongs to a different owner/problem.

Primary evidence:

- `WORKSTREAM.json`
- `TODO.md`
- `EVIDENCE_AND_GATES.md`
- `CLOSEOUT_AUDIT_2026-04-22.md`
- future follow-on lane docs when stronger first-party proof exceeds this closeout

Current status:

- Closed on 2026-04-22 via `CLOSEOUT_AUDIT_2026-04-22.md`.
- This folder is now a closeout record for the landed chrome slice and the no-further-generic
  helper-growth verdict for this cycle.
- Future child-depth pressure should start a different narrow lane with stronger first-party proof
  instead of reopening this folder by default.
