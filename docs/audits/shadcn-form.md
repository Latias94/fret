# shadcn/ui v4 Audit - Form

## Upstream references (non-normative)

This document references optional local checkouts under `repo-ref/` for convenience.
Upstream sources:

- shadcn/ui: https://github.com/shadcn-ui/ui
- react-hook-form: https://react-hook-form.com/

See `docs/repo-ref.md` for the optional local snapshot policy and pinned SHAs.
This audit compares Fret's shadcn-aligned `Form` taxonomy against the upstream shadcn/ui v4 form
composition model.

## Upstream references (source of truth)

- Docs page: `repo-ref/ui/apps/v4/content/docs/components/form.mdx`
- Upstream demo: `repo-ref/ui/apps/v4/app/(internal)/sink/components/form-demo.tsx`

## Fret implementation

- Component facade: `ecosystem/fret-ui-shadcn/src/form.rs`
- Form field helper: `ecosystem/fret-ui-shadcn/src/form_field.rs`
- Field primitives: `ecosystem/fret-ui-shadcn/src/field.rs`
- Gallery page: `apps/fret-ui-gallery/src/ui/pages/form.rs`
- Copyable usage snippet: `apps/fret-ui-gallery/src/ui/snippets/form/usage.rs`

## Audit checklist

### Authoring surface

- Pass: `Form` / `FormItem` / `FormLabel` / `FormControl` / `FormDescription` / `FormMessage` are all
  exposed as shadcn-aligned taxonomy aliases over Fret field primitives.
- Pass: `FormField::new(form_state, id, control)` covers the common helper path for wiring label,
  description, control decoration, and error visibility around one field.
- Pass: The API is intentionally framework-agnostic: upstream RHF-specific render-prop patterns are
  translated into `FormState` + direct model-bound controls instead of mirroring `react-hook-form` literally.
- Note: Because this surface is alias/helper-oriented rather than a nested slot tree, it does not need a
  generic `compose()` builder or a children-driven authoring API beyond the existing `Form`/`FieldSet` composition.

### Composition model

- Pass: `Form` maps to a vertical `FieldSet` container.
- Pass: `FormItem` maps to `Field`.
- Pass: `FormControl` maps to `FieldContent`.
- Pass: `FormMessage` maps to `FieldError`.
- Pass: `FormField` can decorate common controls with invalid styling and labels based on `FormState`.

## Conclusion

- Result: This component does not currently indicate a missing mechanism-layer gap in the shadcn-facing surface.
- Result: The main missing piece for docs parity was a concise gallery `Usage` example and an audit note clarifying the framework-agnostic mapping.
- Result: Follow-up work should focus on richer validation/resolver recipes only if a concrete product need appears.

## Validation

- `cargo check -p fret-ui-gallery --message-format short`
