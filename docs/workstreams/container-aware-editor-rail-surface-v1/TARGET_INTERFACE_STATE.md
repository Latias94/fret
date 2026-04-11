# Container-Aware Editor Rail Surface v1 — Target Interface State

Status: active target-state draft
Last updated: 2026-04-11

Companion docs:

- `DESIGN.md`
- `M0_BASELINE_AUDIT_2026-04-11.md`
- `TODO.md`
- `MILESTONES.md`
- `EVIDENCE_AND_GATES.md`
- `docs/workstreams/adaptive-layout-contract-closure-v1/TARGET_INTERFACE_STATE.md`
- `docs/adr/0325-adaptive-authoring-surface-and-query-axis-taxonomy-v1.md`

This document records the intended end state for a future reusable editor rail surface without
assuming that the public surface should be extracted immediately.

It answers five concrete questions:

1. which layer owns the outer shell seam,
2. which layer owns reusable inner editor content,
3. which adaptive axis the rail should follow,
4. how mobile/device-shell behavior composes with the rail,
5. and what proof must exist before a reusable public surface is promoted.

## 1. Public Surface Tiers

| Tier | Intended audience | Canonical owner | What it owns |
| --- | --- | --- | --- |
| Shell placement seam | workspace/app shell authors | `fret-workspace::WorkspaceFrame` | left/right/top/bottom shell slots, shell width ownership, outer layout |
| Shared adaptive vocabulary | app authors and reusable helpers | `fret::adaptive` backed by `fret-ui-kit` | `PanelAdaptiveClass`, panel/device policy nouns, explicit query-source naming |
| Reusable editor content | editor-oriented ecosystem crates and apps | `fret-ui-editor` | inspector panels, property groups/grids, editor density and content policy |
| Concrete rail recipe/state | app-local or demo-local until promoted | app/demo layer first | concrete rail header/body/footer layout, collapse affordances, local commands/state |
| Future extracted rail surface | only after proof threshold is met | likely `fret-ui-editor` or a workspace-owned follow-on | reusable container-aware rail recipe if and only if repeated evidence exists |

## 2. Target Authoring Rule

### 2.1 Shell placement stays on `WorkspaceFrame.left/right`

The outer shell seam is already shipped.

Target rule:

- do not invent a second shell-placement abstraction just to host an editor rail,
- and do not rename `WorkspaceFrame` slots into an inspector/sidebar recipe surface.

### 2.2 Reusable inner content stays editor-owned

Current reusable editor composites already live in `fret-ui-editor`.

Target rule:

- keep inspector/property-grid content on `fret-ui-editor`,
- and do not stretch shadcn app-shell widgets into editor content ownership.

### 2.3 Editor rails are container-first

Editor rails usually adapt because:

- the dock panel width changes,
- the workspace shell is resized,
- or the host pane is collapsed/expanded.

Target rule:

- use container/panel adaptive vocabulary for reusable editor rails,
- prefer `PanelAdaptiveClass` or explicit panel-query nouns above raw reads,
- and treat viewport/device-shell queries as outer-shell inputs only.

### 2.4 Mobile/device-shell behavior is an outer-shell concern

If an editor-oriented app also targets mobile or compact devices, the outer shell may choose to:

- hide the rail,
- remount it as a sheet/drawer,
- or replace it with a route/stack detail page.

Target rule:

- do not bake mobile-sheet semantics into a generic reusable editor rail component,
- keep that device-shell decision at the app shell or workspace shell boundary,
- and let the inner rail recipe remain container-aware once mounted.

### 2.5 Docking remains a host, not the rail recipe owner

Docking may host the rail inside a docked panel.

Target rule:

- `fret-docking` keeps topology, registry, and host mechanics,
- but concrete rail chrome/policy should remain above docking.

## 3. Promotion Threshold

A reusable public `PanelRail` / `InspectorSidebar` candidate should not be promoted until all of
these are true:

1. there are at least two real consumers with materially similar rail structure,
2. the consumers already mount through `WorkspaceFrame.left/right` or an equivalent existing shell
   seam,
3. the shared shape is container-aware rather than viewport-first,
4. the mobile/device-shell downgrade path is still owned by outer shell code,
5. and one focused panel-resize proof plus one source-level owner-split gate remain green.

## 4. Rejected Interface State

This lane explicitly rejects:

- widening `SidebarProvider::is_mobile(...)` into the generic editor-rail API,
- introducing a public `PanelRail` / `InspectorSidebar` before the second-consumer threshold,
- inventing a new shell slot abstraction instead of using `WorkspaceFrame.left/right`,
- treating viewport width as the primary adaptive driver for reusable editor rails,
- and moving rail recipe policy into `fret-docking`.
