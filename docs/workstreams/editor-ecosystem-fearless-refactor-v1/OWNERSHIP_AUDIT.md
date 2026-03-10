# Editor Ecosystem Fearless Refactor v1 - Ownership Audit

Tracking doc: `docs/workstreams/editor-ecosystem-fearless-refactor-v1/DESIGN.md`

Status: Initial audit
Last updated: 2026-03-09

## Purpose

This note turns the workstream's ownership claims into a concrete audit so that future extraction
work is reviewable and does not quietly blur crate boundaries.

The main question is not "can this code move?".
The main question is "which layer should own it if it moves?".

## Extraction rubric

Promote an app-layer surface into `ecosystem/*` only when all of the following are true:

1. The surface is no longer tied to one app's document/project model.
2. The owning crate can be named by responsibility rather than by one product screen.
3. There is a second in-tree consumer, or a clear near-term reuse path that is already planned.
4. The surface can move without forcing reverse dependencies into design-system or app crates.

Use this routing rule when deciding the landing site:

- reusable editor widgets and composites -> `ecosystem/fret-ui-editor`
- editor shell chrome and non-dock-aware tab/pane frame composition -> `ecosystem/fret-workspace`
- dock-graph-aware tabs, drops, splits, viewport host integration -> `ecosystem/fret-docking`
- viewport tool input glue and arbitration -> `ecosystem/fret-viewport-tooling` / `ecosystem/fret-ui-kit`
- gizmo/tool rendering and host-driven tool mechanics -> `ecosystem/fret-gizmo`
- app/project/domain protocols with no second consumer yet -> stay in `apps/fret-editor`

## Crate ownership summary

| Surface | Recommended owner | Decision | Notes / evidence |
| --- | --- | --- | --- |
| `ecosystem/fret-imui` | authoring frontend only | Keep | Explicitly documented as a small, policy-light facade that compiles to declarative elements: `ecosystem/fret-imui/src/lib.rs`. |
| `ecosystem/fret-ui-editor` | reusable editor controls and composites | Keep and scale | Already scoped as editor-grade UI primitives and controls; optional `imui` facade must stay thin: `ecosystem/fret-ui-editor/src/lib.rs`, `ecosystem/fret-ui-editor/src/imui.rs`. |
| `ecosystem/fret-workspace` | editor shell chrome and shell state | Keep and scale | Owns shell building blocks, tab state, command scope, frame chrome: `ecosystem/fret-workspace/src/lib.rs`, `ecosystem/fret-workspace/src/tabs.rs`. |
| `ecosystem/fret-docking` | dock-aware interaction policy and chrome | Keep separate | Owns docking UI and interaction policy plus `imui` embedding glue: `ecosystem/fret-docking/src/lib.rs`, `ecosystem/fret-docking/src/imui.rs`. |
| `ecosystem/fret-viewport-tooling` | viewport-tool input glue and protocol | Reuse instead of duplicating | Already exists specifically to avoid pushing viewport tool glue into other crates: `ecosystem/fret-viewport-tooling/src/lib.rs`. |
| `ecosystem/fret-gizmo` | host-driven gizmo and viewport-tool domain logic | Reuse instead of duplicating | Already owns mechanism-level gizmo/tooling surfaces and depends on `fret-viewport-tooling`: `ecosystem/fret-gizmo/src/lib.rs`. |

## `apps/fret-editor` module audit

