# Adaptive Layout Contract Closure v1 — Target Interface State

Status: target state for M1 adaptive taxonomy freeze
Last updated: 2026-04-10

Companion docs:

- `DESIGN.md`
- `BASELINE_AUDIT_2026-04-10.md`
- `TODO.md`
- `MILESTONES.md`
- `EVIDENCE_AND_GATES.md`
- `M1_CONTRACT_FREEZE_2026-04-10.md`
- `../../adr/0325-adaptive-authoring-surface-and-query-axis-taxonomy-v1.md`

This document records the intended end state for the adaptive authoring surface.

It answers four concrete questions:

1. which adaptive nouns ordinary app authors should learn,
2. where high-level adaptive policy should live,
3. how recipe/component APIs should name adaptive behavior,
4. which current names should disappear or be narrowed.

## 1. Public Surface Tiers

| Tier | Intended audience | Canonical import lane | What it owns |
| --- | --- | --- | --- |
| Low-level adaptive reads | advanced app code, reusable components, diagnostics-heavy examples | `fret::env::{...}` | container queries, viewport/device queries, safe area, occlusion, pointer capability |
| Adaptive policy / shared vocabulary | app authors who want one higher-level adaptive story | target: `fret::adaptive::{...}` backed by `fret-ui-kit` | typed device/panel classes, shared shell policy helpers, explicit strategy types |
| Recipe strategy | design-system / component crates | `fret-ui-shadcn`, `fret-ui-editor`, future ecosystem crates | source-aligned strategy wrappers, per-family enums, component defaults |
| Advanced / explicit seams | power users and framework authors | explicit raw imports | region ids, raw low-level query helpers, custom strategy composition |

## 2. Target Authoring Rule

### 2.1 `fret::env` stays explicit and low-level

`fret::env` remains the explicit import lane for low-level adaptive reads.

It should continue to own:

- `container_*`
- `viewport_*`
- `safe_area_*`
- `occlusion_*`
- `primary_pointer_*`

It should **not** move into the default prelude.

It should **not** become the only public answer for ordinary app authors forever; it is the
explicit low-level lane, not the final high-level teaching story.

### 2.2 Higher-level adaptive policy belongs above `fret::env`

The target shared policy lane belongs in `fret-ui-kit`, with an app-facing facade re-export from
`fret`.

Target kinds of shared nouns:

- `DeviceAdaptiveClass`
- `PanelAdaptiveClass`
- `AdaptiveQuerySource`
- `DeviceShellPolicy`
- `PanelAdaptivePolicy`

The important part is not the exact final type names but the ownership rule:

- raw query reads stay in `fret::env`,
- shared authoring vocabulary lives above that in ecosystem policy,
- and recipe crates consume that vocabulary instead of re-deriving it ad hoc.

### 2.3 `imui` and declarative surfaces share the vocabulary, not the widget family

The immediate-mode/editor path does not need to consume the same app-shell widgets as declarative
recipe code.

What should be shared:

- the adaptive axis vocabulary,
- query-source naming,
- shell-vs-panel boundary rules,
- diagnostics expectations.

What should **not** be forced:

- one common widget family for app shell and editor shell,
- one common builder shape for declarative and `imui`.

## 3. Naming Rules for New Public APIs

### 3.1 Query-source choice must be explicit

When a public API selects between viewport-driven and container-driven behavior, prefer:

- `*Query`
- `*ResponsiveQuery`
- `*BreakpointQuery`

with variants such as:

- `Viewport`
- `Container`

Current good examples:

- `DataTableToolbarResponsiveQuery`
- `NavigationMenuMdBreakpointQuery`

### 3.2 Device-shell behavior must say so

When the behavior is genuinely about mobile/desktop shell choice or device capabilities, prefer
names such as:

- `device_*`
- `viewport_*`
- `*_shell_*`
- `mobile_*`
- `desktop_*`

Bare `responsive(bool)` is not the target interface.

