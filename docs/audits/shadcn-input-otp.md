# shadcn/ui v4 Audit - Input OTP

## Upstream references (non-normative)

This document references optional local checkouts under `repo-ref/` for convenience.
Upstream sources:

- shadcn/ui: https://github.com/shadcn-ui/ui
- input-otp (upstream engine): https://github.com/guilhermerodz/input-otp

See `docs/repo-ref.md` for the optional local snapshot policy and pinned SHAs.

## Upstream references (source of truth)

- shadcn docs: `repo-ref/ui/apps/v4/content/docs/components/radix/input-otp.mdx`
- shadcn examples (New York v4):
  - `repo-ref/ui/apps/v4/registry/new-york-v4/examples/input-otp-demo.tsx`
  - `repo-ref/ui/apps/v4/registry/new-york-v4/examples/input-otp-pattern.tsx`
  - `repo-ref/ui/apps/v4/registry/new-york-v4/examples/input-otp-separator.tsx`
  - `repo-ref/ui/apps/v4/registry/new-york-v4/examples/input-otp-controlled.tsx`

## Fret implementation anchors

- Component code: `ecosystem/fret-ui-shadcn/src/input_otp.rs`
- Gallery demo: `apps/fret-ui-gallery/src/ui/pages/input_otp.rs`

## Audit checklist

### Composition model (single hidden input + visual slots)

- Pass: Visual slots are authored as a layout-only facade over a single input model.
- Pass: Active slot is reflected via semantics `selected=true` on the active slot for diagnostics gating.
- Pass: Separators are rendered between groups (`group_size`) with a shadcn-aligned minus icon.

### Invalid state (shadcn: `aria-invalid`)

The shadcn docs describe using `aria-invalid` on slots to show an error state.
In Fret, the equivalent outcome is modeled as a recipe-level toggle:

- Pass: `InputOtp::aria_invalid(true)` applies destructive border chrome to all slots.
- Pass: Focus ring uses destructive-tinted ring colors when invalid.

### Disabled state

- Pass: `InputOtp::disabled(true)` disables focus/typing and matches the `opacity-50` disabled outcome.

## Validation

- `cargo nextest run -p fret-ui-shadcn input_otp_aria_invalid_uses_destructive_border_color`
- `cargo run -p fretboard -- diag run tools/diag-scripts/ui-gallery-input-otp-docs-smoke.json --launch -- cargo run -p fret-ui-gallery`

