# shadcn/ui v4 Audit — Toggle

This audit compares Fret’s shadcn-aligned `Toggle` against the upstream shadcn/ui v4 docs and
the `new-york-v4` implementation in `repo-ref/ui`.

## Upstream references (source of truth)

- Docs page: `repo-ref/ui/apps/v4/content/docs/components/toggle.mdx`
- Registry implementation (new-york): `repo-ref/ui/apps/v4/registry/new-york-v4/ui/toggle.tsx`
- Underlying primitive: Radix `@radix-ui/react-toggle`

## Fret implementation

- Component code: `ecosystem/fret-ui-shadcn/src/toggle.rs`

## Audit checklist

### Composition surface

- Partial: Fret exposes a `Toggle` builder instead of a fully composable primitive surface.
- Pass: Supports controlled state via `Model<bool>`.
- Pass: Supports `variant` (`default` / `outline`) and `size` (`sm` / `default` / `lg`).
- Pass: Supports both text labels (`label(...)`) and arbitrary children (`children(...)`).

### Interaction behavior

- Pass: Activation toggles the boolean model.
- Pass: Disabled state prevents interaction and uses disabled styling.
- Pass: A11y uses button semantics and exposes `selected` state.

### Visual defaults (shadcn parity)

- Pass: Size scale matches shadcn’s wrapper (`h-8 / h-9 / h-10` and matching `min-w-*`) via
  `component.toggle.h{,_sm,_lg}` tokens (with `36px/32px/40px` fallbacks).
- Pass: Horizontal padding matches shadcn (`px-2 / px-1.5 / px-2.5`) via
  `component.toggle.px{,_sm,_lg}` tokens (with `8px/6px/10px` fallbacks).
- Pass: `data-[state=on]` styling matches: `bg-accent` + `text-accent-foreground`.
- Pass: Outline variant hover matches: `hover:bg-accent` + `hover:text-accent-foreground`.
- Pass: Focus-visible styling includes a ring-colored outline border plus an outward focus ring
  (best-effort, matching `border-ring` and `ring-ring/50`).

## Validation

- `cargo test -p fret-ui-shadcn --lib toggle`

## Follow-ups (recommended)

- Consider introducing a shared “shadow-xs” token/recipe (upstream outline toggle uses `shadow-xs`).
- Consider an icon-sizing helper to match shadcn’s default `svg.size-4` rule.
