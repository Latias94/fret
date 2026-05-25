---
title: Shadcn Component Harness Matrix v1
status: active
date: 2026-05-25
---

# Shadcn Component Harness Matrix v1

This matrix tracks how far automated parity evidence reaches for each shadcn surface in Fret's self-rendered runtime.

Fret should not compare itself to HTML tree structure. Upstream DOM/CSS snapshots are source references for web-facing shadcn outcomes; Fret proof must come from layout sidecars, bundle schema2 semantics, interaction scripts, text/paint diagnostics, screenshots only when needed, and owner/layer-classified repair queues.

Axis legend:

- `SRC`: upstream source refs are attached.
- `UP-DOM`: upstream DOM/CSS snapshot evidence exists.
- `LAYOUT`: Fret layout/geometry evidence exists.
- `SEM`: Fret bundle semantics evidence exists.
- `TEXT`: Fret text/paint evidence exists.
- `BEHAV`: interaction/behavior diag script exists.
- `RESP`: responsive or non-desktop viewport coverage exists.

## Summary

```json
{
  "axis_component_counts": {
    "fret_bundle_semantics": 10,
    "fret_layout": 18,
    "fret_text_paint": 1,
    "interaction_script": 15,
    "responsive_viewport": 5,
    "source_refs": 18,
    "upstream_dom_snapshot": 14
  },
  "component_count": 59,
  "non_registry_surface_count": 5,
  "registry_component_count": 54,
  "status_counts": {
    "coverage_targeted": 8,
    "harness_hardening": 1,
    "inventory_only": 36,
    "not_in_harness": 5,
    "regression_locked": 9
  }
}
```

## Component Matrix