| Module | Current shape | Recommended target | Decision | Rationale |
| --- | --- | --- | --- | --- |
| `property.rs` | generic path/value vocabulary, but still narrow value enum | future dedicated inspector/property protocol crate | Incubate, then extract | Reusable, but it is not a widget concern. It should not be moved into `fret-ui-editor`. It likely needs a broader value model or host-owned typed payload seam first. Evidence: `apps/fret-editor/src/property.rs`. |
| `inspector_protocol.rs` | property tree + editor registry policy | future dedicated inspector/property protocol crate | Extract candidate | Strong reuse pressure across inspector-style apps, but it is protocol/state, not widget chrome. Keep out of `fret-ui-editor` until a crate boundary like `fret-inspector-protocol` is ready. Evidence: `apps/fret-editor/src/inspector_protocol.rs`. |
| `property_edit.rs` | window-scoped property edit request service | future inspector session crate or keep app-owned | Incubate | This is a session/service layer around the protocol, not a base widget. It should move only with a broader inspector editing story. Evidence: `apps/fret-editor/src/property_edit.rs`. |
| `inspector_edit.rs` | popup edit request + parse helpers | future inspector session crate or keep app-owned | Incubate | Reusable in concept, but currently shaped by popup-edit workflow details and narrow value parsing. Do not move directly into `fret-ui-editor`. Evidence: `apps/fret-editor/src/inspector_edit.rs`. |
| `viewport_tools.rs` | editor viewport interaction state, thresholds, 2D tool policy | split between existing viewport tooling/gizmo crates and app layer | Refactor before extraction | This file overlaps conceptually with `fret-viewport-tooling` and `fret-gizmo`. The generic input/threshold/tool-state parts should converge there first; any remaining editor-product policy can stay app-owned. Evidence: `apps/fret-editor/src/viewport_tools.rs`, `ecosystem/fret-viewport-tooling/src/lib.rs`. |
| `viewport_overlays.rs` | 2D overlay paint helpers for editor demos | stay app-owned for now; future adjacent viewport-editor crate if reuse appears | Stay app for now | The module is explicitly app/editor-level policy and not the same thing as docking or 3D gizmo rendering. Do not push it into `fret-docking`; only extract after a second consumer exists. Evidence: `apps/fret-editor/src/viewport_overlays.rs`. |
| `project.rs` | asset/project tree service and `.meta` handling | app layer | Stay app-owned | This is clearly product/application logic, not reusable editor UI infrastructure. Evidence: `apps/fret-editor/src/project.rs`. |

## First extraction shortlist

## Wave 1 - High-signal protocol cleanup

Target:

- `apps/fret-editor/src/property.rs`
- `apps/fret-editor/src/inspector_protocol.rs`

Recommended destination:

- a future inspector/property protocol crate in `ecosystem/*`

Why first:

- these files define reusable editor data/protocol shapes,
- they are not tied to one visual skin,
- and multiple editor surfaces will need the same tree/path/editor-kind vocabulary.

Prerequisites:

- widen or re-think `PropertyValue` so it does not freeze one narrow value vocabulary too early,
- confirm at least one second in-tree consumer outside `apps/fret-editor`,
- keep the crate separate from `fret-ui-editor` so protocol and widget layers do not collapse.

## Wave 2 - Inspector edit session cleanup

Target:

- `apps/fret-editor/src/property_edit.rs`
- `apps/fret-editor/src/inspector_edit.rs`

Recommended destination:

- a future inspector session/editing crate, only after Wave 1 stabilizes

Why later:

- these services are useful, but they currently encode one popup-edit flow,
- they should follow the protocol split rather than force `fret-ui-editor` to own window-scoped
  edit request services.

## Wave 3 - Viewport tool convergence, not direct extraction

Target:

- `apps/fret-editor/src/viewport_tools.rs`
- `apps/fret-editor/src/viewport_overlays.rs`

Recommended action:

1. first rebase generic tool-input and drag-threshold logic onto `fret-viewport-tooling`,
2. align reusable tool mechanics with `fret-gizmo`,
3. only then evaluate what remains as app-specific editor viewport chrome.

Important non-goal:

- do not move these modules directly into `fret-ui-editor`
- do not move them into `fret-docking`

## Stay app-owned

These should stay in `apps/fret-editor` unless the product scope changes radically:

- `apps/fret-editor/src/project.rs`

Reason:

- asset/project tree management and `.meta` handling are application/product concerns, not reusable
  editor UI library infrastructure.

## Boundary conclusions

1. No current `apps/fret-editor` module should move directly into `fret-ui-editor` in one step.
2. The strongest immediate extraction pressure is around property/inspector protocol, not around
   shell chrome.
3. Viewport-related editor code should converge with the already-existing `fret-viewport-tooling`
   and `fret-gizmo` crates before any new extraction is attempted.
4. `fret-workspace` and `fret-docking` should remain separate:
   shell state and non-dock-aware tabs in workspace, dock-graph-aware behavior in docking.
