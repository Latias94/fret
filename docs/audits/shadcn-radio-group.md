# shadcn/ui v4 Audit — Radio Group

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

- Partial: Fret exposes a `RadioGroup` builder with labeled `RadioGroupItem` entries; shadcn’s wrapper
  exposes `RadioGroup` + `RadioGroupItem` primitives and expects labels to be composed externally.
- Pass: Supports a controlled selection model via `Model<Option<Arc<str>>>`.

### Keyboard & selection behavior

- Pass: Arrow-key roving navigation is implemented via `RovingFlex` + `cx.roving_nav_apg()`.
- Pass: Selection commits via `cx.roving_select_option_arc_str(...)` (Radix RadioGroup behavior).
- Pass: Supports Radix `orientation` outcomes (`RadioGroupOrientation::Vertical` / `Horizontal`).
- Pass: `loop_navigation(true)` defaults to looping behavior (Radix `loop` default).

### Visual defaults (shadcn parity)

- Pass: Item sizing defaults to `size-4` (`16px`) via `component.radio_group.icon_size_px`.
- Pass: Item border defaults to `border-input` and switches to `border-ring` on focus.
- Pass: Selected indicator uses `primary` (dot fill), matching shadcn’s `CircleIcon fill-primary`.
- Partial: The shadcn wrapper applies `shadow-xs`; Fret currently does not model an `xs` shadow tier.

## Validation

- `cargo test -p fret-ui-shadcn --lib radio_group`

## Follow-ups (recommended)

- Consider adding an `xs` shadow tier to match shadcn’s `shadow-xs` token usage.
