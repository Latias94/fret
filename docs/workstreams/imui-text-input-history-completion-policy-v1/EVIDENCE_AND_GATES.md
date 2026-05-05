# Evidence and Gates

Status: Closed
Last updated: 2026-05-04

## Smallest Repro

```bash
cargo nextest run -p fret-imui input_text_completion_command_dispatches_on_unmodified_tab --no-fail-fast
cargo nextest run -p fret-imui input_text_history_commands_dispatch_on_unmodified_arrows_without_default_repeat --no-fail-fast
```

## Gates

```bash
cargo fmt --package fret-ui-kit --package fret-imui
cargo check -p fret-ui-kit --tests
cargo nextest run -p fret-imui models_text --no-fail-fast
cargo check -p fret-cookbook --features cookbook-imui --example imui_action_basics
python tools/check_layering.py
python tools/check_workstream_catalog.py
python -m json.tool docs/workstreams/imui-text-input-history-completion-policy-v1/WORKSTREAM.json
git diff --check
```

## Evidence Anchors

- `ecosystem/fret-ui-kit/src/imui/options/controls.rs`
- `ecosystem/fret-ui-kit/src/imui/text_controls.rs`
- `ecosystem/fret-imui/src/tests/models_text.rs`
- `docs/audits/imui-imgui-gap-audit-2026-04-22.md`

## Upstream Reference Anchors

- `repo-ref/imgui/imgui.h`
- `repo-ref/imgui/imgui_widgets.cpp`
