# ImUi Selectable Highlight Policy v1

Status: closed execution follow-on
Last updated: 2026-05-16

Status note (2026-05-16): this lane closed after the selectable highlight policy landed. Keep
future selectable/list behavior growth in new proof-led follow-ons instead of widening this folder.

## Scope

This lane owns the narrow policy equivalent of Dear ImGui's
`ImGuiSelectableFlags_Highlight`.

The target is intentionally small:

- add an opt-in highlighted visual state to `SelectableOptions`,
- keep highlighted separate from selected semantics and accessibility state,
- keep disabled selectables muted even when highlighted is requested,
- let `multi_selectable_with_options(...)` inherit the same policy through the shared options type,
- fix the input-text picker recipe so its keyboard-active candidate is highlighted, not selected,
- keep `fret-imui` thin and unchanged.

## Assumptions

- Confident: this belongs in `fret-ui-kit::imui`, not `fret-imui`.
  Evidence: `SelectableOptions` already owns selectable policy and `docs/adr/0066-fret-ui-runtime-contract-surface.md`
  keeps runtime contract growth out of policy helpers.
  Consequence if wrong: revert the options-field addition and move the behavior to a recipe-only
  adapter, not to the runtime.
- Confident: highlighted and selected must stay separate.
  Evidence: Dear ImGui exposes `SelectableFlags_Highlight` as "displayed as if hovered", while
  Fret selectable semantics already use `PressableA11y::selected`.
  Consequence if wrong: keyboard-active popup rows would report false selected state to callers and
  accessibility consumers.
- Likely: the input-text picker is the first in-tree proof pressure.
  Evidence: its keyboard navigation already tracks an active candidate, and the old implementation
  used `selected: checked || active`.
  Consequence if wrong: the public option still remains a small, backward-compatible policy axis,
  but future proof should decide whether recipe-local styling is enough.

## Target API

`SelectableOptions` gains:

```rust
pub highlighted: bool
```

Default remains `false`.

When `highlighted` is true, enabled unselected rows use the same background/foreground palette as
hovered rows. The row is not selected, does not report selected accessibility semantics, and does
not change click or focus response behavior.

Selected rows continue to use selected styling even if highlighted is also true. Disabled rows stay
muted and do not gain a hover-style background from a forced highlight.

## Non-Goals

- No `ImGuiSelectableFlags_*` enum mirror.
- No `SpanAllColumns`, `AllowOverlap`, or `SelectOnNav` in this slice.
- No list-box helper.
- No collection request/IO helper.
- No `fret-imui` dependency or runtime contract change.
