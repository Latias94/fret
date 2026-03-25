---
title: Clipboard Write Completion (Fearless Refactor v1) — Richer Clipboard Surfaces
status: draft
date: 2026-03-25
scope: clipboard, payloads, portability, ecosystem
---

# Richer Clipboard Surfaces

## Purpose

This note identifies the first-party surfaces that may eventually want richer clipboard
representations beyond the shipped text lane.

It is intentionally non-normative for v1. The current contract remains text-first:

- `Effect::ClipboardWriteText { window, token, text }`
- `Effect::ClipboardReadText { window, token }`
- `Event::ClipboardWriteCompleted { token, outcome }`
- `Event::ClipboardReadText { token, text }`
- `Event::ClipboardReadFailed { token, error }`

The goal here is to prevent future guesswork about where a richer payload lane would actually pay
for itself.

## Decision summary

For the current refactor, the repo should keep shipping the text-first clipboard lane only.

If richer clipboard support is added later, it should be driven by concrete first-party product
surfaces rather than by abstract capability parity. In particular:

1. text-only copy/paste remains the default for ordinary text editing, docs, diagnostics, and copy
   buttons,
2. richer payloads should be additive rather than a replacement for the text lane,
3. `text/plain` fallback should remain available whenever a richer payload is intended to interop
   outside Fret,
4. file-like clipboard payloads must follow the same handle/token portability rules already used by
   external drop and mobile file dialogs.

## Surface buckets

### A. Surfaces that should remain text-first

These surfaces do not currently justify widening the clipboard contract beyond text:

- `crates/fret-ui/src/text/input/widget.rs`
- `crates/fret-ui/src/text/area/widget.rs`
- `ecosystem/fret-ui-ai/src/elements/code_block.rs`
- `ecosystem/fret-ui-ai/src/elements/commit.rs`
- `ecosystem/fret-ui-ai/src/elements/environment_variables.rs`
- `ecosystem/fret-ui-ai/src/elements/snippet.rs`
- `ecosystem/fret-ui-ai/src/elements/stack_trace.rs`
- `ecosystem/fret-ui-ai/src/elements/terminal.rs`
- `ecosystem/fret-code-view/src/copy_button.rs`
- `apps/fret-ui-gallery/src/ui/doc_layout.rs`
- `ecosystem/fret-ui-kit/src/declarative/list.rs`
- `ecosystem/fret-ui-kit/src/declarative/table.rs`
- `ecosystem/fret-router-ui/src/lib.rs`
- `ecosystem/fret-bootstrap/src/ui_diagnostics/inspect_controller.rs`
- `ecosystem/fret-bootstrap/src/ui_diagnostics/script_steps.rs`
- `apps/fret-devtools/src/native.rs`

Rationale:

- they move plain text only,
- they do not need cross-app rich interop to be useful,
- making them the first richer-clipboard adopters would add contract surface without product
  urgency.

This bucket includes most current AI copy buttons. Their main requirement was honest completion
semantics, not richer payload types.

### B. Strong candidate: structured graph copy/paste

The clearest future richer-payload candidate is:

- `ecosystem/fret-node/src/ui/canvas/widget/clipboard_transfer.rs`

Today this surface serializes a graph fragment through `to_clipboard_text()` and pastes from the
text lane. That is a good portable baseline, but the long-term product shape is likely richer:

- a Fret-specific structured MIME payload for lossless graph transfer,
- optional bounded byte payloads for larger graph fragments if text becomes too limiting,
- `text/plain` fallback for cross-app interoperability and debugging.

This should be the first serious candidate for:

- `Effect::ClipboardWritePayload { window, token, payload }`
- `Effect::ClipboardReadPayload { window, token, formats, limits }`

If this lane lands, the node graph surface should remain the policy owner for paste arbitration
while `fret-core` / `fret-runtime` only define the portable typed payload contract.

### C. Strong candidate: clipboard images and file-like attachments

The clearest future richer read surface is:

- `ecosystem/fret-ui-ai/src/elements/prompt_input.rs`

That surface already exposes attachment-oriented affordances (`on_add_attachments`,
`on_add_screenshot`) and explicitly documents that clipboard files remain app-owned. This is a
strong signal that future AI/chat composition flows may want:

- image paste from the clipboard,
- file-like clipboard references,
- screenshot paste,
- bounded MIME-typed bytes for attachments that are not naturally text.

This lane must not place raw paths or platform URIs in `fret-core`. If clipboard files or images
are supported later, they should align with ADR 0264-style file-handle semantics and with the
bounded-bytes direction already described in ADR 0266 D5.

### D. Medium candidate: rich code copy (`text/html`)

The next likely candidate after node graph and attachment paste is rich code export/copy for:

- `ecosystem/fret-code-editor/src/editor/input/mod.rs`
- `ecosystem/fret-code-view/src/copy_button.rs`
- `ecosystem/fret-ui-ai/src/elements/code_block.rs`
- `ecosystem/fret-ui-ai/src/elements/snippet.rs`

Possible future value:

- preserving syntax colors when pasting into rich-text destinations,
- providing an explicit "Copy as rich text" action,
- optionally pairing `text/html` with the existing plain-text copy.

This should not replace ordinary `Copy`. Default editor copy should stay plain text unless product
requirements clearly demand richer external formatting. Rich code copy is a secondary export-like
action, not a reason to complicate baseline clipboard semantics for all text surfaces.

### E. Surfaces that should not drive the richer contract

The following surfaces should stay behind richer-payload priorities:

- `apps/fret-ui-gallery/src/ui/doc_layout.rs`
- `ecosystem/fret-bootstrap/src/ui_diagnostics/*`
- `apps/fret-devtools/src/native.rs`
- command-style copy helpers in list/table/router flows

Reason:

- they are primarily teaching, diagnostics, or command helpers,
- their value comes from predictable text output,
- letting them define the richer contract would bias the design toward the wrong use cases.

## Recommended priority order

If the repo decides to add richer clipboard support after the v1 refactor, the safest order is:

1. node-graph structured copy/paste,
2. prompt/attachment clipboard images and file-like reads,
3. explicit code-oriented rich-text copy (`text/html`),
4. everything else only if a real first-party surface still remains blocked.

This order keeps the richer lane tied to real editor-grade or AI-composition value instead of to
docs polish.

## Guardrails for the future payload lane

When the richer lane is implemented, keep these constraints:

1. Do not weaken the shipped text lane. `ClipboardWriteText` / `ClipboardReadText` should remain
   the default path for simple copy/paste.
2. Do not leak backend-native clipboard objects into contract crates.
3. Prefer portable typed payloads over opaque blobs. If byte payloads are needed, keep them
   bounded and MIME-described, similar in spirit to `ShareItem::Bytes`.
4. File-like clipboard payloads must use token/handle semantics instead of raw paths/URIs.
5. Keep mechanism and policy separated:
   - `crates/*` own payload transport and completion semantics,
   - `ecosystem/*` owns paste arbitration, default formats, fallback order, and UI policy.

## Closeout statement

The current clipboard refactor is complete without implementing richer payloads.

This note exists so future work can start from a concrete first-party demand map rather than
re-opening whether the v1 text-first contract was sufficient. The answer is:

- yes for current product needs,
- with clear follow-up candidates already identified for the next lane.
