# shadcn/ui v4 Audit — Accordion


## Upstream references (non-normative)

This document references optional local checkouts under `repo-ref/` for convenience.
Upstream sources:

- shadcn/ui: https://github.com/shadcn-ui/ui

See `docs/repo-ref.md` for the optional local snapshot policy and pinned SHAs.
This audit compares Fret’s shadcn-aligned `Accordion` against the upstream shadcn/ui v4 docs and
examples in `repo-ref/ui`.

## Upstream references (source of truth)

- Docs page: `repo-ref/ui/apps/v4/content/docs/components/accordion.mdx`
- Component wrapper (Radix Accordion skin): `repo-ref/ui/apps/v4/registry/new-york-v4/ui/accordion.tsx`
- Demo usage: `repo-ref/ui/apps/v4/registry/new-york-v4/examples/accordion-demo.tsx`

## Fret implementation

- Component code: `ecosystem/fret-ui-shadcn/src/accordion.rs`
- Key building blocks:
  - Roving focus utilities: `ecosystem/fret-ui-kit/src/headless/roving_focus.rs`
  - APG navigation hooks: `ecosystem/fret-ui-kit/src/declarative/collection_semantics.rs`

## Audit checklist

### Composition surface

- Pass: A Radix-shaped, shadcn-skinned component family exists in
  `fret_ui_shadcn::accordion::composable` (`AccordionRoot` / `AccordionItem` / `AccordionTrigger` /
  `AccordionContent`).
- Pass: The legacy builder-style API (`Accordion::single(...)` / `Accordion::multiple(...)`) remains
  available for compact internal recipes.
- Pass: Supports both single and multiple open models (`Model<Option<Arc<str>>>` and
  `Model<Vec<Arc<str>>>`), including `collapsible(true)` in the single-open mode.
- Pass: Supports uncontrolled `defaultValue` (internal selection model).

Note: A fully composable, Radix-shaped surface exists in the primitives layer for non-shadcn users
(`fret-ui-kit::primitives::accordion::AccordionRoot` / `AccordionTrigger` / `AccordionContent`),
while the shadcn wrapper keeps a builder-style ergonomic API.

### Keyboard & interaction behavior

- Pass: Trigger activation toggles open state.
- Pass: Arrow-key roving navigation is implemented via `RovingFlex` + `cx.roving_nav_apg()`.
- Partial: Hover underline styling is intentionally simplified compared to the upstream Tailwind
  implementation.
- Pass: Content open/close animations are driven by a cached measured height + presence timeline,
  matching Radix/shadcn's outcomes (without CSS variables).
  - Shared helper: `ecosystem/fret-ui-kit/src/primitives/collapsible.rs` (delegates to `declarative/collapsible_motion.rs`)

### Visual defaults (shadcn parity)

- Pass: Items render a bottom border only, matching `border-b`, and the last item removes the bottom
  border (matching `last:border-b-0`).
- Pass: Trigger padding matches `py-4` via `Space::N4`.
- Pass: Trigger gap matches `gap-4` via `component.accordion.trigger.gap` (fallback uses the
  component spacing scale).
- Pass: Trigger includes a trailing chevron (`CHEVRON_DOWN`) and rotates it when open.
- Pass: Content padding matches `pt-0 pb-4` (`Space::N0` / `Space::N4`).

## Validation

- `cargo test -p fret-ui-shadcn --lib accordion`
- shadcn web golden (geometry-first): `cargo nextest run -p fret-ui-shadcn --test web_vs_fret_layout`
  - Gates: `web_vs_fret_layout_accordion_demo_geometry_light` / `web_vs_fret_layout_accordion_demo_geometry_dark`

## Follow-ups (recommended)

- Consider adding a composable surface mirroring Radix/shadcn (`AccordionItem` / `Trigger` /
  `Content`) if authoring ergonomics become a priority.
