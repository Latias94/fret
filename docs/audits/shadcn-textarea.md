# shadcn/ui v4 Audit - Textarea

## Upstream references (non-normative)

This document references optional local checkouts under `repo-ref/` for convenience.
Upstream sources:

- shadcn/ui: https://github.com/shadcn-ui/ui

See `docs/repo-ref.md` for the optional local snapshot policy and pinned SHAs.
This audit compares Fret's shadcn-aligned `Textarea` against the current main-worktree
shadcn/ui v4 docs path, the `new-york-v4` recipe and examples, the base/radix registry
Field examples, and the in-repo textarea web, Gallery, diagnostics, and packet gates.

## Upstream references (source of truth)

- Docs page: `repo-ref/ui/apps/v4/content/docs/components/textarea.mdx`
- Component implementation: `repo-ref/ui/apps/v4/registry/new-york-v4/ui/textarea.tsx`
- New York examples: `repo-ref/ui/apps/v4/registry/new-york-v4/examples/textarea-demo.tsx`, `repo-ref/ui/apps/v4/registry/new-york-v4/examples/textarea-disabled.tsx`, `repo-ref/ui/apps/v4/registry/new-york-v4/examples/textarea-with-label.tsx`, `repo-ref/ui/apps/v4/registry/new-york-v4/examples/textarea-with-text.tsx`, `repo-ref/ui/apps/v4/registry/new-york-v4/examples/textarea-with-button.tsx`
- Base/radix follow-up examples: `repo-ref/ui/apps/v4/registry/bases/base/ui/textarea.tsx`, `repo-ref/ui/apps/v4/registry/bases/base/examples/textarea-example.tsx`, `repo-ref/ui/apps/v4/registry/bases/radix/ui/textarea.tsx`, `repo-ref/ui/apps/v4/registry/bases/radix/examples/textarea-example.tsx`
- Existing goldens: `goldens/shadcn-web/v4/new-york-v4/textarea-demo.json`, `goldens/shadcn-web/v4/new-york-v4/textarea-demo.invalid.json`, `goldens/shadcn-web/v4/new-york-v4/textarea-demo.focus.json`, `goldens/shadcn-web/v4/new-york-v4/textarea-demo.invalid-focus.json`, `goldens/shadcn-web/v4/new-york-v4/textarea-disabled.json`, `goldens/shadcn-web/v4/new-york-v4/textarea-with-label.json`, `goldens/shadcn-web/v4/new-york-v4/textarea-with-text.json`, `goldens/shadcn-web/v4/new-york-v4/textarea-with-button.json`

## Fret implementation

- Component code: `ecosystem/fret-ui-shadcn/src/textarea.rs`
- Gallery page: `apps/fret-ui-gallery/src/ui/pages/textarea.rs`
- Matrix packet: `docs/workstreams/shadcn-component-parity-matrix-v1/artifacts/textarea_agent_packet_p0_v1.json`

## Audit checklist

### Authoring surface

- Pass: `Textarea::new(model)` covers the documented upstream `<Textarea />` leaf-control path.
- Pass: `placeholder(...)`, `disabled(...)`, `aria_invalid(...)`, `required(...)`, `min_height(...)`, `rows(...)`, and `control_id(...)` cover the practical control-level surface exposed by the current docs and examples.
- Pass: `Label::for_control(...)` plus `Textarea::control_id(...)` maps the current upstream `htmlFor` / `id` examples without widening `Textarea` itself.
- Pass: `Field::build(...)` remains the focused Fret/base-radix follow-up lane for label/description association; the base/radix example places `Message`, a six-row textarea, and description text after the control.
- Pass: `Textarea` is a leaf text control, and neither `repo-ref/primitives` nor `repo-ref/base-ui` defines a dedicated textarea compound primitive, so Fret intentionally does not add a generic `compose()` / `asChild` / children builder here.

### Layout and default-style ownership

- Pass: root `w-full min-w-0`, control chrome, minimum height, text style, focus-visible ring, invalid border/ring, and resize behavior remain recipe-owned.
- Pass: surrounding width caps, stacked button layout, helper-text placement, Field layout, and visible required markers remain caller-owned.
- Pass: default minimum height matches the upstream `min-h-16` outcome (64px).
- Pass: `rows(...)` raises the initial minimum height when the caller wants a taller starting textarea, while preserving the recipe-owned 64px floor for default and one-row cases.
- Pass: the resize handle stays pointer-only, remains hidden from the visible accessibility tree, captures drag, and clamps the height back to the minimum floor.

### Semantics

- Pass: exposes `SemanticsRole::TextField` and supports explicit `a11y_label`.
- Pass: required and invalid states land on the concrete textarea control.
- Pass: control registry integration supports label/described-by wiring via `control_id(...)`.
- Pass: the dedicated label-focus gate verifies label click forwarding and focus-visible behavior.

### Gallery / docs parity

- Pass: the gallery now mirrors the current Textarea docs path first after collapsing the top `ComponentPreview` / `Default` duplicate into `Demo`: `Demo`, `Usage`, `Disabled`, `With Label`, `With Text`, and `With Button`.
- Pass: `API Reference`, `Invalid`, `Required`, `RTL`, `Field`, and `Label Association` stay as explicit state-depth or Fret/base-radix follow-ups.
- Pass: the `With Label` snippet maps the current upstream label example onto `Label::for_control(...)` plus `Textarea::control_id(...)`.
- Pass: the `Field` snippet now follows the base/radix `Message` + six-row textarea + description-after-control example, while `RTL` keeps the translated feedback composition.
- Pass: this work is docs/public-surface parity and recipe-layer harness hardening, not a new `fret-ui` mechanism fix.

## Validation

- `python -m json.tool docs/workstreams/shadcn-component-parity-matrix-v1/artifacts/textarea_agent_packet_p0_v1.json | Out-Null`
- `Get-ChildItem -Path tools/diag-scripts/ui-gallery/textarea -Filter *.json | ForEach-Object { python -m json.tool $_.FullName | Out-Null }`
- `cargo nextest run -p fret-ui-shadcn --lib --status-level fail textarea`
- `cargo nextest run -p fret-ui-shadcn --features web-goldens --test web_vs_fret_layout --status-level fail textarea`
- `cargo nextest run -p fret-ui-shadcn --features web-goldens --test web_vs_fret_control_chrome --status-level fail textarea`
- `cargo nextest run -p fret-ui-shadcn --test web_vs_fret_textarea --status-level fail`
- `cargo nextest run -p fret-ui-shadcn --test textarea_label_focus --status-level fail`
- `cargo nextest run -p fret-ui-gallery --test textarea_docs_surface --status-level fail`
- `cargo nextest run -p fret-ui-gallery --test ui_authoring_surface_default_app --status-level fail textarea`
