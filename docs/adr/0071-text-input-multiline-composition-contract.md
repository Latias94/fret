---
title: "ADR 0071: Multiline Text Input + IME Composition Contract (Caret/Selection/Preedit)"
---

# ADR 0071: Multiline Text Input + IME Composition Contract (Caret/Selection/Preedit)

Status: Proposed

## Context

ADR 0012 defines the keyboard/IME event model and requires inline preedit rendering plus explicit
IME candidate window positioning (`set_ime_cursor_area`). ADR 0045/0046 define text geometry queries.

However, Fret does not yet lock the multiline text-editing contract for:

- selection and caret indexing semantics (byte vs grapheme vs codepoint),
- IME preedit insertion point and replacement rules in multiline buffers,
- caret rect reporting for IME candidate positioning in multiline layouts (wrapping + scrolling),
- the interaction between preedit, selection, and text editing commands.

These details are hard to retrofit once a code editor and a component ecosystem depend on them.

## Decision

### 1) Indexing model (P0)

Fret uses **byte indices into UTF-8 strings** for:

- caret position,
- selection ranges,
- IME preedit cursor/range metadata.

Rationale:

- aligns with existing core event payload shapes (ADR 0012),
- keeps the contract simple and deterministic across platforms,
- higher-level editors may expose grapheme-aware navigation as behavior on top of byte indices.

Invariants:

- All indices must be clamped to UTF-8 char boundaries before mutation.
- Any geometry query that accepts an index must treat it as a byte index.

### 2) Newline normalization

Multiline text buffers are normalized to `\n` line endings at the framework boundary:

- platform paste / clipboard `\r\n` must be normalized to `\n`,
- IME commit strings may contain newlines; they are inserted verbatim *after normalization*.

### 3) Preedit placement and commit semantics

While preedit is active:

- The preedit string is conceptually inserted at the caret position and rendered inline with a
  marked/underlined range.
- Editing commands that would mutate the buffer (insert/delete) must first consult the active
  preedit state:
  - commit replaces the current selection (if any) or inserts at caret,
  - when commit arrives and a selection exists, commit replaces the selection and clears it.
- Preedit updates (`ImeEvent::Preedit`) must not permanently mutate the underlying model buffer; the
  committed buffer only changes on commit or explicit non-IME edits.
- IME commit must clear the preedit state even if the platform does not emit a follow-up
  "empty preedit" update (prevents stale inline markup and incorrect IME cursor area reporting).

### 4) Caret rect reporting for IME candidate window positioning

Text editing widgets must be able to report a caret rect in window logical coordinates (ADR 0012).
For multiline widgets, this contract is extended:

- caret rect must reflect current scroll offset and wrapping layout,
- caret rect must update when any of the following changes:
  - caret/selection changes,
  - layout changes (wrap width, font metrics, scroll),
  - preedit text or preedit cursor changes,
  - DPI scale changes.

### 5) Interaction with keyboard shortcuts and focus traversal

IME-related arbitration is governed by ADR 0012 (preedit-first). For multiline widgets specifically:

- Tab may be part of IME candidate selection on some platforms and must not be consumed by focus traversal
  while composing.
- Enter may commit IME selection and must not be consumed as "submit" unless composing is inactive.

### 6) Semantics (accessibility) requirements (P0)

If accessibility is enabled (ADR 0033), multiline text fields must expose:

- current value (or a safe excerpt policy for very large buffers),
- selection range,
- IME composition range when preedit is active.

## Conformance checklist (P0)

- Windows Japanese IME: preedit shows inline in multiline input; Tab selects candidate; Enter commits; Escape cancels composition.
- Candidate window is positioned near caret (not at a fixed screen corner) while composing.
- Scrolling: caret rect and candidate positioning remain correct after scrolling within a multiline field.
- Selection replacement: selecting a range then committing IME replaces the selection and clears it.

## Non-Goals

- Full code editor architecture (buffer pieces, incremental layout, syntax highlighting).
- Grapheme-aware cursor movement rules (those are behavior policies above this contract).
- Rich bidi/cursor affinity semantics beyond ADR 0046.

## References

- `docs/adr/0012-keyboard-ime-and-text-input.md`
- `docs/adr/0044-text-editing-state-and-commands.md`
- `docs/adr/0045-text-geometry-queries-hit-testing-and-caret-metrics.md`
- `docs/adr/0046-multiline-text-layout-and-geometry-queries.md`
- `docs/adr/0033-semantics-tree-and-accessibility-bridge.md`
