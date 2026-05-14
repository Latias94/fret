# M1 Button / Action Facade Slice - 2026-05-13

Status: button/action facade owner split landed

## What Changed

Moved the `ImUiFacade` inherent button/action/command-button wrappers out of
`ecosystem/fret-ui-kit/src/imui/facade_writer.rs` into:

- `ecosystem/fret-ui-kit/src/imui/facade_writer/button_actions.rs`

The moved cluster includes:

- `button`
- `small_button`
- `small_button_with_options`
- `arrow_button`
- `arrow_button_with_options`
- `invisible_button`
- `invisible_button_with_options`
- `action_button`
- `action_button_with_options`
- `action_payload_button`
- `action_payload_button_with_options`
- `button_command`
- `button_command_with_options`

These methods remain inherent methods on `ImUiFacade`; only their private source owner changed.

## Preserved Invariants

- No public method names changed.
- No `fret::imui` re-export path changed.
- No `fret-imui` dependency or public surface changed.
- No `crates/fret-ui` runtime contract changed.
- Focus recording behavior remains in the same wrapper methods via `record_focusable(...)`.
- The trait implementation for `UiWriterImUiFacadeExt` remains unchanged.

## Source Size Delta

| File | Before M1 | After M1 |
| --- | ---: | ---: |
| `ecosystem/fret-ui-kit/src/imui/facade_writer.rs` | 1801 lines | 1687 lines |
| `ecosystem/fret-ui-kit/src/imui/facade_writer/button_actions.rs` | n/a | 118 lines |

## Verification

Passed locally:

```bash
cargo check -p fret-ui-kit --features imui
cargo nextest run -p fret-ui-kit --features imui --test imui_adapter_seam_smoke --test imui_response_contract_smoke --no-fail-fast
```

Additional source-policy and formatting gates remain listed in `EVIDENCE_AND_GATES.md` and should
be rerun before closing or committing the lane.

## Next

The next owner split should stay small. Recommended candidates:

1. move another focused inherent-wrapper cluster out of `facade_writer.rs`, such as menu item
   wrappers or selectable/combo focusable wrappers;
2. audit response/status assembly before introducing any private typed helper;
3. avoid additive widget work until this structural lane is closed or deliberately split again.
