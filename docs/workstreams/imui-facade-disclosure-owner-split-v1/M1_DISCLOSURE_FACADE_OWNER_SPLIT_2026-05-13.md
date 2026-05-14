# ImUi Facade Disclosure Owner Split v1 - M1 Disclosure Facade Owner Split

Status: disclosure facade owner split landed
Date: 2026-05-13

## Result

- Moved disclosure inherent facade wrappers into
  `ecosystem/fret-ui-kit/src/imui/facade_writer/disclosure.rs`.
- These methods remain inherent methods on `ImUiFacade`; only their private source owner changed.
- No public method names changed.
- No `fret::imui` re-export path changed.
- No `fret-imui` dependency or public surface changed.
- No `crates/fret-ui` runtime contract changed.
- `ecosystem/fret-ui-kit/src/imui/facade_writer.rs` | 1506 lines | 1464 lines
- `ecosystem/fret-ui-kit/src/imui/facade_writer/disclosure.rs` | n/a | 46 lines

## Gates

- `cargo fmt --package fret-ui-kit -- --check`
- `cargo check -p fret-ui-kit --features imui`
- `cargo nextest run -p fret-ui-kit --features imui --test imui_adapter_seam_smoke --test imui_response_contract_smoke --no-fail-fast`
