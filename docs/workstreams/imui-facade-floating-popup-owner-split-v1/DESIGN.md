# ImUi Facade Floating Popup Owner Split v1 - Design

Status: closed
Last updated: 2026-05-14

This lane is a narrow follow-on from `imui-facade-container-wrapper-owner-split-v1`. It owns only
the private source owner for floating, popup, tooltip, drag/drop, and in-window floating-window
`UiWriterImUiFacadeExt` default implementation bodies.

## In Scope

- Move the implementation bodies for floating area/layer, popup, tooltip, drag/drop, context-menu,
  and `window(...)` trait default methods out of `facade_writer.rs`.
- Keep `UiWriterImUiFacadeExt` as the public trait hub with the same method names and signatures.
- Preserve public names, `fret::imui` re-export paths, `fret-imui`, and `crates/fret-ui` runtime
  contracts.

## Out Of Scope

- New popup, tooltip, drag/drop, or floating-window behavior.
- Splitting the public trait into public subtraits.
- New model/state ownership.
- Docking, multi-window, or runtime contract changes.
