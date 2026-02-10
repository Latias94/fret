---
title: Diagnostics Extensibility + Capabilities v1 - Text & IME
status: draft
date: 2026-02-10
scope: diagnostics, text, ime, regression-gates
---

# Diagnostics Extensibility + Capabilities v1 - Text & IME

This document is a sub-part of `docs/workstreams/diag-extensibility-and-capabilities-v1.md`.

Self-drawn UI frameworks predictably struggle with text editing and IME composition.
The diagnostics contract must make these failures explainable and gateable without relying only on screenshots.

## What diagnostics must be able to explain

- caret “teleports” or becomes invisible,
- selection boundaries are wrong (word/line boundaries, double/triple click),
- IME composition breaks (preedit cancels, commit inserts wrong text, candidate window misplaced),
- global shortcuts steal non-printing keys while composing (Tab/Escape/Enter/arrows).

## Evidence surfaces (bundle/trace)

At minimum, expose a redaction-friendly summary for focused text inputs:

- focus target identity (`test_id` / node id),
- selection range (UTF-16, codepoint-aware),
- caret rect in window coordinates (for candidate placement),
- IME composition state:
  - composing? (bool),
  - composition range (UTF-16),
  - last composition update sequence id (monotonic),
  - last commit/cancel event kind (no raw content required).

The evidence must be stable enough to drive gates.

## Suggested regression gates (script + assertions)

Start with “portable, low-flake” gates:

- `word_boundary`:
  - type a sentence,
  - double-click selects a word,
  - assert selection range matches expected word boundary.
- `line_boundary`:
  - multi-line text,
  - triple-click selects a line,
  - assert selection range clamped to line.
- `caret_visible`:
  - after navigation keys, caret rect remains within window bounds.
- `composition_not_stolen_by_shortcuts`:
  - while composing, press keys that normally trigger global shortcuts,
  - assert the script did not trigger the shortcut path (requires trace + reason codes).

IME-specific behavior may remain runner-dependent; treat missing IME evidence as capability-gated
(`diag.text_ime_trace`) rather than as an implicit timeout.

## Redaction rules

Text content should be redacted by default in bundles and traces.

Allow opt-in for authoring/debugging (`FRET_DIAG_REDACT_TEXT=0`) and ensure gates can still operate on ranges,
not raw strings.

## References

- ADR 0012 (keyboard/IME/text input contract): `docs/adr/0012-keyboard-ime-and-text-input.md`
- Text editing commands: `docs/adr/0044-text-editing-state-and-commands.md`
- Caret metrics and geometry queries: `docs/adr/0045-text-geometry-queries-hit-testing-and-caret-metrics.md`

