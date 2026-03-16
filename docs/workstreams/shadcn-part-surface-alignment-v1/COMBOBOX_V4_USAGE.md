# Combobox v4 Usage (Rust parity notes)

This note documents how to express the shadcn/ui v4 Combobox docs shapes in Fret using the
`into_element_parts(...)` adapters.

Deeper structural convergence work is tracked in `docs/workstreams/select-combobox-deep-redesign-v1/`.

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

use fret_app::App;
use fret_ui::{AnyElement, ElementContext};
use fret_ui_shadcn::facade as shadcn;
use fret_runtime::Model;

pub fn example_combobox(cx: &mut ElementContext<'_, App>) -> AnyElement {
    let value: Model<Option<Arc<str>>> = cx.app.models_mut().insert(None);
    let open: Model<bool> = cx.app.models_mut().insert(false);

    let items = [
        shadcn::ComboboxItem::new("next", "Next.js"),
        shadcn::ComboboxItem::new("svelte", "SvelteKit"),
        shadcn::ComboboxItem::new("nuxt", "Nuxt.js"),
        shadcn::ComboboxItem::new("remix", "Remix"),
        shadcn::ComboboxItem::new("astro", "Astro"),
    ];

    shadcn::Combobox::new(value, open)
        .a11y_label("Select a framework")
        .into_element_parts(cx, |_cx| {
            vec![
                shadcn::ComboboxPart::from(
                    shadcn::ComboboxInput::new().placeholder("Select a framework"),
                ),
                shadcn::ComboboxPart::from(shadcn::ComboboxContent::new([
                    shadcn::ComboboxContentPart::from(
                        shadcn::ComboboxEmpty::new("No items found."),
                    ),
                    shadcn::ComboboxContentPart::from(shadcn::ComboboxList::new().items(items)),
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
shadcn::ComboboxInput::new()
    .placeholder("Select a framework")
    .show_clear(true)
```

## Anchor override (`PopoverAnchor::build(...).into_anchor(cx)`)

Upstream v4 uses `useComboboxAnchor()` (a DOM ref) and passes it to `ComboboxContent` as
`anchor={anchor}` (Base UI `Positioner.anchor`) to control popup positioning.

In Fret, use the generic overlay anchor builder and element ID surface instead of a
combobox-specific helper. Pass the resulting ID to `ComboboxContent::anchor_element_id(...)`:

```rust
use fret_ui_kit::ui;
use fret_ui_shadcn::facade as shadcn;

let anchor = shadcn::PopoverAnchor::build(ui::label("anchor")).into_anchor(cx);
let anchor_id = anchor.element_id();
let _anchor_el = anchor.into_element(cx);

let content = shadcn::ComboboxContent::new([]).anchor_element_id(anchor_id);
```

## Popup trigger (docs “Popup” example)

Upstream “Popup” moves the input inside `ComboboxContent` and uses `ComboboxTrigger(render=Button)`
with `ComboboxValue` as its child.

In Fret:

- Use `ComboboxTrigger` to map to recipe-level trigger knobs (`trigger_variant`, `width`).
- If `ComboboxInput` is placed inside `ComboboxContent`, its `placeholder(...)` maps to the overlay
  search input placeholder (`search_placeholder`), not the trigger placeholder.

```rust
use fret_core::Px;
use fret_ui_shadcn::facade as shadcn;

shadcn::Combobox::new(value, open).into_element_parts(cx, |_cx| {
    vec![
        shadcn::ComboboxPart::from(
            shadcn::ComboboxTrigger::new()
                .variant(shadcn::ComboboxTriggerVariant::Button)
                .width_px(Px(256.0)),
        ),
        shadcn::ComboboxPart::from(shadcn::ComboboxContent::new([
            shadcn::ComboboxContentPart::from(shadcn::ComboboxInput::new().placeholder("Search")),
            shadcn::ComboboxContentPart::from(shadcn::ComboboxEmpty::new("No items found.")),
            shadcn::ComboboxContentPart::from(shadcn::ComboboxList::new().items([
                shadcn::ComboboxItem::new("us", "United States"),
                shadcn::ComboboxItem::new("cn", "China"),
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
use fret_ui_shadcn::facade as shadcn;

let list = shadcn::ComboboxList::new().groups([
    shadcn::ComboboxGroup::new()
        .label(shadcn::ComboboxLabel::new("Europe"))
        .items([
            shadcn::ComboboxItem::new("fr", "France"),
            shadcn::ComboboxItem::new("de", "Germany"),
        ])
        .separator(true),
    shadcn::ComboboxGroup::new()
        .label(shadcn::ComboboxLabel::new("Asia"))
        .items([
            shadcn::ComboboxItem::new("cn", "China"),
            shadcn::ComboboxItem::new("jp", "Japan"),
        ]),
]);

let content = shadcn::ComboboxContent::new([
    shadcn::ComboboxContentPart::from(shadcn::ComboboxEmpty::new("No results.")),
    shadcn::ComboboxContentPart::from(list),
]);
```

## Multiple selection (chips)

Upstream uses a single `Combobox` root with `multiple`, `ComboboxChips`, `ComboboxValue`,
`ComboboxChip`, and `ComboboxChipsInput`.

In Fret, multi-select is currently modeled as a dedicated recipe: `ComboboxChips`. The part adapter
is available as `ComboboxChips::into_element_parts(...)` and supports:

- `ComboboxChipsInput::placeholder(...)` → mapped to both the trigger placeholder (when no chips
  are selected) and the overlay search input placeholder.
- `ComboboxTrigger::width_px(...)` → mapped to the recipe width override.
- `ComboboxChip::show_remove(false)` → disables the recipe’s chip remove affordance.
- `ComboboxContent(Empty/List/Group/Item...)` → overrides `empty_text` and (optionally) items/groups.

Example:

```rust
use std::sync::Arc;

use fret_app::App;
use fret_ui::{AnyElement, ElementContext};
use fret_ui_shadcn::facade as shadcn;
use fret_runtime::Model;

pub fn example_combobox_multiple(cx: &mut ElementContext<'_, App>) -> AnyElement {
    let values: Model<Vec<Arc<str>>> = cx.app.models_mut().insert(Vec::new());
    let open: Model<bool> = cx.app.models_mut().insert(false);

    shadcn::ComboboxChips::new(values, open).into_element_parts(cx, |_cx| {
        vec![
            shadcn::ComboboxChipsPart::from(
                shadcn::ComboboxValue::new([shadcn::ComboboxChip::new("next").show_remove(true)]),
            ),
            shadcn::ComboboxChipsPart::from(
                shadcn::ComboboxChipsInput::new().placeholder("Add framework"),
            ),
            shadcn::ComboboxChipsPart::from(shadcn::ComboboxContent::new([
                shadcn::ComboboxContentPart::from(
                    shadcn::ComboboxEmpty::new("No items found."),
                ),
                shadcn::ComboboxContentPart::from(shadcn::ComboboxList::new().items([
                    shadcn::ComboboxItem::new("next", "Next.js"),
                    shadcn::ComboboxItem::new("svelte", "SvelteKit"),
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
- Base UI anchor refs (`useComboboxAnchor()`) are modeled via
  `PopoverAnchor::build(child).into_anchor(cx)` + `ComboboxContent::anchor_element_id(...)`
  (element ID), not a DOM ref.
