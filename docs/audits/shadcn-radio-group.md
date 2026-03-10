# shadcn/ui v4 Audit - Radio Group

## Upstream references (non-normative)

This document references optional local checkouts under `repo-ref/` for convenience.
Upstream sources:

- shadcn/ui: https://github.com/shadcn-ui/ui

See `docs/repo-ref.md` for the optional local snapshot policy and pinned SHAs.
This audit compares Fret's shadcn-aligned `RadioGroup` against the upstream shadcn/ui v4 base docs,
base examples, and the existing radio-group gates.

## Upstream references (source of truth)

- Docs page: `repo-ref/ui/apps/v4/content/docs/components/base/radio-group.mdx`
- Component implementation: `repo-ref/ui/apps/v4/examples/base/ui/radio-group.tsx`
- Example compositions: `repo-ref/ui/apps/v4/examples/base/radio-group-demo.tsx`, `repo-ref/ui/apps/v4/examples/base/radio-group-description.tsx`, `repo-ref/ui/apps/v4/examples/base/radio-group-choice-card.tsx`, `repo-ref/ui/apps/v4/examples/base/radio-group-fieldset.tsx`, `repo-ref/ui/apps/v4/examples/base/radio-group-disabled.tsx`, `repo-ref/ui/apps/v4/examples/base/radio-group-invalid.tsx`, `repo-ref/ui/apps/v4/examples/base/radio-group-rtl.tsx`

## Fret implementation

- Component code: `ecosystem/fret-ui-shadcn/src/radio_group.rs`
- Gallery page: `apps/fret-ui-gallery/src/ui/pages/radio_group.rs`

## Audit checklist

### Authoring surface

- Pass: `RadioGroup::uncontrolled(default)` and `RadioGroup::new(model)` cover the documented uncontrolled and controlled authoring paths.
- Pass: `RadioGroupItem::children(...)` and `variant(RadioGroupItemVariant::ChoiceCard)` cover the richer description and choice-card compositions.
- Pass: no extra generic `compose()` API is needed here because the existing item children surface already matches the upstream composition model, including the invalid row composition.
- Pass: `control_id(...)` remains the focused Fret bridge for label-forwarding and stays out of the upstream docs path.

### Interaction & default-style ownership

- Pass: selection semantics, roving navigation, icon chrome, border, and focus ring remain recipe-owned.
- Pass: surrounding fieldset, card width, and row layout remain caller-owned composition.
- Pass: existing radio-group layout and focus gates continue to cover representative interaction and geometry outcomes.

### Gallery / docs parity

- Pass: the gallery now mirrors the upstream base Radio Group docs path first: `Demo`, `Usage`, `Description`, `Choice Card`, `Fieldset`, `Disabled`, `Invalid`, `RTL`, and `API Reference`.
- Pass: `Label Association` remains a focused Fret follow-up after the upstream path because it documents the `control_id(...)` bridge.
- Pass: this work is docs/public-surface parity, not a mechanism-layer fix.

## Validation

- `CARGO_TARGET_DIR=target-codex-avatar cargo check -p fret-ui-gallery --message-format short`
- Existing layout gate: `ecosystem/fret-ui-shadcn/tests/web_vs_fret_layout/radio_group.rs` (`radio_group::web_vs_fret_layout_radio_group_demo_geometry_matches_web_fixtures`)
- Existing focus gate: `ecosystem/fret-ui-shadcn/tests/web_vs_fret_control_chrome.rs` (`web_vs_fret_radio_group_demo_focus_ring_matches`)
