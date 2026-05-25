---
title: Shadcn Component Parity Matrix v1 Evidence and Gates
status: active
date: 2026-05-25
---

# Evidence and Gates

## Commands

```powershell
python -m py_compile tools/parity-discovery/shadcn_component_harness_matrix.py
python tools/parity-discovery/shadcn_component_harness_matrix.py
python -m json.tool docs/workstreams/shadcn-component-parity-matrix-v1/artifacts/shadcn_component_harness_matrix_v1.json | Out-Null
python -m json.tool docs/workstreams/shadcn-component-parity-matrix-v1/WORKSTREAM.json | Out-Null
python tools/check_workstream_catalog.py
git diff --check
```

## Evidence

- `tools/parity-discovery/shadcn_component_harness_matrix.py`
- `docs/workstreams/shadcn-component-parity-matrix-v1/MATRIX.md`
- `docs/workstreams/shadcn-component-parity-matrix-v1/artifacts/shadcn_component_harness_matrix_v1.json`
- `docs/shadcn-declarative-progress.md`
- `tools/parity-discovery/manifests/shadcn_parity_coverage_v2.json`
- `docs/workstreams/shadcn-parity-discovery-harness-v2/artifacts/shadcn_parity_suite_report_v2.json`
- `docs/workstreams/component-parity-fact-harness-v1/artifacts/button_group_agent_packet_pilot_v1.json`

## Initial Matrix Summary

- Components: 59.
- Registry components: 54.
- Non-registry surfaces: 5.
- Status counts:
  - `regression_locked`: 9
  - `harness_hardening`: 1
  - `coverage_targeted`: 8
  - `inventory_only`: 36
  - `not_in_harness`: 5
- Axis counts:
  - `source_refs`: 18
  - `upstream_dom_snapshot`: 14
  - `fret_layout`: 18
  - `fret_bundle_semantics`: 10
  - `fret_text_paint`: 1
  - `interaction_script`: 15
  - `responsive_viewport`: 5

## Interpretation

The current harness can already do more than manual screenshot review for selected slices: it can
join upstream source facts, upstream DOM/CSS facts, Fret layout, Fret semantics, interaction scripts,
and packet queues. The depth is still uneven. Most components remain `inventory_only`; the next
useful work is to promote high-risk rows into full harness seeds rather than manually inspecting
screenshots.

## Validation Notes

2026-05-25 local validation:

- `python -m py_compile tools/parity-discovery/shadcn_component_harness_matrix.py`: PASS.
- `python tools/parity-discovery/shadcn_component_harness_matrix.py`: PASS, generated 59 component
  rows.
- Matrix JSON validation: PASS.
- `python -m json.tool docs/workstreams/shadcn-component-parity-matrix-v1/WORKSTREAM.json | Out-Null`:
  PASS.
- `python tools/check_workstream_catalog.py`: PASS, 438 dedicated directories and 47 standalone
  markdown files.
- `git diff --check`: PASS.
