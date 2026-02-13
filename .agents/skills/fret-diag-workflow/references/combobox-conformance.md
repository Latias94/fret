---
title: Combobox conformance (shadcn recipe) - diag playbook
status: draft
date: 2026-02-13
scope: diagnostics, scripted-tests, combobox, overlays, ime, virtualization
---

# Combobox conformance (shadcn recipe) - diag playbook

Goal: keep Combobox regressions explainable and gateable with semantics-first scripts, not screenshots.

## Core invariants (portable)

- Open/close lifecycle:
  - trigger opens listbox,
  - Escape dismisses,
  - outside press dismisses.
- Focus restore:
  - close restores focus to trigger (shadcn-style expectation).
- Selection commit:
  - click/Enter commits and updates `selected` flag for the item,
  - disabled items do not commit.
- Overlay placement sanity:
  - listbox stays within window bounds,
  - placement decisions are explainable via `overlay_placement_trace`.
- Text + IME while open:
  - composing does not leak global shortcuts (Tab/Enter/Escape) unless intended,
  - IME cursor/caret geometry stays sane (in-window, non-empty when expected).
- Long-list / virtualization readiness:
  - selection persists even when items mount/unmount (virtualized lists),
  - scrolling to a deep item remains possible via semantics-first `scroll_into_view`.

## Reference suite (UI gallery)

Run:

- `cargo run -p fretboard -- diag suite ui-gallery-combobox --launch -- cargo run -p fret-ui-gallery --release`

Scripts worth copying from:

- Placement + focus restore:
  - `tools/diag-scripts/ui-gallery-combobox-open-select-focus-restore.json`
- IME routing + geometry:
  - `tools/diag-scripts/ui-gallery-combobox-ime-tab-suppressed.json`
    - gates `wait_shortcut_routing_trace outcome=reserved_for_ime` while composing
    - gates `ime_cursor_area_*` predicates for coarse caret/candidate geometry sanity
- Long list scroll + selection persistence (virtualization-ready invariant):
  - `tools/diag-scripts/ui-gallery-combobox-long-list-scroll-select-last.json`

## Authoring tips

- Prefer `test_id_prefix` on the component, and stable per-item `test_id` derived from item value (not index).
- Prefer `click_stable` for overlay content and scrolled targets.
- Prefer `scroll_into_view` for deep-item selection; do not hardcode pixel coordinates.
- Use `meta.required_capabilities`:
  - `diag.overlay_placement_trace` for placement gates
  - `diag.shortcut_routing_trace` for IME routing gates
  - `diag.text_input_snapshot` for IME caret geometry gates