| Component | Kind | Impl | Harness status | Axes | Targets | Reports | Queues | Next gap |
| --- | --- | --- | --- | --- | ---: | ---: | --- | --- |
| accordion | registry | Present | inventory_only | - | 0 | 0 | repair=0, hardening=0, gate=0 | add_upstream_source_refs |
| alert | registry | Present | inventory_only | - | 0 | 0 | repair=0, hardening=0, gate=0 | add_upstream_source_refs |
| alert-dialog | registry | Present | inventory_only | - | 0 | 0 | repair=0, hardening=0, gate=0 | add_upstream_source_refs |
| aspect-ratio | registry | Present | inventory_only | - | 0 | 0 | repair=0, hardening=0, gate=0 | add_upstream_source_refs |
| avatar | registry | Present | inventory_only | - | 0 | 0 | repair=0, hardening=0, gate=0 | add_upstream_source_refs |
| badge | registry | Present | inventory_only | - | 0 | 0 | repair=0, hardening=0, gate=0 | add_upstream_source_refs |
| breadcrumb | registry | Present | inventory_only | - | 0 | 0 | repair=0, hardening=0, gate=0 | add_upstream_source_refs |
| button | registry | Present | inventory_only | - | 0 | 0 | repair=0, hardening=0, gate=0 | add_upstream_source_refs |
| button-group | registry | Present | harness_hardening | SRC, UP-DOM, LAYOUT, SEM, TEXT | 1 | 1 | repair=0, hardening=1, gate=9 | add_behavior_diag_script |
| calendar | registry | Present | coverage_targeted | SRC, UP-DOM, LAYOUT, BEHAV, RESP | 4 | 0 | repair=0, hardening=0, gate=0 | capture_bundle_schema2_semantics |
| card | registry | Present | inventory_only | - | 0 | 0 | repair=0, hardening=0, gate=0 | add_upstream_source_refs |
| carousel | registry | Defer | not_in_harness | - | 0 | 0 | repair=0, hardening=0, gate=0 | add_upstream_source_refs |
| chart | registry | Defer | not_in_harness | - | 0 | 0 | repair=0, hardening=0, gate=0 | add_upstream_source_refs |
| checkbox | registry | Present | inventory_only | - | 0 | 0 | repair=0, hardening=0, gate=0 | add_upstream_source_refs |
| collapsible | registry | Present | inventory_only | - | 0 | 0 | repair=0, hardening=0, gate=0 | add_upstream_source_refs |
| combobox | non_registry | Present | coverage_targeted | SRC, UP-DOM, LAYOUT, BEHAV, RESP | 2 | 0 | repair=0, hardening=0, gate=0 | capture_bundle_schema2_semantics |
| command | registry | Present | inventory_only | - | 0 | 0 | repair=0, hardening=0, gate=0 | add_upstream_source_refs |
| context-menu | registry | Present | regression_locked | SRC, UP-DOM, LAYOUT, SEM, BEHAV | 1 | 1 | repair=0, hardening=0, gate=0 | add_text_paint_or_paint_snapshot_gate |
| data-table | non_registry | Present | coverage_targeted | SRC, LAYOUT, BEHAV | 1 | 0 | repair=0, hardening=0, gate=0 | capture_upstream_dom_snapshot |
| date-picker | non_registry | Present | inventory_only | - | 0 | 0 | repair=0, hardening=0, gate=0 | add_upstream_source_refs |
| dialog | registry | Present | regression_locked | SRC, UP-DOM, LAYOUT, SEM, BEHAV | 1 | 1 | repair=0, hardening=0, gate=0 | add_text_paint_or_paint_snapshot_gate |
| drawer | registry | Present | coverage_targeted | SRC, LAYOUT, BEHAV, RESP | 1 | 0 | repair=0, hardening=0, gate=0 | capture_upstream_dom_snapshot |
| dropdown-menu | registry | Present | coverage_targeted | SRC, UP-DOM, LAYOUT, BEHAV, RESP | 1 | 0 | repair=0, hardening=0, gate=0 | capture_bundle_schema2_semantics |
| empty | registry | Present | inventory_only | - | 0 | 0 | repair=0, hardening=0, gate=0 | add_upstream_source_refs |
| field | registry | Present | inventory_only | - | 0 | 0 | repair=0, hardening=0, gate=0 | add_upstream_source_refs |
| form | registry | Present | inventory_only | - | 0 | 0 | repair=0, hardening=0, gate=0 | add_upstream_source_refs |
| hover-card | registry | Present | regression_locked | SRC, UP-DOM, LAYOUT, SEM, BEHAV | 1 | 1 | repair=0, hardening=0, gate=0 | add_text_paint_or_paint_snapshot_gate |
| input | registry | Present | coverage_targeted | SRC, UP-DOM, LAYOUT | 1 | 0 | repair=0, hardening=0, gate=0 | capture_bundle_schema2_semantics |
| input-group | registry | Present | inventory_only | - | 0 | 0 | repair=0, hardening=0, gate=0 | add_upstream_source_refs |
| input-otp | registry | Present | regression_locked | SRC, UP-DOM, LAYOUT, SEM, BEHAV | 1 | 1 | repair=0, hardening=0, gate=0 | add_text_paint_or_paint_snapshot_gate |
| item | registry | Present | inventory_only | - | 0 | 0 | repair=0, hardening=0, gate=0 | add_upstream_source_refs |
| kbd | registry | Present | inventory_only | - | 0 | 0 | repair=0, hardening=0, gate=0 | add_upstream_source_refs |
| label | registry | Present | inventory_only | - | 0 | 0 | repair=0, hardening=0, gate=0 | add_upstream_source_refs |
| menubar | registry | Present | regression_locked | SRC, UP-DOM, LAYOUT, SEM, BEHAV | 1 | 1 | repair=0, hardening=0, gate=0 | add_text_paint_or_paint_snapshot_gate |
| native-select | registry | Defer | not_in_harness | - | 0 | 0 | repair=0, hardening=0, gate=0 | add_upstream_source_refs |
| navigation-menu | registry | Present | regression_locked | SRC, UP-DOM, LAYOUT, SEM, BEHAV | 1 | 1 | repair=0, hardening=0, gate=0 | add_text_paint_or_paint_snapshot_gate |
| pagination | registry | Present | inventory_only | - | 0 | 0 | repair=0, hardening=0, gate=0 | add_upstream_source_refs |
| popover | registry | Present | coverage_targeted | SRC, LAYOUT | 1 | 0 | repair=0, hardening=0, gate=0 | capture_upstream_dom_snapshot |
| progress | registry | Present | inventory_only | - | 0 | 0 | repair=0, hardening=0, gate=0 | add_upstream_source_refs |
| radio-group | registry | Present | inventory_only | - | 0 | 0 | repair=0, hardening=0, gate=0 | add_upstream_source_refs |
| resizable | registry | Present | inventory_only | - | 0 | 0 | repair=0, hardening=0, gate=0 | add_upstream_source_refs |
| scroll-area | registry | Present | inventory_only | - | 0 | 0 | repair=0, hardening=0, gate=0 | add_upstream_source_refs |
| select | registry | Present | coverage_targeted | SRC, LAYOUT, BEHAV | 1 | 0 | repair=0, hardening=0, gate=0 | capture_upstream_dom_snapshot |
| separator | registry | Present | inventory_only | - | 0 | 0 | repair=0, hardening=0, gate=0 | add_upstream_source_refs |
| sheet | registry | Present | regression_locked | SRC, UP-DOM, LAYOUT, SEM, BEHAV, RESP | 1 | 1 | repair=0, hardening=0, gate=0 | add_text_paint_or_paint_snapshot_gate |
| sidebar | registry | Present | inventory_only | - | 0 | 0 | repair=0, hardening=0, gate=0 | add_upstream_source_refs |
| skeleton | registry | Present | inventory_only | - | 0 | 0 | repair=0, hardening=0, gate=0 | add_upstream_source_refs |
| slider | registry | Present | inventory_only | - | 0 | 0 | repair=0, hardening=0, gate=0 | add_upstream_source_refs |
| sonner | registry | Present | inventory_only | - | 0 | 0 | repair=0, hardening=0, gate=0 | add_upstream_source_refs |
| spinner | registry | Present | inventory_only | - | 0 | 0 | repair=0, hardening=0, gate=0 | add_upstream_source_refs |
| switch | registry | Present | inventory_only | - | 0 | 0 | repair=0, hardening=0, gate=0 | add_upstream_source_refs |
| table | registry | Present | regression_locked | SRC, UP-DOM, LAYOUT, SEM, BEHAV | 1 | 1 | repair=0, hardening=0, gate=0 | add_text_paint_or_paint_snapshot_gate |
| tabs | registry | Present | inventory_only | - | 0 | 0 | repair=0, hardening=0, gate=0 | add_upstream_source_refs |
| textarea | registry | Present | inventory_only | - | 0 | 0 | repair=0, hardening=0, gate=0 | add_upstream_source_refs |
| toast | non_registry | Skip | not_in_harness | - | 0 | 0 | repair=0, hardening=0, gate=0 | add_upstream_source_refs |
| toggle | registry | Present | inventory_only | - | 0 | 0 | repair=0, hardening=0, gate=0 | add_upstream_source_refs |
| toggle-group | registry | Present | inventory_only | - | 0 | 0 | repair=0, hardening=0, gate=0 | add_upstream_source_refs |
| tooltip | registry | Present | regression_locked | SRC, UP-DOM, LAYOUT, SEM, BEHAV | 1 | 1 | repair=0, hardening=0, gate=0 | add_text_paint_or_paint_snapshot_gate |
| typography | non_registry | Skip | not_in_harness | - | 0 | 0 | repair=0, hardening=0, gate=0 | add_upstream_source_refs |

## Interpretation

- `regression_locked` means the current suite report has no repair or hardening queue for that component slice. It does not mean every state, breakpoint, DPI, font metric, and interaction path is covered.
- `coverage_targeted` means a priority target exists in the manifest, but it is not yet represented as a current suite report.
- `inventory_only` means the component exists in the shadcn inventory but does not yet have a harness seed.
- The next automation step is to turn high-risk `inventory_only` and `coverage_targeted` rows into fixtures with upstream source refs, Fret `test_id`s, diag scripts, and packet checks.
