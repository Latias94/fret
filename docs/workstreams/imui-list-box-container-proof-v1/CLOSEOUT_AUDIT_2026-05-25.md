# IMUI List Box Container Proof v1 - Closeout Audit - 2026-05-25

Status: closed
Last updated: 2026-05-25

## Objective

Close the narrow Dear ImGui `BeginListBox`-style container proof after shipping a semantic,
scrollable row host without generic collection helper growth.

## Completion Checklist

| Requirement | Evidence |
| --- | --- |
| `ListBoxOptions` exposes only container/layout/test-id knobs | `ecosystem/fret-ui-kit/src/imui/options/containers.rs` |
| ListBox control stamps ListBox semantics and hosts children | `ecosystem/fret-ui-kit/src/imui/list_box_controls.rs` |
| Thin facade/writer path exists | `ecosystem/fret-ui-kit/src/imui/facade_writer.rs` |
| `fret-imui` focused proof passes | `ecosystem/fret-imui/src/tests/composition/layout_collections.rs` |
| Source gate prevents collection-helper widening | `tools/gate_imui_workstream_source.py` |
| Fresh gates recorded | `docs/workstreams/imui-list-box-container-proof-v1/EVIDENCE_AND_GATES.md` |

## Residual Boundaries

- No selection model, filtering/typeahead, active-descendant policy, command package,
  virtualization, or overlay recipe policy shipped in this lane.
- Collection-helper widening remains closed by
  `docs/workstreams/imui-collection-helper-readiness-v1/CLOSEOUT_AUDIT_2026-04-24.md`.

## Outcome

The ListBox container proof is closed. Any future collection helper work needs a new proof-led
follow-on naming the exact shared helper shape.
