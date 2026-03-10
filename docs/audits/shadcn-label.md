# shadcn/ui v4 Audit - Label

## Upstream references (non-normative)

This document references optional local checkouts under `repo-ref/` for convenience.
Upstream sources:

- shadcn/ui: https://github.com/shadcn-ui/ui

See `docs/repo-ref.md` for the optional local snapshot policy and pinned SHAs.
This audit compares Fret's shadcn-aligned `Label` against the upstream shadcn/ui v4 base docs,
base examples, and the existing label layout gates.

## Upstream references (source of truth)

- Docs page: `repo-ref/ui/apps/v4/content/docs/components/base/label.mdx`
- Component implementation: `repo-ref/ui/apps/v4/examples/base/ui/label.tsx`
- Example compositions: `repo-ref/ui/apps/v4/examples/base/label-demo.tsx`, `repo-ref/ui/apps/v4/examples/base/field-demo.tsx`, `repo-ref/ui/apps/v4/examples/base/label-rtl.tsx`
- Existing layout gates: `goldens/shadcn-web/v4/new-york-v4/label-demo.json`

## Fret implementation

- Primitive implementation: `ecosystem/fret-ui-kit/src/primitives/label.rs`
- Re-export surface: `ecosystem/fret-ui-shadcn/src/label.rs`
- Gallery page: `apps/fret-ui-gallery/src/ui/pages/label.rs`

## Audit checklist

### Authoring surface

- Pass: `Label::new(text)` plus `for_control(...)` covers the documented label surface.
- Pass: `Label::for_control(...)` plus control-side `control_id(...)` is the right Fret bridge for the upstream `htmlFor` / `id` pairing.
- Pass: form-specific structure remains on `Field`, `FieldLabel`, `FieldDescription`, and `FieldError`; Fret does not need to widen `Label` itself.
- Pass: no extra generic children / `asChild` / `compose()` API is needed here.

### Layout & default-style ownership

- Pass: text sizing and line-height remain recipe-owned on the label primitive.
- Pass: surrounding form layout, width caps, and label-plus-control stacking remain caller-owned composition.
- Pass: `label-demo` geometry and the peer-disabled marker remain covered by existing web gates.

### Gallery / docs parity

- Pass: the gallery now mirrors the upstream base Label docs path first: `Demo`, `Usage`, `Label in Field`, `RTL`, and `API Reference`.
- Pass: this work is docs/public-surface parity, not a mechanism-layer fix.

## Validation

- `CARGO_TARGET_DIR=target-codex-avatar cargo check -p fret-ui-gallery --message-format short`
- Existing layout gate: `ecosystem/fret-ui-shadcn/tests/web_vs_fret_layout/basic.rs` (`web_vs_fret_layout_label_demo_geometry`)
- Existing targeted marker gate: `ecosystem/fret-ui-shadcn/tests/web_vs_fret_misc_targeted.rs`
