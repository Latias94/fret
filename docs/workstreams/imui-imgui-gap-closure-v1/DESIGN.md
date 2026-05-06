# ImUi Dear ImGui Gap Closure v1

Status: Active execution lane
Last updated: 2026-05-06

## Purpose

This lane exists because the previous Dear ImGui parity notes are useful but no longer reliable as
the only current answer. Since April, the repo added many narrow IMUI text, color edit, debug-draw,
identity, and collection follow-ons. Before the next fearless cleanup pass, the project needs one
fresh source-backed gap ledger that says:

- what current Fret IMUI already provides,
- what remains materially below Dear ImGui-class editor usability,
- which stale conclusions should stop driving work,
- and which next slices should delete/refactor code versus widen public surface.

The target is not API cloning. The target is a user-usable IMUI layer that can author real editor
panels while preserving Fret's architecture:

- `fret-imui`: thin authoring facade over `fret-authoring` and `fret-ui`.
- `fret-ui-kit::imui`: policy/helper layer for immediate-style controls and interaction behavior.
- `fret-ui-editor::imui`: thin adapters over declarative editor controls.
- runner/docking/workspace/devtools crates: owners for multi-window, shell, and diagnostics feel.

## Non-goals

- Do not introduce a second immediate runtime beside `crates/fret-ui`.
- Do not widen `crates/fret-ui` just to mirror Dear ImGui helper names.
- Do not reopen closed narrow follow-ons unless fresh source evidence proves their closeout is now
  wrong.
- Do not treat every missing `ImGuiWindowFlags_*`, `ImGuiChildFlags_*`, or raw `ImDrawList` knob as
  a required Fret public API.

## P0 Source-Audit Assumptions

- Confident: `fret-imui` remains policy-light and compiles to Fret elements. If this changes, this
  lane must be reclassified as a runtime contract lane.
- Confident: the largest practical Dear ImGui-class gaps are product/shell, backend multi-window
  hand-feel, diagnostics discoverability, and porting ergonomics rather than a missing runtime
  substrate.
- Likely: public helper widening should continue to require at least two first-party proof surfaces.
- Likely: several old parity notes are partially stale because the 2026-05 debug-draw and color-edit
  follow-ons closed many gaps they still describe as open.

## Priority Model

Use this order when choosing the next slice:

1. **P0 facts and stale-doc cleanup**: update the gap ledger from current source and repo-ref/imgui.
2. **P1 fearless cleanup/deletion**: remove misleading public/proof surfaces, duplicate adapters,
   or obsolete teaching paths once the owner and gate are clear.
3. **P2 user-usable golden path**: make `fret::imui` teach and run a coherent editor panel path.
4. **P3 gap-specific parity slices**: debug-draw depth, porting sugar, collection/helper readiness,
   child-region depth, diagnostics/metrics, docking/multi-window.
5. **P4 product closure**: one workbench-grade proof surface that feels like one system.

## First Slice

The first landable slice is this P0 baseline:

- create the lane,
- add a current source audit,
- mark the old standalone parity audit as partially superseded,
- wire the lane into the workstream index and repo trackers,
- and run the documentation/source-policy gates listed in `EVIDENCE_AND_GATES.md`.

No runtime code changes are admitted in this first slice.
