# ImUi Text Input Policy Depth v1 Evidence And Gates

Status: Closed
Last updated: 2026-05-04

## Reference Evidence

- `repo-ref/imgui/imgui.h`: `ImGuiInputTextFlags_ReadOnly`,
  `ImGuiInputTextFlags_AutoSelectAll`, and `ImGuiInputTextFlags_AllowTabInput`.
- `repo-ref/imgui/imgui_widgets.cpp`: read-only handling in `InputTextEx` and select-all activation
  behavior; `AllowTabInput` remains opt-in rather than default multiline behavior.
- `docs/adr/0066-fret-ui-runtime-contract-surface.md`: `fret-ui` remains mechanism-only.

## Implementation Anchors

- `crates/fret-ui/src/element.rs`
- `crates/fret-ui/src/text/input/widget.rs`
- `crates/fret-ui/src/text/area/widget.rs`
- `ecosystem/fret-ui-kit/src/imui/options/controls.rs`
- `ecosystem/fret-ui-kit/src/imui/facade_support.rs`
- `ecosystem/fret-ui-kit/src/imui/text_controls.rs`
- `ecosystem/fret-imui/src/frontend.rs`
- `ecosystem/fret-imui/src/tests/models_text.rs`
- `apps/fret-cookbook/examples/imui_action_basics.rs`
- `apps/fret-cookbook/src/lib.rs`
- `docs/workstreams/imui-text-input-policy-depth-v1/GOAL_BACKWARD_AUDIT_2026-05-04.md`
- `docs/workstreams/imui-text-input-policy-depth-v1/CLOSEOUT_AUDIT_2026-05-04.md`

## Gates

```bash
cargo fmt --package fret-ui --package fret-ui-kit --package fret-imui --package fret-cookbook
cargo nextest run -p fret-ui text_area_tab_key_respects_allow_tab_input_policy --no-fail-fast
cargo nextest run -p fret-imui textarea_tab_key_does_not_insert_by_default textarea_allow_tab_input_inserts_tab_and_reports_changed --no-fail-fast
cargo nextest run -p fret-ui text_input text_area --no-fail-fast
cargo nextest run -p fret-imui models_text --no-fail-fast
cargo nextest run -p fret-cookbook cookbook_imui_example_keeps_current_facade_teaching_surface --no-fail-fast
cargo check -p fret-cookbook --features cookbook-imui --example imui_action_basics
python tools/check_layering.py
python -m json.tool docs/workstreams/imui-text-input-policy-depth-v1/WORKSTREAM.json
python tools/check_workstream_catalog.py
python .agents/skills/fret_skills.py validate --strict --check-anchors --check-symbols
git diff --check
```

If package-level filtering is too broad for an iteration, run the new test names directly first and
finish with the broader gates before closing the slice.
