# Evidence and Gates

Status: Closed
Last updated: 2026-05-06

## Smallest Repro

```bash
cargo nextest run -p fret-imui textarea_submit_and_cancel_commands_dispatch_from_focused_multiline_field textarea_enter_submit_policy_can_opt_into_enter_and_repeat textarea_enter_submit_policy_consumes_repeat_when_repeat_is_disabled --no-fail-fast
cargo nextest run -p fret-ui-kit --features imui textarea_command_policy_options_compile --no-fail-fast
```

## Gates

```bash
cargo fmt --package fret-ui-kit --package fret-imui
cargo nextest run -p fret-imui textarea_submit_and_cancel_commands_dispatch_from_focused_multiline_field textarea_enter_submit_policy_can_opt_into_enter_and_repeat textarea_enter_submit_policy_consumes_repeat_when_repeat_is_disabled --no-fail-fast
cargo nextest run -p fret-ui-kit --features imui textarea_command_policy_options_compile --no-fail-fast
cargo nextest run -p fret-ui-kit --features imui --no-fail-fast
cargo nextest run -p fret-imui --no-fail-fast
python tools/check_workstream_catalog.py
python tools/gate_imui_workstream_source.py
python .agents/skills/fret_skills.py validate --strict --check-anchors --check-symbols
python tools/check_layering.py
python -m json.tool docs/workstreams/imui-textarea-command-policy-v1/WORKSTREAM.json
git diff --check
```

## Known Gate Result

`cargo nextest run -p fret-imui --no-fail-fast` currently has one unrelated table/composition
failure:

- `tests::composition::table_helper_keeps_header_and_body_columns_aligned_and_clips_long_cells`
- observed drift: `status x header vs row0 drifted: left=335, right=17`

This lane does not modify table helpers or `composition.rs`.

## Evidence Anchors

- `ecosystem/fret-ui-kit/src/imui/options/controls.rs`
- `ecosystem/fret-ui-kit/src/imui/text_controls.rs`
- `ecosystem/fret-ui-kit/tests/imui_textarea_smoke.rs`
- `ecosystem/fret-imui/src/tests/models_text_area.rs`
- `docs/audits/imui-imgui-gap-audit-2026-04-22.md`

## Upstream Reference Anchors

- `repo-ref/imgui/imgui.h`
- `repo-ref/imgui/imgui_widgets.cpp`
