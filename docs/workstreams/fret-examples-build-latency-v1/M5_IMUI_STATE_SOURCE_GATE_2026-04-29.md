# Fret Examples Build Latency v1 - M5 IMUI State Source Gate - 2026-04-29

Status: complete

## Decision

Move two remaining source-only IMUI state/entry marker checks out of the monolithic
`fret-examples` unit-test module and into `tools/gate_imui_facade_teaching_source.py`.

## Migrated Checks

- `imui_immediate_mode_examples_use_local_state_bridge_reads`
- `workspace_shell_demo_prefers_root_fret_imui_entry_surface`

## Rationale

- Both checks only scan source text.
- The existing IMUI facade/teaching source gate already owns the relevant first-party IMUI examples.
- Keeping these markers in the Python gate avoids compiling `fret-examples` for source-policy
  drift while preserving the same required/forbidden marker coverage.

## Gates

```text
python tools/gate_imui_facade_teaching_source.py
python -m py_compile tools/gate_imui_facade_teaching_source.py
cargo fmt --package fret-examples --check
cargo check -p fret-examples --lib --jobs 1
```
