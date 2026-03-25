# Ecosystem Clipboard Caller Audit

## Scope

This note classifies the remaining clipboard callers after the write-completion refactor so future
work does not re-open the same migration question.

## Conclusion

There are currently two distinct caller buckets:

1. Completion-driven copy surfaces.
   These surfaces expose visible success/failure UI state (`Copied`, timeout reset, `on_error`,
   etc.) and therefore must stay tokened and completion-driven.
2. Command-style clipboard actions.
   These surfaces issue copy/paste as a side effect of a command or automation flow, but they do
   not expose optimistic copied UI or a public error callback contract. They can remain
   fire-and-forget at the component layer while still using the runtime's tokened effect contract.

## Completion-driven surfaces

The following surfaces already require completion-driven semantics and have been migrated:

- `ecosystem/fret-ui-ai/src/elements/code_block.rs`
- `ecosystem/fret-ui-ai/src/elements/commit.rs`
- `ecosystem/fret-ui-ai/src/elements/environment_variables.rs`
- `ecosystem/fret-ui-ai/src/elements/snippet.rs`
- `ecosystem/fret-ui-ai/src/elements/stack_trace.rs`
- `ecosystem/fret-ui-ai/src/elements/terminal.rs`
- `ecosystem/fret-code-view/src/copy_button.rs`

These are the only current ecosystem surfaces that expose copied state, timer reset behavior, or
structured copy failure hooks.

## Command-style clipboard actions

The following callers do not currently need a higher-level migration because they expose no
optimistic success state and no public copy failure callback:

- `ecosystem/fret-ui-kit/src/declarative/list.rs`
- `ecosystem/fret-ui-kit/src/declarative/table.rs`
- `ecosystem/fret-router-ui/src/lib.rs`
- `ecosystem/fret-genui-core/src/executor.rs`
- `ecosystem/fret-node/src/ui/canvas/widget/clipboard_transfer.rs`
- `apps/fret-ui-gallery/src/ui/doc_layout.rs`
- `apps/fret-devtools/src/native.rs`
- `ecosystem/fret-bootstrap/src/ui_diagnostics/inspect_overlay.rs`
- `ecosystem/fret-bootstrap/src/ui_diagnostics/inspect_controller.rs`
- `ecosystem/fret-bootstrap/src/ui_diagnostics/script_steps.rs`
- `ecosystem/fret-bootstrap/src/ui_diagnostics/script_steps_input.rs`
- `ecosystem/fret-bootstrap/src/ui_app_driver.rs`

These callers should keep using `Effect::ClipboardWriteText` / `Effect::ClipboardReadText` and the
runtime token contract, but they do not need extra component-level copied/error plumbing today.

## Read-path consumers that depend on routing

Two ecosystem surfaces already depend on tokened clipboard read completion routing:

- `ecosystem/fret-code-editor/src/editor/mod.rs`
- `ecosystem/fret-node/src/ui/canvas/widget/event_router_system_lifecycle.rs`

These surfaces do not need a caller migration, but they do require `fret-ui` to route
`ClipboardReadText` / `ClipboardReadFailed` window events into the declarative or widget subtree.

## Migration trigger rule

Any current command-style caller must move into the completion-driven bucket if it adds one of the
following:

- visible copied/checkmark state,
- timeout-based reset state,
- user-facing copy failure UI,
- public `on_copy` / `on_error` semantics that imply real completion.
