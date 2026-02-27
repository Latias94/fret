# Theme Token Alignment v1 — Milestones

This document defines the milestone plan for auditing and migrating ecosystem components to the
token taxonomy defined in `design.md`.

Status legend:

- `[ ]` Not started
- `[~]` In progress
- `[x]` Done

## M0 — Lock rules + inventory seed

- [ ] Confirm the token taxonomy (semantic vs named vs component-derived) is sufficient for shadcn v4 parity.
- [ ] Publish a minimal audit playbook in `todo.md` (how to scan, how to decide, what gates to add).
- [ ] Seed an initial tracker table covering at least:
  - shadcn core controls (Button, Badge, Input, Select, Checkbox, Radio, Switch, Slider)
  - overlay family (Dialog, Sheet/Drawer, Popover, DropdownMenu, ContextMenu, Tooltip, HoverCard)
  - data viz (Chart/Plot) and editor surfaces (CodeView/Markdown) as “audit targets”.

Exit criteria:

- The tracker exists and a contributor can add a new row in < 2 minutes.
- At least one example shows each token class (semantic, named, component-derived) and how to gate it.

## M1 — Close high-signal shadcn v4 literal tokens

- [ ] Ensure all upstream `text-white` / `bg-white` / `bg-black/*` parity points are expressed via `ThemeNamedColorKey`
      (or explicitly justified as semantic).
- [ ] Ensure all upstream `dark:*` background deltas that affect contrast are encoded via component-derived tokens.

Exit criteria:

- No shadcn recipe uses semantic `*-foreground` to represent upstream `text-white`.
- zinc/dark has diag screenshot gates for the highest-risk rows (Button destructive, Badge destructive, overlay scrim).

## M2 — Overlay family completeness (style + token rules)

- [ ] Audit all overlay components for scrim/background/foreground rules where upstream uses literal colors or alpha.
- [ ] Ensure overlay tokens are seeded consistently across presets.

Exit criteria:

- Each overlay family member has at least one “zinc/dark” diag screenshot scenario focusing on scrim + content contrast.

## M3 — Cross-ecosystem adoption (Material3 + charts/plot/editor)

- [ ] Audit Material3 recipes for literal color assumptions (e.g. white/black surfaces) and replace with semantic or named rules.
- [ ] Audit chart/plot/editor ecosystems for “literal white/black” usage that should be semantic (or named when upstream requires).

Exit criteria:

- Named literal colors remain minimal (no “tailwind palette creep”).
- Cross-ecosystem tokens do not conflict in meaning (component-derived tokens remain component-scoped).

## M4 — Hardening + drift prevention

- [ ] Add a lightweight lint/check (optional) that flags new uses of disallowed patterns (e.g. `color_token("white")`).
- [ ] Document “how to add a new named color” decision gate and the evidence required.

Exit criteria:

- Contributors can safely port new shadcn recipes without reintroducing semantic-vs-literal confusion.

