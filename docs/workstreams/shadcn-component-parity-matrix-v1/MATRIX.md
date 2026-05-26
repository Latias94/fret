---
title: Shadcn Component Harness Matrix v1
status: active
date: 2026-05-26
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

State-depth legend:

- `DIS`: disabled / aria-disabled / focusable-disabled evidence.
- `HOV`: hover evidence.
- `FOCUS-VIS`: focus-visible or focus-ring evidence.
- `PRESS`: pressed-state evidence.
- `DRAG`: splitter drag or resize evidence.
- `OPEN`: open / expanded evidence.
- `KEY`: keyboard path evidence.
- `MOB`: mobile, constrained, or responsive viewport evidence.
- `RTL`: right-to-left evidence.
- `TEXT-MET`: text metrics or style-aware text measurement evidence.
- `PAINT`: paint, token, chrome, color, border, radius, or contrast evidence.

## Summary

```json
{
  "axis_component_counts": {
    "fret_bundle_semantics": 43,
    "fret_layout": 43,
    "fret_text_paint": 23,
    "interaction_script": 43,
    "responsive_viewport": 9,
    "source_refs": 43,
    "upstream_dom_snapshot": 43
  },
  "component_count": 59,
  "non_registry_surface_count": 5,
  "registry_component_count": 54,
  "state_depth_component_counts": {
    "disabled": 17,
    "drag": 3,
    "focus_visible": 14,
    "hover": 12,
    "keyboard": 26,
    "mobile": 14,
    "open": 25,
    "paint_token": 41,
    "pressed": 1,
    "rtl": 23,
    "text_metrics": 23
  },
  "status_counts": {
    "harness_hardening": 1,
    "inventory_only": 11,
    "not_in_harness": 5,
    "regression_locked": 42
  }
}
```

## Component Matrix

