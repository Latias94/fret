# ImUi Child Region Depth v1 - TODO

Status: closed closeout record
Last updated: 2026-04-22

Status note (2026-04-22): this lane closes after M2 landed the bounded
`ChildRegionChrome::{Framed, Bare}` slice and the closeout audit confirmed that the remaining
`BeginChild()`-scale pressure belongs to other owners or future narrower lanes. Do not continue
additive child-region helper growth in this folder by default.

## Lane setup

- [x] Create the lane as a narrow follow-on under the immediate-mode product-closure umbrella.
- [x] Wire the lane into `docs/workstreams/README.md`, `docs/roadmap.md`, `docs/todo-tracker.md`,
      and the umbrella status docs.
- [x] Freeze that this lane follows the closed collection/pane proof record rather than reopening
      that folder.
- [x] Freeze one current repro/gate/evidence package instead of leaving the lane open-ended.

## M0 - Baseline and owner freeze

- [x] Write one baseline audit that re-reads:
      - the collection/pane closeout,
      - the umbrella parity status,
      - the current parity audit,
      - the current `child_region` helper/options,
      - and the local `repo-ref/imgui` child-window references.
      Result: `M0_BASELINE_AUDIT_2026-04-22.md`.
- [x] Freeze the default owner split for this lane.
      Result: `DESIGN.md` now keeps `fret-ui-kit::imui` as the additive helper owner,
      `fret-imui` as the focused proof owner, `apps/fret-examples` as the pane-proof/source-policy
      owner, and `fret-workspace` as the shell/workbench owner without reopening runtime growth.
- [x] Name the smallest current repro/gate/evidence package.
      Result: `EVIDENCE_AND_GATES.md` now freezes the current pane-proof demos, the focused
      composition proof, and the lane-local source-policy gate.

## M1 - Target surface and defer list

- [x] Freeze which `BeginChild()`-scale concerns are credible candidates for generic admission:
      - frame/padding policy,
      - axis-specific resize,
      - axis-specific auto-resize / always-auto-resize,
      - focus/navigation boundary posture,
      - and visibility/clipping posture.
      Result: `M1_TARGET_SURFACE_FREEZE_2026-04-22.md` now keeps frame/padding posture as the
      first credible generic candidate while rejecting a `size_arg` clone and deferring resize,
      auto-resize, focus-boundary flattening, and begin-return posture.
- [x] Decide which of those concerns must stay out of generic `child_region` for now.
      Result: `M1_TARGET_SURFACE_FREEZE_2026-04-22.md` now keeps axis-specific resize shell-owned,
      keeps auto-resize and focus-boundary posture deferred, and rejects a `BeginChild() -> bool`
      clone for the declarative helper surface.
- [x] Freeze the first defer list for this lane:
      - collection marquee / lasso breadth,
      - key-owner / collection keyboard-owner work,
      - menu/tab policy,
      - shell-helper promotion,
      - and runner/backend multi-window parity.
      Result: `M1_TARGET_SURFACE_FREEZE_2026-04-22.md` now freezes that defer list and adds
      shell-owned pane resize plus imperative begin/end return cloning to the explicit reject set.

## M2 - First bounded slice or no-new-surface verdict

- [x] If the evidence is strong enough, land only one bounded `ChildRegionOptions` slice with
      focused proof.
      Result: `M2_CHILD_REGION_CHROME_SLICE_2026-04-22.md` now lands
      `ChildRegionChrome::{Framed, Bare}` as the only admitted generic child-depth slice, while
      `fret-ui-kit` seam smoke plus the focused `fret-imui` composition floor keep the contract
      executable.
- [x] If the evidence is still thin, close M2 on a no-new-generic-surface verdict instead of
      adding a wide `ChildRegionFlags` clone.
      Result: `M2_CHILD_REGION_CHROME_SLICE_2026-04-22.md` now explicitly rejects widening the
      lane into a broad child-flag bag beyond the landed chrome slice.
- [x] Add or refine focused gates only for the admitted slice or verdict.
      Result: `EVIDENCE_AND_GATES.md`, `WORKSTREAM.json`, and the lane-local source-policy test
      now lock the chrome slice, the closeout state, and the focused `fret-ui-kit` / `fret-imui`
      proof package against drift.

## M3 - Closeout or split again

- [x] Close the lane if the target surface and first verdict become explicit enough.
      Result: `CLOSEOUT_AUDIT_2026-04-22.md` now closes the lane and reclassifies this folder as a
      closeout record for the landed chrome slice.
- [x] Start a different narrow follow-on instead of widening this lane if the remaining pressure
      becomes mostly:
      - collection keyboard-owner depth,
      - shell/product pane behavior,
      - or runner/backend view/window behavior.
      Result: `CLOSEOUT_AUDIT_2026-04-22.md` now keeps resize, auto-resize, focus-boundary
      flattening, broader pane behavior, and any future runner/backend pressure out of this closed
      folder unless stronger first-party proof starts a different narrow lane.
