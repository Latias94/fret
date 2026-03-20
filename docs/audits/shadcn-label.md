# shadcn/ui v4 Audit - Label

## Upstream references (non-normative)

This document references optional local checkouts under `repo-ref/` for convenience.
Upstream sources:

- shadcn/ui: https://github.com/shadcn-ui/ui

See `docs/repo-ref.md` for the optional local snapshot policy and pinned SHAs.
This audit compares Fret's shadcn-aligned `Label` against the upstream shadcn/ui v4 base/radix
docs, registry examples, and the current label layout/interaction gates.

## Upstream references (source of truth)

- Docs page: `repo-ref/ui/apps/v4/content/docs/components/base/label.mdx`
- Docs page (radix): `repo-ref/ui/apps/v4/content/docs/components/radix/label.mdx`
- Component implementation: `repo-ref/ui/apps/v4/registry/new-york-v4/ui/label.tsx`
- Headless references: `repo-ref/ui/apps/v4/registry/bases/base/ui/label.tsx`, `repo-ref/ui/apps/v4/registry/bases/radix/ui/label.tsx`
- Example compositions: `repo-ref/ui/apps/v4/registry/new-york-v4/examples/label-demo.tsx`, `repo-ref/ui/apps/v4/registry/new-york-v4/examples/input-with-label.tsx`, `repo-ref/ui/apps/v4/examples/base/label-rtl.tsx`
- Existing layout gates: `goldens/shadcn-web/v4/new-york-v4/label-demo.json`

## Fret implementation

- Primitive implementation: `ecosystem/fret-ui-kit/src/primitives/label.rs`
- Re-export surface: `ecosystem/fret-ui-shadcn/src/label.rs`
- Gallery page: `apps/fret-ui-gallery/src/ui/pages/label.rs`
- Label docs gate: `tools/diag-scripts/ui-gallery/label/ui-gallery-label-docs-smoke.json`
- Label interaction gate: `tools/diag-scripts/ui-gallery/label/ui-gallery-label-click-label-toggles-checkbox.json`

## Audit checklist

### Authoring surface

- Pass: `Label::new(text)` plus `for_control(...)` covers the documented label surface.
- Pass: `Label::for_control(...)` plus control-side `control_id(...)` is the right Fret bridge for the upstream `htmlFor` / `id` pairing.
- Pass: form-specific structure remains on `Field`, `FieldLabel`, `FieldDescription`, and `FieldError`; Fret does not need to widen `Label` itself.
- Decision: do not widen `Label` into a generic compound-children / `compose()` API yet.
  The current shadcn/base/radix docs path only needs text children plus association, and richer
  clickable subtrees already have a recipe-owned home in `FieldLabel::wrap(...)`. Revisit this
  only if a first-party non-field `Label` use case needs wrapped-child ownership.

### Layout & default-style ownership

- Pass: text sizing and line-height remain recipe-owned on the label primitive.
- Pass: disabled associated labels now match the upstream `opacity-50` outcome.
- Pass: plain associated labels no longer let ambient pressable shells suppress the documented click-to-toggle / click-to-focus path.
- Pass: surrounding form layout, width caps, and label-plus-control stacking remain caller-owned composition.
- Pass: `label-demo` geometry and the peer-disabled marker remain covered by existing web gates.

### Gallery / docs parity

- Pass: the gallery now mirrors the upstream docs path first: checkbox `Demo`, `Usage`, `Label in Field`, `RTL`, and `API Reference`.
- Pass: this work is mostly docs/public-surface parity plus two narrow primitive fixes: disabled associated-label opacity and associated-label forwarding under ambient pressable shells.

## Validation

- `CARGO_TARGET_DIR=target-codex-avatar cargo check -p fret-ui-gallery --message-format short`
- `cargo test -p fret-ui-kit label_for_disabled_control_uses_half_opacity -- --nocapture`
- `cargo test -p fret-ui-kit label_for_control_click_invokes_registered_control_action_inside_ancestor_pressable -- --nocapture`
- Existing layout gate: `ecosystem/fret-ui-shadcn/tests/web_vs_fret_layout/basic.rs` (`web_vs_fret_layout_label_demo_geometry`)
- Existing targeted marker gate: `ecosystem/fret-ui-shadcn/tests/web_vs_fret_misc_targeted.rs`
- `cargo run -p fretboard -- diag run tools/diag-scripts/ui-gallery/label/ui-gallery-label-docs-smoke.json --launch -- cargo run -p fret-ui-gallery --release`
- `cargo run -p fretboard -- diag run tools/diag-scripts/ui-gallery/label/ui-gallery-label-click-label-toggles-checkbox.json --launch -- cargo run -p fret-ui-gallery --release`