| Component | Kind | Impl | Harness status | Axes | Depth | Missing depth | Targets | Reports | Queues | Next gap |
| --- | --- | --- | --- | --- | --- | --- | ---: | ---: | --- | --- |
| accordion | registry | Present | regression_locked | SRC, UP-DOM, LAYOUT, SEM, TEXT, BEHAV | DIS, OPEN, RTL, TEXT-MET, PAINT | KEY | 1 | 1 | repair=0, hardening=0, gate=0 | expand_keyboard_state_depth |
| alert | registry | Present | regression_locked | SRC, UP-DOM, LAYOUT, SEM, TEXT, BEHAV | KEY, RTL, TEXT-MET, PAINT | ok | 1 | 1 | repair=0, hardening=0, gate=0 | state_depth_model_satisfied |
| alert-dialog | registry | Present | regression_locked | SRC, UP-DOM, LAYOUT, SEM, TEXT, BEHAV | OPEN, KEY, MOB, RTL, TEXT-MET, PAINT | ok | 1 | 1 | repair=0, hardening=0, gate=0 | state_depth_model_satisfied |
| aspect-ratio | registry | Present | regression_locked | SRC, UP-DOM, LAYOUT, SEM, TEXT, BEHAV | RTL, TEXT-MET, PAINT | - | 1 | 1 | repair=0, hardening=0, gate=0 | state_depth_model_satisfied |
| avatar | registry | Present | regression_locked | SRC, UP-DOM, LAYOUT, SEM, TEXT, BEHAV | OPEN, KEY, RTL, TEXT-MET, PAINT | - | 1 | 1 | repair=0, hardening=0, gate=0 | state_depth_model_satisfied |
| badge | registry | Present | regression_locked | SRC, UP-DOM, LAYOUT, SEM, TEXT, BEHAV | HOV, FOCUS-VIS, KEY, RTL, TEXT-MET, PAINT | ok | 1 | 1 | repair=0, hardening=0, gate=0 | state_depth_model_satisfied |
| breadcrumb | registry | Present | regression_locked | SRC, UP-DOM, LAYOUT, SEM, TEXT, BEHAV, RESP | DIS, HOV, OPEN, KEY, MOB, RTL, TEXT-MET, PAINT | - | 1 | 1 | repair=0, hardening=0, gate=0 | state_depth_model_satisfied |
| button | registry | Present | regression_locked | SRC, UP-DOM, LAYOUT, SEM, TEXT, BEHAV | DIS, HOV, FOCUS-VIS, PRESS, KEY, TEXT-MET, PAINT | ok | 1 | 1 | repair=0, hardening=0, gate=0 | state_depth_model_satisfied |
| button-group | registry | Present | regression_locked | SRC, UP-DOM, LAYOUT, SEM, TEXT, BEHAV | DIS, TEXT-MET, PAINT | KEY | 1 | 1 | repair=0, hardening=0, gate=0 | expand_keyboard_state_depth |
| calendar | registry | Present | regression_locked | SRC, UP-DOM, LAYOUT, SEM, BEHAV, RESP | HOV, FOCUS-VIS, OPEN, MOB, PAINT | TEXT-MET | 4 | 1 | repair=0, hardening=0, gate=0 | add_text_paint_or_paint_snapshot_gate |
| card | registry | Present | regression_locked | SRC, UP-DOM, LAYOUT, SEM, TEXT, BEHAV | KEY, RTL, TEXT-MET, PAINT | - | 1 | 1 | repair=0, hardening=0, gate=0 | state_depth_model_satisfied |
| carousel | registry | Defer | not_in_harness | - | - | - | 0 | 0 | repair=0, hardening=0, gate=0 | add_upstream_source_refs |
| chart | registry | Defer | not_in_harness | - | - | - | 0 | 0 | repair=0, hardening=0, gate=0 | add_upstream_source_refs |
| checkbox | registry | Present | regression_locked | SRC, UP-DOM, LAYOUT, SEM, TEXT, BEHAV | DIS, FOCUS-VIS, KEY, RTL, TEXT-MET, PAINT | ok | 1 | 1 | repair=0, hardening=0, gate=0 | state_depth_model_satisfied |
| collapsible | registry | Present | regression_locked | SRC, UP-DOM, LAYOUT, SEM, TEXT, BEHAV | DIS, OPEN, KEY, RTL, TEXT-MET, PAINT | ok | 1 | 1 | repair=0, hardening=0, gate=0 | state_depth_model_satisfied |
| combobox | non_registry | Present | regression_locked | SRC, UP-DOM, LAYOUT, SEM, BEHAV, RESP | OPEN, MOB | KEY, TEXT-MET, PAINT | 2 | 1 | repair=0, hardening=0, gate=0 | add_text_paint_or_paint_snapshot_gate |
| command | registry | Present | regression_locked | SRC, UP-DOM, LAYOUT, SEM, TEXT, BEHAV | DIS, FOCUS-VIS, OPEN, KEY, RTL, TEXT-MET, PAINT | ok | 1 | 1 | repair=0, hardening=0, gate=0 | state_depth_model_satisfied |
| context-menu | registry | Present | regression_locked | SRC, UP-DOM, LAYOUT, SEM, BEHAV | OPEN, KEY, PAINT | ok | 1 | 1 | repair=0, hardening=0, gate=0 | add_text_paint_or_paint_snapshot_gate |
| data-table | non_registry | Present | regression_locked | SRC, UP-DOM, LAYOUT, SEM, BEHAV | RTL, PAINT | KEY | 1 | 1 | repair=0, hardening=0, gate=0 | add_text_paint_or_paint_snapshot_gate |
| date-picker | non_registry | Present | regression_locked | SRC, UP-DOM, LAYOUT, SEM, BEHAV, RESP | OPEN, KEY, MOB, PAINT | ok | 1 | 1 | repair=0, hardening=0, gate=0 | add_text_paint_or_paint_snapshot_gate |
| dialog | registry | Present | regression_locked | SRC, UP-DOM, LAYOUT, SEM, BEHAV | DIS, HOV, FOCUS-VIS, OPEN, PAINT | KEY | 1 | 1 | repair=0, hardening=0, gate=0 | add_text_paint_or_paint_snapshot_gate |
| drawer | registry | Present | regression_locked | SRC, UP-DOM, LAYOUT, SEM, BEHAV, RESP | OPEN, MOB, PAINT | ok | 1 | 1 | repair=0, hardening=0, gate=0 | add_text_paint_or_paint_snapshot_gate |
| dropdown-menu | registry | Present | regression_locked | SRC, UP-DOM, LAYOUT, SEM, BEHAV, RESP | OPEN, KEY, MOB, PAINT | ok | 1 | 1 | repair=0, hardening=0, gate=0 | add_text_paint_or_paint_snapshot_gate |
| empty | registry | Present | regression_locked | SRC, UP-DOM, LAYOUT, SEM, TEXT, BEHAV | DRAG, KEY, MOB, RTL, TEXT-MET, PAINT | ok | 1 | 1 | repair=0, hardening=0, gate=0 | state_depth_model_satisfied |
| field | registry | Present | regression_locked | SRC, UP-DOM, LAYOUT, SEM, TEXT, BEHAV, RESP | MOB, TEXT-MET, PAINT | - | 1 | 1 | repair=0, hardening=0, gate=0 | state_depth_model_satisfied |
| form | registry | Present | regression_locked | SRC, UP-DOM, LAYOUT, SEM, TEXT, BEHAV | DIS, RTL, TEXT-MET, PAINT | - | 1 | 1 | repair=0, hardening=0, gate=0 | state_depth_model_satisfied |
| hover-card | registry | Present | regression_locked | SRC, UP-DOM, LAYOUT, SEM, BEHAV | HOV, OPEN, PAINT | KEY | 1 | 1 | repair=0, hardening=0, gate=0 | add_text_paint_or_paint_snapshot_gate |
| input | registry | Present | regression_locked | SRC, UP-DOM, LAYOUT, SEM, BEHAV | PAINT | DIS, FOCUS-VIS, KEY | 1 | 1 | repair=0, hardening=0, gate=0 | add_text_paint_or_paint_snapshot_gate |
| input-group | registry | Present | regression_locked | SRC, UP-DOM, LAYOUT, SEM, TEXT, BEHAV | DIS, FOCUS-VIS, OPEN, KEY, RTL, TEXT-MET, PAINT | - | 1 | 1 | repair=0, hardening=0, gate=0 | state_depth_model_satisfied |
| input-otp | registry | Present | regression_locked | SRC, UP-DOM, LAYOUT, SEM, BEHAV | DIS, PAINT | FOCUS-VIS, KEY | 1 | 1 | repair=0, hardening=0, gate=0 | add_text_paint_or_paint_snapshot_gate |
| item | registry | Present | regression_locked | SRC, UP-DOM, LAYOUT, SEM, TEXT, BEHAV | HOV, FOCUS-VIS, OPEN, KEY, RTL, TEXT-MET, PAINT | ok | 1 | 1 | repair=0, hardening=0, gate=0 | state_depth_model_satisfied |
| kbd | registry | Present | regression_locked | SRC, UP-DOM, LAYOUT, SEM, TEXT, BEHAV | KEY, RTL, TEXT-MET, PAINT | ok | 1 | 1 | repair=0, hardening=0, gate=0 | state_depth_model_satisfied |
| label | registry | Present | regression_locked | SRC, UP-DOM, LAYOUT, SEM, TEXT, BEHAV | DIS, KEY, RTL, TEXT-MET, PAINT | ok | 1 | 1 | repair=0, hardening=0, gate=0 | state_depth_model_satisfied |
| menubar | registry | Present | regression_locked | SRC, UP-DOM, LAYOUT, SEM, BEHAV | DIS, OPEN, PAINT | KEY | 1 | 1 | repair=0, hardening=0, gate=0 | add_text_paint_or_paint_snapshot_gate |
| native-select | registry | Defer | not_in_harness | - | - | - | 0 | 0 | repair=0, hardening=0, gate=0 | add_upstream_source_refs |
| navigation-menu | registry | Present | regression_locked | SRC, UP-DOM, LAYOUT, SEM, BEHAV | HOV, FOCUS-VIS, OPEN, KEY, PAINT | ok | 1 | 1 | repair=0, hardening=0, gate=0 | add_text_paint_or_paint_snapshot_gate |
| pagination | registry | Present | regression_locked | SRC, UP-DOM, LAYOUT, SEM, TEXT, BEHAV | DIS, FOCUS-VIS, OPEN, KEY, MOB, RTL, TEXT-MET, PAINT | - | 1 | 1 | repair=0, hardening=0, gate=0 | state_depth_model_satisfied |
| popover | registry | Present | regression_locked | SRC, UP-DOM, LAYOUT, SEM, BEHAV | OPEN, KEY, MOB | PAINT | 1 | 1 | repair=0, hardening=0, gate=0 | add_text_paint_or_paint_snapshot_gate |
| progress | registry | Present | regression_locked | SRC, UP-DOM, LAYOUT, SEM, BEHAV | RTL, PAINT | ok | 1 | 1 | repair=0, hardening=0, gate=0 | add_text_paint_or_paint_snapshot_gate |
| radio-group | registry | Present | regression_locked | SRC, UP-DOM, LAYOUT, SEM, TEXT, BEHAV | DIS, FOCUS-VIS, OPEN, KEY, RTL, TEXT-MET, PAINT | ok | 1 | 1 | repair=0, hardening=0, gate=0 | state_depth_model_satisfied |
| resizable | registry | Present | regression_locked | SRC, UP-DOM, LAYOUT, SEM, BEHAV | DRAG, KEY, RTL, PAINT | ok | 1 | 1 | repair=0, hardening=0, gate=0 | add_text_paint_or_paint_snapshot_gate |
| scroll-area | registry | Present | inventory_only | - | - | - | 0 | 0 | repair=0, hardening=0, gate=0 | add_upstream_source_refs |
| select | registry | Present | regression_locked | SRC, UP-DOM, LAYOUT, SEM, BEHAV | OPEN, KEY, MOB, PAINT | ok | 1 | 1 | repair=0, hardening=0, gate=0 | add_text_paint_or_paint_snapshot_gate |
| separator | registry | Present | inventory_only | - | - | - | 0 | 0 | repair=0, hardening=0, gate=0 | add_upstream_source_refs |
| sheet | registry | Present | regression_locked | SRC, UP-DOM, LAYOUT, SEM, BEHAV, RESP | DIS, HOV, FOCUS-VIS, OPEN, MOB, PAINT | KEY | 1 | 1 | repair=0, hardening=0, gate=0 | add_text_paint_or_paint_snapshot_gate |
| sidebar | registry | Present | harness_hardening | SRC, UP-DOM, LAYOUT, SEM, TEXT, BEHAV, RESP | HOV, FOCUS-VIS, DRAG, OPEN, KEY, MOB, RTL, TEXT-MET, PAINT | ok | 1 | 1 | repair=0, hardening=1, gate=1 | state_depth_model_satisfied |
| skeleton | registry | Present | inventory_only | - | - | - | 0 | 0 | repair=0, hardening=0, gate=0 | add_upstream_source_refs |
| slider | registry | Present | inventory_only | - | - | - | 0 | 0 | repair=0, hardening=0, gate=0 | add_upstream_source_refs |
| sonner | registry | Present | inventory_only | - | - | - | 0 | 0 | repair=0, hardening=0, gate=0 | add_upstream_source_refs |
| spinner | registry | Present | inventory_only | - | - | - | 0 | 0 | repair=0, hardening=0, gate=0 | add_upstream_source_refs |
| switch | registry | Present | inventory_only | - | - | - | 0 | 0 | repair=0, hardening=0, gate=0 | add_upstream_source_refs |
| table | registry | Present | regression_locked | SRC, UP-DOM, LAYOUT, SEM, BEHAV | HOV, PAINT | ok | 1 | 1 | repair=0, hardening=0, gate=0 | add_text_paint_or_paint_snapshot_gate |
| tabs | registry | Present | inventory_only | - | - | - | 0 | 0 | repair=0, hardening=0, gate=0 | add_upstream_source_refs |
| textarea | registry | Present | inventory_only | - | - | - | 0 | 0 | repair=0, hardening=0, gate=0 | add_upstream_source_refs |
| toast | non_registry | Skip | not_in_harness | - | - | - | 0 | 0 | repair=0, hardening=0, gate=0 | add_upstream_source_refs |
| toggle | registry | Present | inventory_only | - | - | - | 0 | 0 | repair=0, hardening=0, gate=0 | add_upstream_source_refs |
| toggle-group | registry | Present | inventory_only | - | - | - | 0 | 0 | repair=0, hardening=0, gate=0 | add_upstream_source_refs |
| tooltip | registry | Present | regression_locked | SRC, UP-DOM, LAYOUT, SEM, BEHAV | DIS, HOV, FOCUS-VIS, OPEN, KEY, PAINT | ok | 1 | 1 | repair=0, hardening=0, gate=0 | add_text_paint_or_paint_snapshot_gate |
| typography | non_registry | Skip | not_in_harness | - | - | - | 0 | 0 | repair=0, hardening=0, gate=0 | add_upstream_source_refs |

## Interpretation

- `regression_locked` means the current suite report has no repair or hardening queue for that component slice. It does not mean every state, breakpoint, DPI, font metric, and interaction path is covered.
- `Depth` records state signals proven by manifest targets, component packets, validation gates, and Fret diagnostics summaries. `Missing depth` is filtered through component-specific applicability so irrelevant states are not treated as gaps.
- `coverage_targeted` means a priority target exists in the manifest, but it is not yet represented as a current suite report.
- `inventory_only` means the component exists in the shadcn inventory but does not yet have a harness seed.
- The next automation step is to turn high-risk `inventory_only` and `coverage_targeted` rows into fixtures with upstream source refs, Fret `test_id`s, diag scripts, and packet checks.
