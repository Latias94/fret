# shadcn/ui v4 Audit — Toggle


## Upstream references (non-normative)

This document references optional local checkouts under `repo-ref/` for convenience.
Upstream sources:

- shadcn/ui: https://github.com/shadcn-ui/ui

See `docs/repo-ref.md` for the optional local snapshot policy and pinned SHAs.
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

- Pass: Exposes a Radix-shaped `ToggleRoot` primitive surface (pressed + defaultPressed), while keeping
  the ergonomic shadcn-aligned `Toggle` builder for styling/variants.
- Pass: Supports controlled state via `Model<bool>`.
- Pass: Supports uncontrolled initial state via `defaultPressed` (`Toggle::uncontrolled(...)` /
  `Toggle::default_pressed(...)` / `toggle_uncontrolled(...)`).
- Pass: Supports a source-aligned snapshot/action path via `Toggle::from_pressed(...)` plus
  `action(...)` / `action_payload(...)`, so action-first views can render a toggle from plain local
  state without allocating a `Model<bool>`.
- Pass: Supports `variant` (`default` / `outline`) and `size` (`sm` / `default` / `lg`).
- Pass: Supports both text labels (`label(...)`) and arbitrary children (`children(...)`).

### Interaction behavior

- Pass: Activation toggles the boolean model when a controlled/uncontrolled model path is used.
- Pass: Snapshot/action toggles can render from plain local state and dispatch typed actions without a model handle.
- Pass: Disabled state prevents interaction and uses disabled styling.
- Pass: A11y uses button semantics and exposes `pressed` state.

### Visual defaults (shadcn parity)

- Pass: Size scale matches shadcn’s wrapper (`h-8 / h-9 / h-10` and matching `min-w-*`) via
  `component.toggle.h{,_sm,_lg}` tokens (with `36px/32px/40px` fallbacks).
- Pass: Horizontal padding matches shadcn (`px-2 / px-1.5 / px-2.5`) via
  `component.toggle.px{,_sm,_lg}` tokens (with `8px/6px/10px` fallbacks).
- Pass: `data-[state=on]` styling matches: `bg-accent` + `text-accent-foreground`.
- Pass: Outline variant hover matches: `hover:bg-accent` + `hover:text-accent-foreground`.
- Pass: Outline variant uses `shadow_xs`, matching shadcn’s `shadow-xs` default.
- Pass: Focus-visible styling includes a ring-colored outline border plus an outward focus ring
  (best-effort, matching `border-ring` and `ring-ring/50`).

## Validation

- `cargo nextest run -p fret-ui-shadcn toggle`
- `cargo test -p fret-ui-shadcn --lib toggle_pressed_value_exposes_semantics_without_model --message-format short`
- `cargo check -p fret-cookbook --example toggle_basics --message-format short`
- `cargo test -p fret-ui-shadcn --lib field_label_click_dispatches_action_for_snapshot_toggle_control --message-format short`

## Follow-ups (recommended)

- Pass: Snapshot/action toggles now participate in `control_id` / label forwarding with the same command-backed activation path as the model-backed contract.
- Consider an icon-sizing helper to match shadcn’s default `svg.size-4` rule.
