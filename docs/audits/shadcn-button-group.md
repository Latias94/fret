# shadcn/ui v4 Audit - Button Group

## Upstream references (non-normative)

This document references optional local checkouts under `repo-ref/` for convenience.
Upstream sources:

- shadcn/ui: https://github.com/shadcn-ui/ui

See `docs/repo-ref.md` for the optional local snapshot policy and pinned SHAs.
This audit compares Fret's shadcn-aligned `ButtonGroup` against the upstream shadcn/ui v4 docs and
base example implementations in `repo-ref/ui`.

## Upstream references (source of truth)

- Docs page: `repo-ref/ui/apps/v4/content/docs/components/base/button-group.mdx`
- Component implementation: `repo-ref/ui/apps/v4/examples/base/ui/button-group.tsx`
- Example compositions: `repo-ref/ui/apps/v4/examples/base/button-group-demo.tsx`, `repo-ref/ui/apps/v4/examples/base/button-group-orientation.tsx`, `repo-ref/ui/apps/v4/examples/base/button-group-size.tsx`, `repo-ref/ui/apps/v4/examples/base/button-group-nested.tsx`, `repo-ref/ui/apps/v4/examples/base/button-group-separator.tsx`, `repo-ref/ui/apps/v4/examples/base/button-group-split.tsx`, `repo-ref/ui/apps/v4/examples/base/button-group-input.tsx`, `repo-ref/ui/apps/v4/examples/base/button-group-input-group.tsx`, `repo-ref/ui/apps/v4/examples/base/button-group-dropdown.tsx`, `repo-ref/ui/apps/v4/examples/base/button-group-select.tsx`, `repo-ref/ui/apps/v4/examples/base/button-group-popover.tsx`, `repo-ref/ui/apps/v4/examples/base/button-group-rtl.tsx`

## Fret implementation

- Component code: `ecosystem/fret-ui-shadcn/src/button_group.rs`
- Gallery page: `apps/fret-ui-gallery/src/ui/pages/button_group.rs`

## Audit checklist

### Authoring surface

- Pass: `ButtonGroup::new(...)` covers the primary upstream composition model for grouped action buttons.
- Pass: `ButtonGroup::a11y_label(...)` provides the `aria-label` equivalent required by the upstream accessibility guidance.
- Pass: `ButtonGroupSeparator::new().orientation(...)` maps directly to the documented separator surface.
- Pass: `ButtonGroupText::new(...)` and `ButtonGroupText::children(...)` cover the upstream `ButtonGroupText` use cases without adding a generic `asChild` slot merge surface.
- Pass: first-party composition examples now also follow the child family defaults: the embedded
  `InputGroup` example uses the compact slot shorthand, and the embedded `Select` example uses the
  compact direct root chain instead of reintroducing child-family parts adapters inside
  `ButtonGroup`.
- Note: Fret intentionally keeps `ButtonGroup` distinct from `ToggleGroup`; pressed-state behavior remains owned by `toggle_group` rather than being overloaded into the action-group recipe.

### Layout & default-style ownership

- Pass: Recipe-owned defaults cover the upstream intrinsic chrome: `w-fit` root sizing, merged borders, outer-corner preservation, separator thickness, and nested-group `gap-2` spacing.
- Pass: Caller-owned layout negotiation stays at the page/example level: `w_full`, `flex_1`, max-width constraints, and container composition remain explicit in snippets such as `Flex-1 items`.
- Pass: Input/select/menu/popover compositions are handled through typed items and edge/corner overrides, not through generic slot prop merging.

### Gallery / docs parity

- Pass: The gallery now mirrors the upstream docs path first: `Demo`, `Usage`, `Accessibility`, `ButtonGroup vs ToggleGroup`, the example set through `RTL`, and `API Reference`.
- Pass: `ButtonGroupText` appears as an explicit follow-up section after `API Reference`, which keeps the page source-comparable while still giving Fret a copyable equivalent for the upstream `ButtonGroupText` API examples.
- Pass: `Flex-1 items` remains a Fret-specific extension after the upstream path because it demonstrates caller-owned flex negotiation rather than intrinsic recipe behavior.

## Validation

- `cargo nextest run -p fret-ui-shadcn --test web_vs_fret_control_chrome web_vs_fret_button_group_demo_button_chrome_matches --status-level fail`
- `cargo nextest run -p fret-ui-shadcn --test web_vs_fret_control_chrome web_vs_fret_button_group_nested_geometry_and_chrome_match --status-level fail`
- `cargo nextest run -p fret-ui-shadcn --test web_vs_fret_control_chrome web_vs_fret_button_group_separator_geometry_and_chrome_match --status-level fail`
- `cargo nextest run -p fret-ui-shadcn --test web_vs_fret_control_chrome web_vs_fret_button_group_dropdown_geometry_and_chrome_match --status-level fail`
