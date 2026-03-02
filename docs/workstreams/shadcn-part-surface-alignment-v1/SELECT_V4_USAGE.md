# Select v4 usage mapping (shadcn/ui → fret-ui-shadcn)

This note documents **how to express the upstream shadcn/ui v4 `Select` docs snippets** using the
current `fret-ui-shadcn` surface.

Deeper structural convergence work is tracked in `docs/workstreams/select-combobox-deep-redesign-v1/`.

It is intentionally pragmatic: it aims to keep copy/paste workflows unblocked while `Select`
remains a **configuration + entries** API rather than a true nested part tree (`SelectTrigger`,
`SelectValue`, `SelectContent`, ... as real children).

## TL;DR

- Upstream composes a nested part tree.
- Fret currently composes via:
  - `Select::trigger(SelectTrigger { ... })` for trigger/value configuration, and
  - `Select::entries(Vec<SelectEntry>)` for content (items, groups, labels, separators).
- This is structurally different, but most docs examples translate 1:1.

## Upstream mental model (v4)

Upstream typically uses:

- `Select` root
- `SelectTrigger` + `SelectValue` for the collapsed button
- `SelectContent` holding:
  - `SelectGroup`
  - `SelectLabel`
  - `SelectItem`
  - `SelectSeparator`
  - (optional) scroll buttons

## Fret mapping

### Root state

In Fret, `Select` is driven by two models:

- `Model<Option<Arc<str>>>` (selected value)
- `Model<bool>` (open state)

Create them in your view state and pass them to:

- `Select::new(value, open)`

### Trigger + value (collapsed state)

Upstream:

- `SelectTrigger` → in Fret: `Select::trigger(SelectTrigger::new() ...)`
- `SelectValue` → in Fret: `SelectTrigger::value(SelectValue::new() ...)`

Placeholder maps to:

- `SelectValue::placeholder("...")`

### Content (items, groups, separators)

Upstream `SelectContent` is represented in Fret by `Select::entries(...)`.

The entries list is made of `SelectEntry`, typically built from:

- `SelectGroup::new([...])`
- `SelectLabel::new("...")`
- `SelectItem::new(value, label)`
- `SelectSeparator::new()`

### Optional: `into_element_parts(...)` adapter (nested authoring)

If you prefer a more shadcn-like "nested parts" call site, use:

- `Select::into_element_parts(cx, trigger, content)`

Where `content` can be built with `SelectContent::new().with_entries(...)`.

This still maps to the underlying configuration + entries implementation, but lets you keep the
call site closer to the upstream docs structure.

## Example (docs-shaped, Rust-shaped)

```rust
use fret_ui_shadcn::{self as shadcn, prelude::*};
use std::sync::Arc;

fn view<H: UiHost>(cx: &mut ElementContext<'_, H>) -> AnyElement {
    let value: Model<Option<Arc<str>>> = cx.app.models_mut().insert(None);
    let open: Model<bool> = cx.app.models_mut().insert(false);

    shadcn::Select::new(value, open).into_element_parts(
        cx,
        |_cx| {
            shadcn::SelectTrigger::new()
                .value(shadcn::SelectValue::new().placeholder("Select a fruit"))
        },
        |_cx| {
            shadcn::SelectContent::new().with_entries([shadcn::SelectGroup::new([
                shadcn::SelectLabel::new("Fruits").into(),
                shadcn::SelectItem::new("apple", "Apple").into(),
                shadcn::SelectItem::new("banana", "Banana").into(),
                shadcn::SelectSeparator::new().into(),
                shadcn::SelectItem::new("grape", "Grape").into(),
            ])
            .into()])
        },
    )
}
```

## Known gaps (why the mapping is not identical)

- `SelectTrigger` / `SelectValue` / `SelectContent` are **not literal nested elements** today.
  They are configuration/entry surfaces applied to a single `Select` recipe.
- Scroll buttons are represented, but the authoring shape may not match upstream exactly in all
  cases (especially when interacting with virtualization / available height constraints).

If you need strict copy/paste parity for the nested part tree, track Milestone 6 in
`docs/workstreams/select-combobox-deep-redesign-v1/` (Select v4 part surface convergence).
