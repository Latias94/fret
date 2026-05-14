# ImUi Facade Disclosure Owner Split v1 - Design

Status: closed
Last updated: 2026-05-13

This lane is a narrow follow-on from `imui-kit-owner-split-v1`. It owns only the private source
owner for disclosure-adjacent `ImUiFacade` inherent wrappers.

## In Scope

- Move `collapsing_header(...)`, `collapsing_header_with_options(...)`, `tree_node(...)`, and
  `tree_node_with_options(...)` out of `facade_writer.rs`.
- Keep the methods as inherent `ImUiFacade` methods.
- Preserve public names, `fret::imui` re-export paths, `fret-imui`, and `crates/fret-ui` runtime
  contracts.

## Out Of Scope

- New disclosure behavior.
- New Dear ImGui widgets.
- Text, boolean/model, table, debug draw, docking, multi-window, or runtime contract changes.
