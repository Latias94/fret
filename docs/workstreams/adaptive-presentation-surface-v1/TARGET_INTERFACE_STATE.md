# Adaptive Presentation Surface v1 — Target Interface State

Status: Closed-lane target-state reference
Last updated: 2026-04-11

Companion docs:

- `DESIGN.md`
- `M0_BASELINE_AUDIT_2026-04-11.md`
- `M1_CONTRACT_FREEZE_2026-04-11.md`
- `CLOSEOUT_AUDIT_2026-04-11.md`
- `TODO.md`
- `MILESTONES.md`
- `EVIDENCE_AND_GATES.md`
- `../adaptive-layout-contract-closure-v1/TARGET_INTERFACE_STATE.md`
- `../../adr/0325-adaptive-authoring-surface-and-query-axis-taxonomy-v1.md`

This document records the intended end state for the upper adaptive presentation interface.

Status note (2026-04-11): this target-state document now reads as the shipped reference for the
closed lane. The closeout verdict is "no new generic adaptive-presentation helper yet"; future
extraction must start as a narrower family-specific follow-on.

It answers five concrete questions:

1. which layer should own same-feature different-presentation decisions,
2. when explicit branching is still the correct surface,
3. when family-specific wrappers are justified,
4. how editor/container work composes with app-shell device branching,
5. and what proof is required before a new shared helper is promoted.

## 1. Public Surface Tiers

| Tier | Intended audience | Canonical owner | What it owns |
| --- | --- | --- | --- |
| Low-level facts | advanced app/component code | `fret::env::{...}` | viewport/device/container facts, pointer/safe-area/occlusion, raw query reads |
| Shared adaptive classification | app authors and reusable helpers | `fret::adaptive::{...}` backed by `fret-ui-kit` | `DeviceAdaptiveClass`, `PanelAdaptiveClass`, `DeviceShellSwitchPolicy`, `device_shell_mode(...)`, `device_shell_switch(...)` |
| App-shell presentation composition | app authors and gallery/docs surfaces | app-local code first | choosing one explicit presentation over another when the app still owns UX semantics |
| Family-specific wrapper or recipe policy | recipe/component crates | `fret-ui-shadcn`, `fret-ui-editor`, other ecosystem crates | source-aligned wrapper APIs only when repetition is real inside one family |
| Editor-shell downgrade | workspace/app shells | shell code above `WorkspaceFrame` / editor content | mobile/compact downgrade of a rail or inspector surface while keeping the mounted inner surface container-aware |

## 2. Target Authoring Rule

### 2.1 Ask the driver question first

Before choosing any adaptive presentation API, answer:

1. is the driver panel/container size,
2. device shell / viewport capability,
3. or ordinary caller-owned sizing?

If the answer is panel/container width, the decision must not be encoded as a viewport-first
device-shell helper.

If the answer is device shell, the decision must not be relabeled as a generic editor/container
surface.

If the answer is only ordinary width ownership, keep it on layout APIs instead of adaptive
presentation APIs.

### 2.2 The outermost layer that still knows semantics owns the presentation choice

The correct owner is the highest layer that can still name the user-visible semantic intent.

Examples:

- `Dialog` vs `Drawer` profile editing remains app/gallery composition today because the caller
  still owns whether those branches should share copy, width, focus flow, and footer actions.
- `SidebarProvider::is_mobile(...)` remains recipe/provider-owned because the sidebar family
  already owns the app-shell desktop-vs-sheet semantics.
- editor-rail mobile downgrade remains outer-shell-owned because the shell still decides whether
  the rail becomes a drawer, route, sheet, or disappears entirely.

### 2.3 A new shared helper needs stronger proof than repeated explicit branches

A new helper or wrapper is justified only if all of these are true:

1. at least two real consumers share the same adaptive decision shape,
2. the shared part is more specific than generic `device_shell_switch(...)`,
3. the consumers agree on owner layer and adaptive axis,
4. lifecycle policy is materially aligned:
   - focus/dismiss semantics,
   - state ownership,
   - sizing ownership,
   - and accessibility surface,
5. one source gate and one behavior proof keep that shared shape reviewable.

Without that proof, explicit branching is preferred.

### 2.4 Family-specific wrappers stay family-specific

If a helper is extracted, it should default to the narrowest valid owner:

- app-local shell helper before a cross-family helper,
- family-specific recipe wrapper before a repo-wide presentation manager,
- editor-specific helper before widening app-shell recipe surfaces.

The current exemplar remains `Combobox`, not a generic repo-wide overlay-presentation wrapper.

## 3. Current Classified Surfaces

| Surface | Current owner | Current interface state | Upper-interface verdict |
| --- | --- | --- | --- |
| `Dialog` + `Drawer` responsive pairing | app/gallery composition | explicit paired branches in `responsive_dialog.rs` | current v1 verdict: keep explicit; the family does not yet meet wrapper-extraction threshold |
| `Date Picker` / `Breadcrumb` device-shell branching | app/gallery composition using shared helper | explicit `device_shell_switch(...)` call site | keep helper visible at the call site |
| `Combobox` responsive presentation | recipe family | recipe-owned wrapper above shared helper | keep as current wrapper exemplar, not a generic precedent |
| `SidebarProvider::is_mobile(...)` / `is_mobile_breakpoint(...)` | sidebar app-shell recipe/provider | provider-owned mobile sheet inference | keep app-shell-only; do not widen into panel/editor adaptation |
| editor rail / inspector mobile downgrade | outer shell above `WorkspaceFrame` and `fret-ui-editor` | explicit shell choice outside the mounted rail | keep outer-shell-owned; do not collapse into `Sidebar` or a generic adaptive wrapper |

## 4. Rejected Interface State

This lane rejects:

- a new repo-wide `AdaptivePresentation` manager in `crates/fret-ui`,
- a new generic `responsive(bool)` pattern for presentation selection,
- widening `Sidebar` into the editor/panel adaptive story,
- turning the explicit drawer responsive dialog proof into an implicit generic wrapper before the
  proof threshold exists,
- and using `device_shell_switch(...)` as a reason to hide meaningful family-specific policy.

## 5. Promotion / Exit Criteria

This lane can close only after one of these outcomes is explicit:

1. **No-new-helper verdict**
   - the repo records that current explicit surfaces remain the correct v1 answer,
   - and future extraction must start as a narrower family-specific follow-on.

2. **Narrow helper promotion**
   - one specific family or shell layer has met the extraction threshold,
   - and a new follow-on owns the concrete API, gates, and migration.

This lane should not close by quietly drifting into broad implementation work.
