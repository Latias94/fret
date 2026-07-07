# shadcn/ui v4 Audit - Label

## Upstream references (non-normative)

This document references optional local checkouts under `repo-ref/` for convenience.
Upstream sources:

- shadcn/ui: https://github.com/shadcn-ui/ui

See `docs/repo-ref.md` for the optional local snapshot policy and pinned SHAs.
This audit compares Fret's shadcn-aligned `Label` against the current upstream shadcn/ui v4 docs,
registry examples, and the current label layout/interaction gates.

## Upstream references (source of truth)

- Docs page: `repo-ref/ui/apps/v4/content/docs/components/label.mdx`
- Component implementation: `repo-ref/ui/apps/v4/registry/new-york-v4/ui/label.tsx`
- Headless references: `repo-ref/ui/apps/v4/registry/bases/base/ui/label.tsx`, `repo-ref/ui/apps/v4/registry/bases/radix/ui/label.tsx`
- Example compositions: `repo-ref/ui/apps/v4/registry/new-york-v4/examples/label-demo.tsx`, `repo-ref/ui/apps/v4/registry/new-york-v4/examples/input-with-label.tsx`, `repo-ref/ui/apps/v4/registry/new-york-v4/examples/textarea-with-label.tsx`, `repo-ref/ui/apps/v4/registry/new-york-v4/examples/input-group-label.tsx`
- Existing layout gates: `goldens/shadcn-web/v4/new-york-v4/label-demo.json`

## Fret implementation

- Primitive implementation: `ecosystem/fret-ui-kit/src/primitives/label.rs`
- Re-export surface: `ecosystem/fret-ui-shadcn/src/label.rs`
- Gallery page: `apps/fret-ui-gallery/src/ui/pages/label.rs`
- Gallery docs-surface gate: `apps/fret-ui-gallery/tests/label_docs_surface.rs`
- Label docs gate: `tools/diag-scripts/ui-gallery/label/ui-gallery-label-docs-smoke.json`
- Label interaction gate: `tools/diag-scripts/ui-gallery/label/ui-gallery-label-click-label-toggles-checkbox.json`

## Audit checklist

### Authoring surface

- Pass: `Label::new(text)` plus `for_control(...)` covers the documented label surface.
- Pass: `Label::for_control(...)` plus control-side `control_id(...)` is the right Fret bridge for the upstream `htmlFor` / `id` pairing.
- Pass: `Label::children(...)` now covers the common composable-inline lane in the shadcn recipe layer while preserving the label text as the accessible name and association label.
- Pass: form-specific structure remains on `Field`, `FieldLabel`, `FieldDescription`, and `FieldError`; Fret does not need to widen `Label` itself.
- Pass: `Label::wrap(...)` is now the explicit full-subtree lane when a first-party non-field label needs custom visible content; `FieldLabel::wrap(...)` remains the form-specific richer structure surface.

### Layout & default-style ownership

- Pass: text sizing and line-height remain recipe-owned on the label primitive.
- Pass: `fret-ui-shadcn::Label` now owns the upstream `flex items-center gap-2` row outcome instead of short-circuiting directly to the headless primitive export.
- Pass: disabled associated labels now match the upstream `opacity-50` outcome.
- Pass: plain associated labels no longer let ambient pressable shells suppress the documented click-to-toggle / click-to-focus path.
- Pass: surrounding form layout, width caps, and label-plus-control stacking remain caller-owned composition.
- Pass: `label-demo` geometry and the peer-disabled marker remain covered by existing web gates.

### Gallery / docs parity

- Pass: the gallery now mirrors the current upstream docs path first: checkbox `Demo` and `Usage`.
- Pass: `Label in Field`, `RTL`, `Composable Content`, and `API Reference` are explicitly retained after the current upstream docs path as Fret follow-ups.
- Pass: `apps/fret-ui-gallery/tests/label_docs_surface.rs` locks the page order, copyable association snippets, raw-free `Label in Field` example, and Label diagnostics anchors.
- Pass: this work is still a narrow public-surface/recipe alignment plus existing primitive association fixes: disabled associated-label opacity and associated-label forwarding under ambient pressable shells.

## Validation

- `cargo nextest run -p fret-ui-kit --lib --status-level fail label_for_disabled_control_uses_half_opacity label_for_control_click_invokes_registered_control_action_inside_ancestor_pressable`
- `cargo nextest run -p fret-ui-shadcn --lib --status-level fail label`
- `cargo nextest run -p fret-ui-shadcn --features web-goldens --test web_vs_fret_layout --status-level fail label_demo`
- `cargo nextest run -p fret-ui-shadcn --test web_vs_fret_misc_targeted --status-level fail`
- `cargo nextest run -p fret-ui-shadcn --test input_label_focus --test textarea_label_focus --status-level fail`
- `cargo nextest run -p fret-ui-gallery --test label_docs_surface`
- `cargo nextest run -p fret-ui-gallery --test ui_authoring_surface_default_app --status-level fail label`
- Existing layout gate: `ecosystem/fret-ui-shadcn/tests/web_vs_fret_layout/basic.rs` (`web_vs_fret_layout_label_demo_geometry`)
- Existing targeted marker gate: `ecosystem/fret-ui-shadcn/tests/web_vs_fret_misc_targeted.rs`
- `Get-ChildItem -Path tools/diag-scripts/ui-gallery/label -Filter *.json | ForEach-Object { python -m json.tool $_.FullName | Out-Null }`
