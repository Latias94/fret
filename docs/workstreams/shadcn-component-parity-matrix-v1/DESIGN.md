---
title: Shadcn Component Parity Matrix v1
status: active
date: 2026-05-25
scope: shadcn, parity-discovery, diagnostics, automation
---

# Shadcn Component Parity Matrix v1

This lane turns shadcn parity tracking from manual visual review into an automated, evidence-backed
component matrix.

Fret is a self-rendered UI framework. It should not try to prove parity by matching HTML tree
structure or by relying on browser semantics as the implementation truth. Upstream shadcn DOM/CSS
snapshots are useful source references for web-facing outcomes, but the Fret proof must come from
Fret-owned diagnostics:

- layout sidecars for geometry, bounds, clipping, and placement,
- bundle schema2 semantics for roles, labels, states, relations, focusability, and actions,
- diagnostics scripts for open/dismiss/focus/keyboard/hover/selection flows,
- `tables.text_paint` and future paint/scene diagnostics for text, color, chrome, and token output,
- owner/layer queues that say whether a mismatch belongs to recipe, policy, mechanism, gallery
  composition, or diagnostics.

## Automation Axes

The matrix tracks these axes per component:

- `SRC`: upstream source refs are attached.
- `UP-DOM`: upstream DOM/CSS snapshot evidence exists.
- `LAYOUT`: Fret layout/geometry evidence exists.
- `SEM`: Fret bundle semantics evidence exists.
- `TEXT`: Fret text/paint evidence exists.
- `BEHAV`: interaction/behavior diag script exists.
- `RESP`: responsive or non-desktop viewport coverage exists.

These axes are intentionally outcome-based. A component can be `regression_locked` for one docs-demo
slice while still missing pressed state, disabled state, mobile breakpoint, text metric, or keyboard
coverage.

## State-Depth Model

The binary axes are necessary but not sufficient. `SRC + UP-DOM + LAYOUT + SEM + BEHAV` can prove a
single docs-demo slice while hiding a missing disabled, hover, focus-visible, pressed, drag/resize,
open, keyboard, mobile, RTL, text-metric, or paint/token gate.

`tools/parity-discovery/shadcn_component_harness_matrix.py` therefore derives a second
`state_depth` model from the coverage manifest, component packets, validation gates, upstream
contexts, and Fret diagnostics summaries. The model records observable evidence tokens for:

- disabled / aria-disabled / focusable-disabled,
- hover,
- focus-visible / focus ring,
- pressed,
- drag / splitter resize,
- open / expanded,
- keyboard paths,
- mobile / constrained / responsive viewports,
- RTL,
- text metrics / style-aware measurement,
- paint, token, chrome, color, border, radius, and contrast output.

The matrix also carries component-specific `required_state_depth` and `missing_state_depth` lists.
This keeps irrelevant states from becoming false gaps: a Button does not need an `open` state, while
Select and Dropdown Menu do. A missing depth token is an automation planning signal, not proof of a
recipe bug.

## Data Sources

- Inventory and existing human-audit state: `docs/shadcn-declarative-progress.md`
- Current target priorities: `tools/parity-discovery/manifests/shadcn_parity_coverage_v2.json`
- Current suite state: `docs/workstreams/shadcn-parity-discovery-harness-v2/artifacts/shadcn_parity_suite_report_v2.json`
- Extra component packet proof:
  `docs/workstreams/shadcn-component-parity-matrix-v1/artifacts/button_group_agent_packet_p0_v1.json`,
  `docs/workstreams/shadcn-component-parity-matrix-v1/artifacts/drawer_agent_packet_p0_v1.json`,
  `docs/workstreams/shadcn-component-parity-matrix-v1/artifacts/calendar_agent_packet_p0_v1.json`,
  and
  `docs/workstreams/shadcn-component-parity-matrix-v1/artifacts/select_agent_packet_p0_v1.json`,
  `docs/workstreams/shadcn-component-parity-matrix-v1/artifacts/combobox_agent_packet_p0_v1.json`,
  and
  `docs/workstreams/shadcn-component-parity-matrix-v1/artifacts/popover_agent_packet_p0_v1.json`,
  and
  `docs/workstreams/shadcn-component-parity-matrix-v1/artifacts/dropdown_menu_agent_packet_p0_v1.json`,
  and
  `docs/workstreams/shadcn-component-parity-matrix-v1/artifacts/input_agent_packet_p0_v1.json`,
  and
  `docs/workstreams/shadcn-component-parity-matrix-v1/artifacts/data_table_agent_packet_p0_v1.json`,
  and
  `docs/workstreams/shadcn-component-parity-matrix-v1/artifacts/progress_agent_packet_p0_v1.json`,
  and
  `docs/workstreams/shadcn-component-parity-matrix-v1/artifacts/badge_agent_packet_p0_v1.json`,
  and
  `docs/workstreams/shadcn-component-parity-matrix-v1/artifacts/button_agent_packet_p0_v1.json`,
  and
  `docs/workstreams/shadcn-component-parity-matrix-v1/artifacts/accordion_agent_packet_p0_v1.json`,
  and
  `docs/workstreams/shadcn-component-parity-matrix-v1/artifacts/alert_agent_packet_p0_v1.json`,
  and
  `docs/workstreams/shadcn-component-parity-matrix-v1/artifacts/alert_dialog_agent_packet_p0_v1.json`,
  and
  `docs/workstreams/shadcn-component-parity-matrix-v1/artifacts/date_picker_agent_packet_p0_v1.json`,
  and
  `docs/workstreams/shadcn-component-parity-matrix-v1/artifacts/resizable_agent_packet_p0_v1.json`,
  and
  `docs/workstreams/shadcn-component-parity-matrix-v1/artifacts/sidebar_agent_packet_p0_v1.json`
  and
  `docs/workstreams/shadcn-component-parity-matrix-v1/artifacts/aspect_ratio_agent_packet_p0_v1.json`
  and
  `docs/workstreams/shadcn-component-parity-matrix-v1/artifacts/avatar_agent_packet_p0_v1.json`
  and
  `docs/workstreams/shadcn-component-parity-matrix-v1/artifacts/breadcrumb_agent_packet_p0_v1.json`
  and
  `docs/workstreams/shadcn-component-parity-matrix-v1/artifacts/field_agent_packet_p0_v1.json`
  and
  `docs/workstreams/shadcn-component-parity-matrix-v1/artifacts/form_agent_packet_p0_v1.json`
  and
  `docs/workstreams/shadcn-component-parity-matrix-v1/artifacts/input_group_agent_packet_p0_v1.json`
  and
  `docs/workstreams/shadcn-component-parity-matrix-v1/artifacts/pagination_agent_packet_p0_v1.json`

## Output

- Machine-readable matrix:
  `docs/workstreams/shadcn-component-parity-matrix-v1/artifacts/shadcn_component_harness_matrix_v1.json`
- Human-readable matrix:
  `docs/workstreams/shadcn-component-parity-matrix-v1/MATRIX.md`

## Definition of Done

The first slice is done when:

- the matrix can be regenerated from repo-local source docs and parity artifacts,
- every registry/non-registry shadcn surface has a row,
- each row shows which automation axes are currently covered,
- gaps are expressed as next machine-actionable steps rather than "looks different" notes.
