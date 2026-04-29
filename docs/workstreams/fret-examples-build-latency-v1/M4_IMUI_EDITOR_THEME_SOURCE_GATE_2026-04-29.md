# Fret Examples Build Latency v1 - M4 IMUI Editor Theme Source Gate - 2026-04-29

Status: complete

## Decision

Move the remaining source-only IMUI editor proof theme/preset checks out of the monolithic
`fret-examples` unit-test module and into `tools/gate_imui_facade_teaching_source.py`.

## Rationale

- The checks only assert source markers in `imui_editor_proof_demo.rs`.
- They do not need Rust type checking or launched diagnostics.
- The same Python gate already owns the current IMUI teaching-surface source policy and reads
  `imui_editor_proof_demo.rs`, so keeping the markers there avoids another one-off script.

## Migrated Checks

- `imui_editor_proof_demo_defaults_to_imgui_like_dense_preset_for_editor_grade_launches`
- `imui_editor_proof_demo_keeps_a_demo_owned_fixed_editor_theme`

## Gates

```text
python tools/gate_imui_facade_teaching_source.py
python -m py_compile tools/gate_imui_facade_teaching_source.py
cargo check -p fret-examples --lib --jobs 1
```
