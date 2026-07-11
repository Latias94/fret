# IMUI Style Theme Editor Proof v1 - Evidence & Gates

Status: Closed
Last updated: 2026-05-25

## Evidence Anchors

- Workstream:
  - `docs/workstreams/imui-style-theme-editor-proof-v1/WORKSTREAM.json`
  - `docs/workstreams/imui-style-theme-editor-proof-v1/DESIGN.md`
  - `docs/workstreams/imui-style-theme-editor-proof-v1/TODO.md`
  - `docs/workstreams/imui-style-theme-editor-proof-v1/MILESTONES.md`
  - `docs/workstreams/imui-style-theme-editor-proof-v1/EVIDENCE_AND_GATES.md`
- Implementation:
  - `ecosystem/fret-ui-editor/src/theme.rs`
  - `ecosystem/fret-ui-editor/src/controls/editor_theme_preset_picker.rs`
  - `ecosystem/fret-ui-editor/src/primitives/readout.rs`
  - `ecosystem/fret-ui-editor/src/controls/mod.rs`
  - `ecosystem/fret-ui-editor/src/imui.rs`
  - `apps/fret-examples/src/editor_notes_demo.rs`
  - `apps/fret-examples/src/editor_notes_device_shell_demo.rs`
  - `apps/fret-examples/src/lib.rs`
- Proof:
  - `ecosystem/fret-ui-editor/tests/imui_adapter_smoke.rs`
  - `ecosystem/fret-ui-editor/tests/imui_surface_policy.rs`
  - `apps/fret-examples/tests/imui_editor_workbench_golden_path_surface.rs`
  - `apps/fret-examples/tests/editor_notes_device_shell_surface.rs`
  - `tools/gate_imui_workstream_source.py`

## Focused Gates

```powershell
cargo fmt --check -p fret-ui-editor
cargo check -p fret-ui-editor --features imui
cargo nextest run -p fret-ui-editor --features imui --no-fail-fast
cargo nextest run -p fret-examples --test imui_editor_workbench_golden_path_surface --no-fail-fast
cargo nextest run -p fret-examples --test editor_notes_device_shell_surface --no-fail-fast
cargo check -p fret-demo --bin imui_editor_workbench_demo
python tools/gate_imui_workstream_source.py
python tools/check_workstream_catalog.py
python -m json.tool docs/workstreams/imui-style-theme-editor-proof-v1/WORKSTREAM.json
git diff --check
```

## 2026-05-25 Slice Results

- PASS: `cargo fmt -p fret-ui-editor`
- PASS: `cargo check -p fret-ui-editor --features imui`
  - Existing warnings only from `crates/fret-ui`: `unexpected cfg` for
    `unstable-retained-bridge` and unused `current_effective_opacity`.
- PASS: `cargo nextest run -p fret-ui-editor --features imui --no-fail-fast`
  - 189 passed.
  - Includes `editor_theme_preset_picker_stamps_listbox_options_and_selected_state`.
  - Includes `editor_theme_preset_picker_click_updates_model_and_replays_reversible_preset`.
  - Includes `default_preset_resets_dense_numeric_scrub_tokens`.
  - Includes `imui_module_stays_a_thin_into_element_adapter_layer`.
- PASS: `python tools/gate_imui_workstream_source.py`
  - The gate now freezes this lane's boundary against `GetStyle`, `PushStyleVar`, `ImGuiStyle`,
    `fret-ui-kit::imui` implementation drift, and non-thin IMUI adapter growth.
- PASS: `python tools/check_workstream_catalog.py`
  - Validated 443 dedicated directories and 47 standalone markdown files.
- PASS: `python -m json.tool docs/workstreams/imui-style-theme-editor-proof-v1/WORKSTREAM.json`
- PASS_WITH_WARNINGS: `git diff --check`
  - No whitespace errors.
  - Existing line-ending warnings remain for `Cargo.lock` and `apps/fret-examples/src/lib.rs`.

## 2026-05-25 Canonical Workbench Integration

- `apps/fret-examples/src/editor_notes_demo.rs` now mounts `EditorThemePresetPicker` in the
  inspector as `editor-notes-demo.inspector.theme-preset`.
- `apps/fret-examples/src/editor_notes_device_shell_demo.rs` reuses the same inspector content and
  initializes the same installed preset model for both desktop rails and mobile drawer content.
- `apps/fret-examples/src/lib.rs` delegates preset key parsing to
  `EditorThemePreset::from_key`, so tools and demos share the editor-owned parser.
- `apps/fret-examples/tests/imui_editor_workbench_golden_path_surface.rs` and
  `apps/fret-examples/tests/editor_notes_device_shell_surface.rs` now prove the canonical route and
  responsive shell cannot silently drop the theme picker path.

Fresh gates:

- PASS: `cargo fmt --check -p fret-examples -p fret-demo`
- PASS: `cargo check -p fret-demo --bin imui_editor_workbench_demo`
  - Existing warnings only from `crates/fret-ui`, `fret-chart`, and `fret-plot`.
- PASS: `cargo nextest run -p fret-examples --test imui_editor_workbench_golden_path_surface --no-fail-fast`
  - 2 passed after rerunning serially; the first parallel attempt timed out while waiting on Cargo
    locks, not from a test failure.
- PASS: `cargo nextest run -p fret-examples --test editor_notes_device_shell_surface --no-fail-fast`
  - 2 passed.
- PASS: `cargo nextest run -p fret-examples parse_editor_theme_preset_key --no-fail-fast`
  - 2 passed, 136 skipped.
- PASS: `python tools/gate_imui_workstream_source.py`
- PASS: `python -m py_compile tools/gate_imui_workstream_source.py`
- PASS: `python tools/check_workstream_catalog.py`
  - Validated 443 dedicated directories and 47 standalone markdown files.
- PASS: `python -m json.tool docs/workstreams/imui-style-theme-editor-proof-v1/WORKSTREAM.json`
- PASS: `python -m json.tool docs/workstreams/imui-editor-workbench-golden-path-v1/WORKSTREAM.json`
- PASS_WITH_WARNINGS: `git diff --check`
  - No whitespace errors.
  - Existing line-ending warnings remain for `Cargo.lock` and `apps/fret-examples/src/lib.rs`.
