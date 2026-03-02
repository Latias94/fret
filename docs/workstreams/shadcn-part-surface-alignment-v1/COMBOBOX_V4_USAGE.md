# Combobox v4 Usage (Rust parity notes)

This note documents how to express the shadcn/ui v4 Combobox docs shapes in Fret using the
`into_element_parts(...)` adapters.

Sources of truth:

- Upstream docs: `repo-ref/ui/apps/v4/content/docs/components/radix/combobox.mdx`
- Upstream base parts: `repo-ref/ui/apps/v4/registry/bases/radix/ui/combobox.tsx`

## Design intent

- Keep `ecosystem/fret-ui-shadcn` as a *taxonomy + recipe* layer.
- Provide v4-named *part surfaces* as thin adapters over existing recipes where possible.
- Prefer deterministic gates (unit tests + diag scripts) over pixel-perfect parity.

## Basic (docs “Usage” shape)

Upstream (simplified):

- `Combobox` root owns `items`
- `ComboboxInput` configures placeholder (and optionally clear/trigger affordances)
- `ComboboxContent` owns the list surface

Fret (Rust) equivalent:

```rust
use std::sync::Arc;

use fret_ui_shadcn::{
    Combobox, ComboboxContent, ComboboxContentPart, ComboboxEmpty, ComboboxInput, ComboboxItem,
    ComboboxList, ComboboxPart,
};
use fret_ui::{AnyElement, ElementContext};
use fret_app::App;
use fret_runtime::Model;

pub fn example_combobox(cx: &mut ElementContext<'_, App>) -> AnyElement {
    let value: Model<Option<Arc<str>>> = cx.app.models_mut().insert(None);
    let open: Model<bool> = cx.app.models_mut().insert(false);

    let items = [
        ComboboxItem::new("next", "Next.js"),
        ComboboxItem::new("svelte", "SvelteKit"),
        ComboboxItem::new("nuxt", "Nuxt.js"),
        ComboboxItem::new("remix", "Remix"),
        ComboboxItem::new("astro", "Astro"),
    ];

    Combobox::new(value, open)
        .a11y_label("Select a framework")
        .into_element_parts(cx, |_cx| {
            vec![
                ComboboxPart::from(ComboboxInput::new().placeholder("Select a framework")),
                ComboboxPart::from(ComboboxContent::new([
                    ComboboxContentPart::from(ComboboxEmpty::new("No items found.")),
                    ComboboxContentPart::from(ComboboxList::new().items(items)),
                ])),
            ]
        })
}
```

Notes:

- Upstream uses render props (`(item) => ...`) to map `items → rows`. Rust cannot express that 1:1,
  so the adapter accepts explicit `ComboboxList::items(...)` / `ComboboxList::groups(...)`.
- If `ComboboxList` provides items/groups, those become the recipe’s source of truth for options.

## Clear button

Upstream uses `showClear` on `ComboboxInput`. Fret maps it to the existing recipe clear affordance:

```rust
ComboboxInput::new()
    .placeholder("Select a framework")
    .show_clear(true)
```

## Anchor override (`useComboboxAnchor`)

Upstream v4 uses `useComboboxAnchor()` (a DOM ref) and passes it to `ComboboxContent` as
`anchor={anchor}` (Base UI `Positioner.anchor`) to control popup positioning.

In Fret, we model the same outcome via a layout-only wrapper (`useComboboxAnchor(child)`) that
exposes a stable element ID. Pass the ID to `ComboboxContent::anchor_element_id(...)`:

```rust
use fret_ui_shadcn::{ComboboxContent, useComboboxAnchor};

let anchor = useComboboxAnchor(cx.text("anchor"));
let anchor_id = anchor.element_id();
let _anchor_el = anchor.into_element(cx);

let content = ComboboxContent::new([]).anchor_element_id(anchor_id);
```

## Popup trigger (docs “Popup” example)

Upstream “Popup” moves the input inside `ComboboxContent` and uses `ComboboxTrigger(render=Button)`
with `ComboboxValue` as its child.

In Fret:

- Use `ComboboxTrigger` to map to recipe-level trigger knobs (`trigger_variant`, `width`).
- If `ComboboxInput` is placed inside `ComboboxContent`, its `placeholder(...)` maps to the overlay
  search input placeholder (`search_placeholder`), not the trigger placeholder.

