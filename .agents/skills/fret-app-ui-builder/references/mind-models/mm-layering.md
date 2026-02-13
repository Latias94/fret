# Mind model: Layering (contracts vs policy vs recipes)

Goal: keep the framework scalable by putting behavior in the right place.

## Rule of thumb

- Put **mechanisms/contracts** in `crates/fret-ui`:
  - element tree, layout, semantics/a11y, focus primitives, overlay roots/layers, outside-press observers.
- Put **interaction policy + headless state machines** in `ecosystem/fret-ui-kit`:
  - roving focus, typeahead, menu navigation, hover intent, focus scope/trap helpers, overlay policy queues.
- Put **shadcn taxonomy + composition** in `ecosystem/fret-ui-shadcn`:
  - recipe structs, variants, token-driven styling, and wiring to `fret-ui-kit` policies.

## Anti-patterns

- Don’t bake “dismiss on escape/outside press” directly into `fret-ui` widgets; prefer action hooks at the component layer.
- Don’t add renderer/platform dependencies into `fret-ui-shadcn`.

## When you’re unsure

Ask: “Is this behavior *hard to change* and *widely shared*?”

- If yes: it’s probably a `fret-ui` contract/mechanism (or needs an ADR).
- If it’s a component convention: it’s `fret-ui-kit`/`fret-ui-shadcn`.

## See also

- `fret-shadcn-source-alignment` (when matching upstream behavior)
