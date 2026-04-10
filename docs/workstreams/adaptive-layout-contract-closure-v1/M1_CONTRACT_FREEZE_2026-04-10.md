# M1 Contract Freeze — 2026-04-10

Status: accepted decision

Related:

- `DESIGN.md`
- `TARGET_INTERFACE_STATE.md`
- `BASELINE_AUDIT_2026-04-10.md`
- `TODO.md`
- `MILESTONES.md`
- `EVIDENCE_AND_GATES.md`
- `../../adr/0325-adaptive-authoring-surface-and-query-axis-taxonomy-v1.md`
- `../../adr/IMPLEMENTATION_ALIGNMENT.md`

## Decision

Freeze the adaptive authoring story like this:

### 1) Four adaptive axes stay explicit

The public story must keep these separate:

- container/panel adaptation,
- viewport/device/capability adaptation,
- caller-owned shell sizing,
- and strategy-layer adaptive recipes.

This lane does not reopen ADR 0231 or ADR 0232; it freezes how app authors and recipe authors are
supposed to use them together.

### 2) `fret::env` stays the explicit low-level lane

Low-level query helpers remain on `fret::env::{...}` and stay out of the default prelude.

If Fret needs a more approachable adaptive lane for ordinary app code, it should be an explicit
ecosystem-backed facade lane above `fret::env`, not a runtime widening and not a default-prelude
expansion.

### 3) New public APIs must name the adaptive axis

New public APIs should avoid bare `responsive` booleans and encode the axis explicitly:

- query-source choice uses `*Query` / `*ResponsiveQuery`,
- device-shell behavior uses `device_*`, `viewport_*`, `*_shell_*`, or explicit mobile/desktop
  wording,
- panel-width behavior uses `container_*`, `panel_*`, or container/panel-adaptive naming.

### 4) App-shell sidebars and editor-panel rails stay separate

Current `Sidebar` remains an app-shell surface.

If editor-grade panel/container semantics need a dedicated sidebar/rail later, that should be a
separate container-aware surface rather than widening the current app-shell sidebar until its
meaning blurs.

### 5) Adaptive work does not justify broad `children(...)` growth by itself

Adaptive participation alone is not evidence for widening generic `children(...)` APIs.

Any such widening still requires source-aligned proof that the current component-specific seam is
insufficient.

## What this unblocks next

After this freeze, the lane can move to:

1. proof-surface promotion for panel-resize vs narrow-window evidence,
2. bounded rename/cleanup slices on ambiguous recipe APIs,
3. clearer Gallery/docs teaching surfaces for axis ownership.
