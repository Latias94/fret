# ImUi Textarea Command Policy v1

Status: Closed
Last updated: 2026-05-06

## Problem

Dear ImGui's multiline `InputText` surface mixes text editing behavior, callback routing, and submit
keys inside the active input implementation. Fret should not copy that contract into
`crates/fret-ui`: the runtime textarea remains a mechanism surface, while IMUI owns app-facing
policy and command routing.

This lane closes the practical multiline submit/cancel gap by giving `fret-ui-kit::imui`
app-owned command hooks for focused text areas.

## Target

- Add opt-in submit/cancel command fields to `TextAreaOptions`.
- Default submit to Ctrl+Enter so unmodified Enter keeps inserting multiline text.
- Allow an explicit Enter-submit policy for form-like text areas.
- Dispatch Escape cancel only when unmodified.
- Ignore IME composition, Alt, Meta, and repeated keydown unless explicitly enabled.
- Use capture-phase key handling so Enter-submit policy wins before the runtime textarea inserts a
  newline.
- Keep `crates/fret-ui::TextAreaProps` unchanged.

## Ownership

- `ecosystem/fret-ui-kit/src/imui/options/controls.rs`: public IMUI textarea policy options.
- `ecosystem/fret-ui-kit/src/imui/text_controls.rs`: focused key arbitration and command dispatch.
- `ecosystem/fret-ui-kit/tests/imui_textarea_smoke.rs`: public options compile smoke.
- `ecosystem/fret-imui/src/tests/models_text_area.rs`: app-facing immediate-mode proof.

## Non-Goals

- Dear ImGui-style mutable buffer callbacks.
- Runtime-owned multiline undo/redo stacks.
- Ctrl+Enter newline insertion policy.
- Rich selection/range APIs.
- Platform IME redesign.
