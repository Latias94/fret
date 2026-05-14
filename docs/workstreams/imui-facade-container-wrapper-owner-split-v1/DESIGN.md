# ImUi Facade Container Wrapper Owner Split v1 - Design

Status: closed
Last updated: 2026-05-13

This lane is a narrow follow-on from `imui-facade-value-model-owner-split-v1`. It owns only the
private source owner for structural container `ImUiFacade` inherent wrappers.

## In Scope

- Move `horizontal(...)`, `menu_bar(...)`, `tab_bar(...)`, `vertical(...)`, `grid(...)`,
  `table(...)`, `virtual_list(...)`, `scroll(...)`, and `child_region(...)` wrapper pairs out of
  `facade_writer.rs`.
- Keep the methods as inherent `ImUiFacade` methods.
- Preserve public names, `fret::imui` re-export paths, `fret-imui`, and `crates/fret-ui` runtime
  contracts.

## Out Of Scope

- New container behavior, table flags, tab behavior, virtual-list behavior, or child-region flags.
- Moving or redesigning `UiWriterImUiFacadeExt`; this slice only moves inherent wrappers.
- Popup, floating-window, docking, multi-window, or runtime contract changes.