### 3.3 Panel/container behavior must say so

When the behavior follows local panel/container width, prefer names such as:

- `container_*`
- `panel_*`
- `*ContainerAdaptive*`
- `*PanelAdaptive*`

Bare `Responsive` as an enum variant is not the target interface unless the query axis is explicit
at the same type boundary.

### 3.4 Width ownership stays a layout story

If the real issue is width/height negotiation on the local page shell or preview shell, keep that
story on ordinary layout/sizing APIs.

Do not relabel caller-owned sizing as adaptive policy just because the example also happens to be
narrow-window sensitive.

## 4. Component Family Classification

### 4.1 App-shell / device-shell family

These surfaces are allowed to stay viewport/device-oriented:

- `Drawer`
- `Sheet`
- desktop/mobile dialog wrappers
- app-shell `Sidebar`

Target rule:

- they may branch on device shell,
- but they should say so explicitly in their public naming and docs.

### 4.2 Panel/container family

These surfaces are expected to be container-first when panel width is the real driver:

- `Field` responsive label/content orientation
- `Carousel` panel-width item/layout variation
- `NavigationMenu` inside resizable panels
- data-table toolbar adaptations in editor-like shells
- future editor rails / inspector sidebars

Target rule:

- if panel width is the real driver, viewport breakpoints are only fallback/legacy debt, not the
  preferred interface state.

### 4.3 Strategy wrappers

Recipe-level strategy wrappers are acceptable when they reduce repeated desktop/mobile branching,
but they should remain explicit about:

- which axis they consume,
- which widths remain caller-owned,
- and which layer owns the policy.

## 5. Current Classified Surfaces

Recently aligned toward the target state:

| Current surface | Current meaning | Current classification |
| --- | --- | --- |
| `Combobox::device_shell_responsive(bool)` / `device_shell_md_breakpoint(Px)` | device-shell drawer-vs-popover branch | explicit device-shell vocabulary; keep viewport-driven behavior on this lane |
| `FieldOrientation::ContainerAdaptive` | container-query-driven label/content layout | explicit container/panel vocabulary; keep the upstream `responsive` label only in docs/example parity copy |

Remaining migration pressure:

| Current surface | Current meaning | Target direction |
| --- | --- | --- |
| `SidebarProvider::is_mobile(...)` / `is_mobile_breakpoint(...)` | app-shell mobile inference | keep as app-shell wording or wrap in an explicit shell-mode surface; do not generalize into the editor panel story |

Surfaces that are already closer to the target state and should be treated as exemplars:

- `DataTableToolbarResponsiveQuery`
- `NavigationMenuMdBreakpointQuery`
- carousel viewport-vs-container breakpoint split
- `ResponsiveGrid` / `ResponsiveStack` only when their query source remains explicit and docs keep
  the default container-first story visible

## 6. `children(...)` Boundary

Adaptive participation does **not** automatically justify broader generic `children(...)` APIs.

Target rule:

- widen `children(...)` only when source-aligned authoring evidence shows that the current
  component-specific seam is insufficient,
- do not widen it merely because a component is adaptive,
- keep recipe-specific builders as the default when they already express the intended adaptive
  authoring path clearly.

## 7. Rejected Interface State

The target state explicitly rejects:

- a new generic `responsive: bool` pattern across public component APIs,
- a runtime-wide "responsive manager" in `crates/fret-ui`,
- viewport-width breakpoints as the default answer for editor/panel adaptation,
- moving adaptive helpers into the default app prelude,
- and broad generic `children(...)` growth justified only by adaptive participation.

## 8. Minimum Proof Before Large Renames

Before widening rename work beyond one family, keep these proofs green:

- one UI Gallery narrow-window/device-shell proof,
- one fixed-window panel-resize/container proof,
- and one docs/Gallery teaching surface that explicitly compares viewport-driven and
  container-driven behavior.

This keeps the target interface tied to user-visible truth rather than to naming alone.
