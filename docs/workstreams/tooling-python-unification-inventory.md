# Tooling Python Unification Inventory

Status: active inventory
Last updated: 2026-03-09

## Goal

Track the remaining `tools/*.ps1` scripts after the main aggregate entrypoints (`pre_release`,
`gates_full`, `gates_fast`) were unified to Python.

This note is intentionally operational rather than aspirational:

- identify which PowerShell scripts already have a Python replacement,
- identify which ones are still Windows-/PowerShell-oriented on purpose,
- identify which ones are good next candidates for Python migration,
- avoid deleting wrappers or diagnostics runners blindly.

## Current policy

- Prefer `python tools/*.py` as the canonical documented entrypoint whenever a Python version exists.
- Keep `.ps1` only when one of the following is true:
  - it is a temporary compatibility wrapper,
  - it is tightly tied to Windows / PowerShell ergonomics,
  - it has no Python equivalent yet and still serves a real workflow.

## Classification

| Script | Current status | Category | Recommended next step |
| --- | --- | --- | --- |
| `tools/audit_crate.ps1` | `tools/audit_crate.py` exists | Compatibility wrapper | Update remaining docs first, then consider delete |
| `tools/gate_fret_builder_only_surface.ps1` | deleted; `tools/gate_fret_builder_only_surface.py` is canonical | Removed wrapper | Done |
| `tools/gate_fret_launch_surface_contract.ps1` | deleted; `tools/gate_fret_launch_surface_contract.py` is canonical | Removed wrapper | Done |
| `tools/gate_no_mvu_in_cookbook.ps1` | deleted; `tools/gate_no_mvu_in_cookbook.py` is canonical | Removed wrapper | Done |
| `tools/gate_no_mvu_in_tree.ps1` | deleted; `tools/gate_no_mvu_in_tree.py` is canonical | Removed wrapper | Done |
| `tools/gate_no_public_stack_in_ui_kit.ps1` | deleted; `tools/gate_no_public_stack_in_ui_kit.py` is canonical | Removed wrapper | Done |
| `tools/gate_no_stack_in_cookbook.ps1` | deleted; `tools/gate_no_stack_in_cookbook.py` is canonical | Removed wrapper | Done |
| `tools/gate_no_stack_in_examples.ps1` | deleted; `tools/gate_no_stack_in_examples.py` is canonical | Removed wrapper | Done |
| `tools/gate_no_stack_in_ui_gallery_shell.ps1` | deleted; `tools/gate_no_stack_in_ui_gallery_shell.py` is canonical | Removed wrapper | Done |
| `tools/diag_gate_action_first_authoring_v1.ps1` | deleted; `tools/diag_gate_action_first_authoring_v1.py` is canonical | Removed wrapper | Done |
| `tools/diag_gate_interaction_kernel_v1.ps1` | deleted; `tools/diag_gate_interaction_kernel_v1.py` is canonical | Removed wrapper | Done |
| `tools/diag_scripts_refresh.ps1` | no Python twin yet | Python migration candidate | Medium-value maintenance candidate |
| `tools/gates_delinea_fast.ps1` | no Python twin yet | Python migration candidate | Candidate if this workstream stays active |
| `tools/gates_wasm_smoke.ps1` | no Python twin yet | Mixed shell / platform runner | Audit before migrating |
| `tools/perf/diag_drop_shadow_v1_gate.ps1` | no Python twin yet | Perf/diag specialized runner | Leave until perf workflow unification pass |
| `tools/perf/diag_extras_marquee_gate.ps1` | `tools/perf/diag_extras_marquee_gate.py` exists | Compatibility / Windows history | Prefer Python in docs, then audit delete |
| `tools/rg-safe.ps1` | no Python twin | PowerShell utility | Keep for now; only migrate if repo-search workflow needs parity |

## Recommended order

### Phase 1 — Remove easy wrappers

Completed:

- `tools/gate_fret_builder_only_surface.ps1`
- `tools/gate_fret_launch_surface_contract.ps1`
- `tools/gate_no_mvu_in_cookbook.ps1`
- `tools/gate_no_mvu_in_tree.ps1`
- `tools/gate_no_public_stack_in_ui_kit.ps1`
- `tools/gate_no_stack_in_cookbook.ps1`
- `tools/gate_no_stack_in_examples.ps1`
- `tools/gate_no_stack_in_ui_gallery_shell.ps1`

Guardrail used:

- reference audit first,
- docs updated to Python command,
- wrapper deletion in a focused change.

### Phase 2 — Migrate high-value diag runners

Completed:

1. `tools/diag_gate_action_first_authoring_v1.ps1`
2. `tools/diag_gate_interaction_kernel_v1.ps1`

Why these were prioritized:

- they appear in active workstream docs,
- they are repeatedly used as “canonical” gates,
- cross-platform parity here brings higher value than deleting small wrappers.

### Phase 3 — Audit intentional PowerShell surfaces

Keep until a clear product need exists:

- `tools/rg-safe.ps1`
- specialized Windows/perf helpers
- any script that primarily exists for PowerShell shell ergonomics rather than repo policy

## Current recommendation

Do **not** keep deleting `.ps1` blindly.

The safest next move is:

1. clear the low-risk compatibility wrappers that already have Python twins,
2. then migrate the two high-value diagnostics gate runners,
3. leave the Windows-/PowerShell-native utility surfaces for a separate decision.
