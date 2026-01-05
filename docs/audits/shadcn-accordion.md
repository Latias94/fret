# shadcn/ui v4 Audit — Accordion

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

- Partial: Fret exposes `Accordion::single(...)` / `Accordion::multiple(...)` builders with
  `AccordionItem` entries, rather than a fully composable `AccordionItem` / `AccordionTrigger` /
  `AccordionContent` surface mirroring Radix/shadcn.
- Pass: Supports both single and multiple open models (`Model<Option<Arc<str>>>` and
  `Model<Vec<Arc<str>>>`), including `collapsible(true)` in the single-open mode.

### Keyboard & interaction behavior

- Pass: Trigger activation toggles open state.
- Pass: Arrow-key roving navigation is implemented via `RovingFlex` + `cx.roving_nav_apg()`.
- Partial: Hover underline styling is intentionally simplified compared to the upstream Tailwind
  implementation.
- Partial: Content open/close animations use a presence-driven cached-height clip (best-effort; no
  CSS variables yet).
  - Shared helper: `ecosystem/fret-ui-kit/src/declarative/collapsible_motion.rs`

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

## Follow-ups (recommended)

- Consider adding a composable surface mirroring Radix/shadcn (`AccordionItem` / `Trigger` /
  `Content`) if authoring ergonomics become a priority.
