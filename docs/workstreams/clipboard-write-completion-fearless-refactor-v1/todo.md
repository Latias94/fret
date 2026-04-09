---
title: Clipboard Write Completion (Fearless Refactor v1) — TODO
status: draft
date: 2026-03-25
scope: clipboard, runtime contract, diagnostics, ai elements parity, refactor
---

# Clipboard Write Completion (Fearless Refactor v1) — TODO

Workstream entry:

- `docs/workstreams/clipboard-write-completion-fearless-refactor-v1/DESIGN.md`
- `docs/workstreams/clipboard-write-completion-fearless-refactor-v1/richer-clipboard-surfaces.md`

## Contract reset

- [x] Decide final public naming:
  - `ClipboardWriteText` / `ClipboardReadText`
  - or a narrower compromise if naming churn is rejected.
- [x] Add a structured clipboard error payload in `fret-core`.
- [x] Add `ClipboardWriteCompleted { token, outcome }` in `fret-core`.
- [x] Replace read failure string-only surfaces with the structured clipboard error payload if we
      choose the full fearless-reset path.
- [x] Update ADR 0041 and ADR 0266 to the final contract vocabulary.
- [x] Document the future payload lane explicitly in ADR/workstream terms:
  - `ClipboardWritePayload`
  - `ClipboardReadPayload`
  - `ClipboardPayload`
  - `ClipboardItem`

## Runtime + runner implementation

- [x] Add the new clipboard write effect in `crates/fret-runtime`.
- [x] Keep any compatibility adapter internal-only and delete it before closeout.
- [x] Desktop runner:
  - emit write success/failure completion events,
  - stop treating write result as diagnostics-only state.
- [x] Web runner:
  - map `navigator.clipboard.writeText(...)` Promise resolution/rejection to write completion
    events,
  - preserve user-activation-sensitive synchronous invocation timing.
- [x] Update clipboard diagnostics storage to record from the same real completion path.
- [x] Keep the text lane implementation independent from the future payload lane so P1 remains
      additive rather than blocking P0.

## `fret-ui` mechanism

- [x] Add clipboard completion action hook types in `crates/fret-ui/src/action.rs`.
- [x] Add `ElementContext` clipboard completion hook registration helpers.
- [x] Add focused tests for:
  - matching-token delivery,
  - non-matching-token isolation,
  - multiple pending clipboard writes in one window,
  - write failure not triggering success handlers.
- [x] Keep the hook surface clipboard-specific; do not add a generic window event listener.

## Caller migration

- [x] Migrate all explicit clipboard write callers away from `ClipboardSetText`.
- [x] AI copy surfaces:
  - [x] `StackTraceCopyButton`
  - [x] `CodeBlockCopyButton`
  - [x] `SnippetCopyButton`
  - [x] `CommitCopyButton`
  - [x] `TerminalCopyButton`
  - [x] `EnvironmentVariableCopyButton`
- [x] Add `on_error` builder/callback support where upstream semantics expect it.
- [x] Update copied/check UI state to flip only on write success.
- [x] Re-audit non-AI callers:
  - [x] `ecosystem/fret-code-view`
  - [x] `ecosystem/fret-router-ui`
  - [x] editor copy/cut paths
  - [x] diagnostics helpers / devtools
  - [x] list/table selection copy paths
- [x] Record the remaining ecosystem caller buckets in
      `docs/workstreams/clipboard-write-completion-fearless-refactor-v1/ecosystem-caller-audit.md`.
- [x] Identify first-party surfaces that may eventually want richer clipboard representations
      (`text/html`, image, MIME bytes) without implementing them yet.
  Evidence:
  `docs/workstreams/clipboard-write-completion-fearless-refactor-v1/richer-clipboard-surfaces.md`

## UI Gallery + teaching surface

- [x] Update AI Gallery notes/pages that currently describe `onError` as missing due to
      fire-and-forget clipboard writes.
- [x] Update prop tables so `on_copy` is described as success-only again.
- [x] Add at least one docs-aligned AI page that demonstrates `on_error`.
- [x] Audit snippet/page prose for "success hook" language that is currently optimistic.

## Diagnostics + regression gates

- [x] Update `docs/ui-diagnostics-and-scripted-tests.md` so
      `set_clipboard_force_unavailable` explicitly covers write failure simulation too.
- [x] Add dedicated script steps:
  - [x] `wait_clipboard_write_result`
  - [x] `assert_clipboard_write_result`
- [x] Add at least one diag script that forces clipboard write failure on a copy button and asserts:
  - copied state does not appear,
  - failure callback / UI seam fires,
  - diagnostics bundle captures the write failure.
- [x] Add unit tests for AI copy button success and failure semantics.
  Evidence:
  `ecosystem/fret-ui-ai/src/elements/clipboard_copy.rs`

## Deletion / closeout

- [x] Delete old `ClipboardSetText` / `ClipboardGetText` public names once all callers are migrated.
- [x] Delete transitional adapter code and migration comments.
- [x] Update `docs/adr/IMPLEMENTATION_ALIGNMENT.md` with final evidence anchors.
- [ ] Run layering and correctness gates before closeout:
  - `python3 tools/check_layering.py`
  - targeted `cargo nextest run ...`
  - relevant `fretboard-dev diag` scripts
