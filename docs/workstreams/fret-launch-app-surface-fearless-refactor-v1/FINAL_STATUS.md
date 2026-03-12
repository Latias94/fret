# Fret Launch + App Surface (Fearless Refactor v1) Final Status

Date: 2026-03-06
Status: examples-side launch posture migration complete

## Summary

This workstream started from a surface-audit concern: the advanced launch story was technically
extensible, but app-facing posture still taught too much compatibility surface (`WinitAppDriver`) in
examples and helper naming.

The final conclusion is unchanged from the earlier audits, but it is now supported by stronger
implementation evidence:

- `FnDriver` is sufficient as the preferred advanced example posture.
- The main problem was posture / naming / facade defaults, not missing launch hooks.
- The examples-side direct `WinitAppDriver` inventory is now zero.
- `fret::advanced::run_native_with_configured_fn_driver(...)` closed the last app-facing helper gap for
  preconfigured drivers.

## Landed outcomes

### 1) App-facing launch posture is tighter

- `fret` keeps the app-author/defaults story.
- `fret-framework` remains the curated manual-assembly facade.
- `fret-launch` remains the advanced integration crate.
- `FnDriver` is now the dominant advanced example posture instead of direct `WinitAppDriver` impls.

### 2) Example inventory reached zero

The following advanced / medium / heavy examples were migrated during this workstream tail and now
serve as evidence that the existing hook matrix is sufficient:

- docking-heavy: `docking_demo`, `container_queries_docking_demo`, `docking_arbitration_demo`
- node-graph: `node_graph_legacy_demo`, `node_graph_domain_demo`
- 3D / engine-frame: `gizmo3d_demo`
- gallery / accessibility-heavy: `components_gallery`
- plus the earlier low-risk, single-window, medium-complexity, stress, and custom-effect batches
  tracked in `MILESTONES.md` and `SURFACE_AUDIT.md`

Current examples-side direct `WinitAppDriver` inventory:

- `0`

### 3) Guardrails are in place

The workstream is now backed by focused gates that make posture regressions reviewable:

- `python tools/gate_winit_driver_example_hook_coverage.py`
- `python tools/gate_fn_driver_example_naming.py`
- `python tools/gate_fret_builder_only_surface.py`
- `python tools/gate_fret_launch_surface_contract.py`
- `python tools/gate_fret_framework_launch_surface.py`
- `python tools/check_layering.py`

## What this workstream did not prove

This workstream materially de-risked the launch posture story, but it did not claim that every
future advanced embedding need is already solved forever.

The remaining open questions are still the same architectural questions captured by the audit docs:

- whether native-side runner-handle parity is ever needed
- how far helper-layer config curation should go before introducing new public wrappers
- whether any future advanced integration should ever justify reintroducing direct compat posture in
  examples (it should now require explicit review)

## Recommended next steps

### Immediate follow-up

- Merge or rebase the current branch onto `main` while the touched surface is still fresh.
- Keep this workstream open until conflict resolution is complete and the branch is green again.

### Short-term maintenance

- Treat any future example-local direct `WinitAppDriver` impl as a regression unless explicitly
  justified.
- Keep adding advanced examples through `build_fn_driver()` / configured-`FnDriver` posture.
- Prefer app-facing helper improvements over expanding the lower-level launch contract surface.

### Optional doc cleanup

- If this workstream is considered functionally complete after branch sync, mark it as closed in a
  future doc pass and link the final merged PR / commit range here.

## Evidence anchors

- `docs/workstreams/fret-launch-app-surface-fearless-refactor-v1/MILESTONES.md`
- `docs/workstreams/fret-launch-app-surface-fearless-refactor-v1/SURFACE_AUDIT.md`
- `docs/workstreams/fret-launch-app-surface-fearless-refactor-v1/TODO.md`
- `docs/workstreams/crate-audits/fret-launch.l1.md`
- `ecosystem/fret/src/lib.rs`
