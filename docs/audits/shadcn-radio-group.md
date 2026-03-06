# shadcn/ui v4 Audit — Radio Group


## Upstream references (non-normative)

This document references optional local checkouts under `repo-ref/` for convenience.
Upstream sources:

- shadcn/ui: https://github.com/shadcn-ui/ui

See `docs/repo-ref.md` for the optional local snapshot policy and pinned SHAs.
This audit compares Fret’s shadcn-aligned `RadioGroup` against the upstream shadcn/ui v4 docs and
examples in `repo-ref/ui`.

## Upstream references (source of truth)

- Component wrapper (Radix RadioGroup skin): `repo-ref/ui/apps/v4/registry/new-york-v4/ui/radio-group.tsx`
- Docs page: `repo-ref/ui/apps/v4/content/docs/components/radio-group.mdx`

## Fret implementation

- Component code: `ecosystem/fret-ui-shadcn/src/radio_group.rs`
- Key building blocks:
  - Roving focus utilities: `ecosystem/fret-ui-kit/src/headless/roving_focus.rs`
  - APG navigation hooks: `ecosystem/fret-ui-kit/src/declarative/collection_semantics.rs`

## Audit checklist

### Composition surface

- Pass: `RadioGroupItem` supports composable contents via `RadioGroupItem::children(...)` while keeping
  the convenient `RadioGroup::item(RadioGroupItem::new(value, label))` builder shape.
- Note: Because `RadioGroupItem::new(value, label)` already covers the common shadcn authoring path and
  `children(...)` covers richer card content, Fret intentionally does not add a separate generic
  `compose()` builder for this component.
- Pass: Supports a controlled selection model via `Model<Option<Arc<str>>>`.
- Pass: Supports uncontrolled `defaultValue` (internal selection model).

### Keyboard & selection behavior

- Pass: Arrow-key roving navigation is implemented via `RovingFlex` + `cx.roving_nav_apg()`.
- Pass: Selection commits via `cx.roving_select_option_arc_str(...)` (Radix RadioGroup behavior).
- Pass: Enter key presses are consumed to match Radix/WAI-ARIA expectations; Space activates items.
- Pass: Supports Radix `orientation` outcomes (`RadioGroupOrientation::Vertical` / `Horizontal`).
- Pass: `loop_navigation(true)` defaults to looping behavior (Radix `loop` default).

### Visual defaults (shadcn parity)

- Pass: Item sizing defaults to `size-4` (`16px`) via `component.radio_group.icon_size_px`.
- Pass: Item border defaults to `border-input` and switches to `border-ring` on focus.
- Pass: Selected indicator uses `primary` (dot fill), matching shadcn’s `CircleIcon fill-primary`.
- Pass: The item icon uses `shadow_xs`, matching shadcn’s `shadow-xs` default.
- Pass: Focus ring thickness (`ring-[3px]`) matches shadcn-web focus variant (`radio-group-demo.focus`).

## Validation

- `cargo test -p fret-ui-shadcn --lib radio_group`
- Web layout gate: `cargo nextest run -p fret-ui-shadcn --test web_vs_fret_layout`
  (`web_vs_fret_layout_radio_group_demo_row_geometry`).
- Focus ring gate: `cargo nextest run -p fret-ui-shadcn --test web_vs_fret_control_chrome`
  (`web_vs_fret_radio_group_demo_focus_ring_matches`).

## Follow-ups (recommended)

- None at the moment.
