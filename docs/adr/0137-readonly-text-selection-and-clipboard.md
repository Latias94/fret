# ADR 0137: Read-only Text Selection and Clipboard Commands

Status: Proposed

## Context

Fret already defines an editor-grade text command vocabulary (`text.*`, ADR 0044) and a text system boundary
(layout vs paint, ADR 0006). However, we also need a **read-only** text surface for:

- Markdown rendering (articles, docs, chat transcripts, streaming LLM output).
- Code preview blocks (syntax-highlighted but non-editable).
- Any UI that must support mouse selection + copy without becoming a full text editor.

If read-only selection is not provided as a first-class primitive, Markdown/code widgets tend to implement bespoke
hit-testing, selection state, and clipboard integration, which quickly diverges from editable widgets.

We also want to align with Zed’s approach: selection/clipboard is a reusable UX primitive, while full editor
features (multi-cursor, cross-buffer selection, syntax-object selection) are layered on top of a shared baseline.

## Decision

### 1) Add a `SelectableText` element as the baseline read-only surface

Introduce a declarative element that renders `RichText` and supports:

- focus on click,
- mouse drag selection (anchor + caret),
- selection highlight painting,
- clipboard copy of the selected UTF-8 slice,
- accessibility selection updates (`Event::SetTextSelection`).

This element is **not** a text input (no IME, no insertion, no editing). It only participates in focus + commands.

### 2) Extend `text.*` commands to cover selectable (not only editable) text surfaces

ADR 0044 reserves the `text.*` namespace for focused text-editable widgets. We amend the interpretation:

- `text.*` is reserved for **focused text surfaces**.
- Editable widgets support the full command set.
- Read-only `SelectableText` supports a strict subset:
  - `text.copy`
  - `text.select_all`

This keeps command routing consistent across editable and non-editable text, and matches user expectations
(Ctrl/Cmd+C should “just work” when a text selection exists).

### 3) Provide a minimal default keymap for standard text shortcuts

The default keymap service should include platform-appropriate bindings for:

- `text.copy`, `text.cut`, `text.paste`, `text.select_all`

Bindings should be safe to dispatch globally: if the focused element does not handle the command, nothing happens.
Apps can override/unbind via `KeymapService` (last-wins semantics).

## Implementation Notes (Current Workspace)

- Element: `SelectableTextProps/SelectableTextState`
  - `crates/fret-ui/src/element.rs`
- Declarative plumbing:
  - `crates/fret-ui/src/declarative/*`
- Event handling (mouse selection, focus, capture):
  - `crates/fret-ui/src/declarative/host_widget/event/selectable_text.rs`
- Copy/select-all command handling:
  - `crates/fret-ui/src/declarative/host_widget.rs`
- Default keymap bindings:
  - `crates/fret-app/src/app.rs`

This builds on existing text services:

- `prepare_rich` / `measure_rich`
- `hit_test_point`
- `selection_rects`

implemented by the renderer-backed text system (`crates/fret-render-wgpu/src/text/mod.rs`).

## Consequences

- Markdown/code preview can share the same selection/clipboard behavior as editable widgets.
- Command routing remains unified (`text.*`) while keeping editing-specific behavior in editable widgets.
- We deliberately do **not** support cross-element selection in this baseline (consistent with Zed’s non-cross-buffer
  selection model). Future ADRs can define a selection host for cross-block selection if needed.

## Future Work

### Phased Plan

**Phase A (selection UX parity)**

1) Keyboard navigation + selection commands for `SelectableText` (`text.move_*`, `text.select_*`).
2) Word/line boundary selection (double click, triple click) using Unicode segmentation rules.
3) Auto-scroll while selecting inside scroll containers.

**Phase B (rendering richness)**

4) Text decorations in `RichText` runs (underline/strikethrough) + consistent layout metrics for decoration placement.

**Phase C (editor-grade composition)**

5) Multi-selection (editor-grade) as a separate layer (not part of baseline `SelectableText`).
6) Rich inline fragments (inline widgets/images) inside text layout, similar to Zed’s `LineFragment` model, to enable:
   - more faithful Markdown inline composition,
   - tighter wrapping behavior around inline elements,
   - selection that can skip or include inline fragments consistently.

## References

- `docs/adr/0006-text-system.md`
- `docs/adr/0020-focus-and-command-routing.md`
- `docs/adr/0044-text-editing-state-and-commands.md`
- Zed GPUI line wrapping and layout: `repo-ref/zed/crates/gpui/src/text_system/`