```rust
use fret_ui_shadcn::{
    Combobox, ComboboxContent, ComboboxContentPart, ComboboxEmpty, ComboboxInput, ComboboxItem,
    ComboboxList, ComboboxPart, ComboboxTrigger, ComboboxTriggerVariant,
};
use fret_core::Px;

Combobox::new(value, open).into_element_parts(cx, |_cx| {
    vec![
        ComboboxPart::from(
            ComboboxTrigger::new()
                .variant(ComboboxTriggerVariant::Button)
                .width_px(Px(256.0)),
        ),
        ComboboxPart::from(ComboboxContent::new([
            ComboboxContentPart::from(ComboboxInput::new().placeholder("Search")),
            ComboboxContentPart::from(ComboboxEmpty::new("No items found.")),
            ComboboxContentPart::from(ComboboxList::new().items([
                ComboboxItem::new("us", "United States"),
                ComboboxItem::new("cn", "China"),
            ])),
        ])),
    ]
})
```

## Groups (+ separator)

Upstream uses `ComboboxGroup`, `ComboboxLabel`, `ComboboxCollection`, and an optional
`ComboboxSeparator`.

Fret supports an equivalent shape via `ComboboxList::groups(...)` + `ComboboxGroup::separator(true)`:

```rust
use fret_ui_shadcn::{
    ComboboxContent, ComboboxContentPart, ComboboxEmpty, ComboboxGroup, ComboboxInput, ComboboxItem,
    ComboboxLabel, ComboboxList,
};

let list = ComboboxList::new().groups([
    ComboboxGroup::new()
        .label(ComboboxLabel::new("Europe"))
        .items([
            ComboboxItem::new("fr", "France"),
            ComboboxItem::new("de", "Germany"),
        ])
        .separator(true),
    ComboboxGroup::new()
        .label(ComboboxLabel::new("Asia"))
        .items([
            ComboboxItem::new("cn", "China"),
            ComboboxItem::new("jp", "Japan"),
        ]),
]);

let content = ComboboxContent::new([
    ComboboxContentPart::from(ComboboxEmpty::new("No results.")),
    ComboboxContentPart::from(list),
]);
```

## Multiple selection (chips)

Upstream uses a single `Combobox` root with `multiple`, `ComboboxChips`, `ComboboxValue`,
`ComboboxChip`, and `ComboboxChipsInput`.

In Fret, multi-select is currently modeled as a dedicated recipe: `ComboboxChips`. The part adapter
is available as `ComboboxChips::into_element_parts(...)` and supports:

- `ComboboxChipsInput::placeholder(...)` → mapped to the overlay search input placeholder.
- `ComboboxChip::show_remove(false)` → disables the recipe’s chip remove affordance.
- `ComboboxContent(Empty/List/Group/Item...)` → overrides `empty_text` and (optionally) items/groups.

Example:

```rust
use std::sync::Arc;

use fret_ui_shadcn::{
    ComboboxChip, ComboboxChips, ComboboxChipsInput, ComboboxChipsPart, ComboboxContent,
    ComboboxContentPart, ComboboxEmpty, ComboboxItem, ComboboxList, ComboboxValue,
};
use fret_ui::{AnyElement, ElementContext};
use fret_app::App;
use fret_runtime::Model;

pub fn example_combobox_multiple(cx: &mut ElementContext<'_, App>) -> AnyElement {
    let values: Model<Vec<Arc<str>>> = cx.app.models_mut().insert(Vec::new());
    let open: Model<bool> = cx.app.models_mut().insert(false);

    ComboboxChips::new(values, open).into_element_parts(cx, |_cx| {
        vec![
            ComboboxChipsPart::from(
                ComboboxValue::new([ComboboxChip::new("next").show_remove(true)]),
            ),
            ComboboxChipsPart::from(ComboboxChipsInput::new().placeholder("Add framework")),
            ComboboxChipsPart::from(ComboboxContent::new([
                ComboboxContentPart::from(ComboboxEmpty::new("No items found.")),
                ComboboxContentPart::from(ComboboxList::new().items([
                    ComboboxItem::new("next", "Next.js"),
                    ComboboxItem::new("svelte", "SvelteKit"),
                ])),
            ])),
        ]
    })
}
```

## Known drift (explicitly accepted for now)

- Base UI’s in-trigger editable chips input is not represented as a literal nested element in Fret;
  the filter input lives in the overlay surface.
- Base UI’s `ComboboxInput` inside `ComboboxContent` is treated as configuration for the overlay
  search field (placeholder), not a literal nested input element.
- Render-prop ergonomics are not modeled; lists are provided explicitly.
- `ComboboxInput.show_trigger(false)` hides the default trigger icon (the trigger control still
  toggles the overlay).
- Base UI anchor refs (`useComboboxAnchor()`) are modeled via `useComboboxAnchor(child)` +
  `ComboboxContent::anchor_element_id(...)` (element ID), not a DOM ref.
