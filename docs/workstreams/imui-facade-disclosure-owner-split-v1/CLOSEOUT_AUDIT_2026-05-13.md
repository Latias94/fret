# ImUi Facade Disclosure Owner Split v1 - Closeout Audit - 2026-05-13

Status: closed
Last updated: 2026-05-13

## Outcome

The disclosure wrapper cluster is no longer owned by `facade_writer.rs`. It now lives in
`ecosystem/fret-ui-kit/src/imui/facade_writer/disclosure.rs`.

## Completion Checklist

| Requirement | Evidence |
| --- | --- |
| Move disclosure wrappers privately | `facade_writer/disclosure.rs`, `M1_DISCLOSURE_FACADE_OWNER_SPLIT_2026-05-13.md` |
| Preserve public method names and `fret::imui` paths | `cargo check -p fret-ui-kit --features imui`, `M1_DISCLOSURE_FACADE_OWNER_SPLIT_2026-05-13.md` |
| Keep `fret-imui` and runtime contracts unchanged | `DESIGN.md`, `M1_DISCLOSURE_FACADE_OWNER_SPLIT_2026-05-13.md` |
| Keep future work narrow | this closeout |

## Residual Scope

Text, boolean/model, table, docking, multi-window, and additive Dear ImGui component parity remain
separate lanes. Do not reopen this folder for those concerns.
