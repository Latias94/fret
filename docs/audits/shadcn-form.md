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

There is no single `components/form.mdx` page in the current v4 repo snapshot. Use these sources
instead:

- Component source: `repo-ref/ui/apps/v4/registry/new-york-v4/ui/form.tsx`
- Upstream demo: `repo-ref/ui/apps/v4/app/(internal)/sink/components/form-demo.tsx`
- Related docs: `repo-ref/ui/apps/v4/content/docs/forms/react-hook-form.mdx`
- Related docs: `repo-ref/ui/apps/v4/content/docs/forms/tanstack-form.mdx`

## Fret implementation

- Component facade: `ecosystem/fret-ui-shadcn/src/form.rs`
- Form field helper: `ecosystem/fret-ui-shadcn/src/form_field.rs`
- Field primitives: `ecosystem/fret-ui-shadcn/src/field.rs`
- Gallery page: `apps/fret-ui-gallery/src/ui/pages/form.rs`
- Copyable usage snippet: `apps/fret-ui-gallery/src/ui/snippets/form/usage.rs`

## Audit checklist

### Authoring surface

- Pass: `Form` / `FormItem` / `FormLabel` / `FormControl` / `FormDescription` / `FormMessage`
  remain available under the expected shadcn taxonomy.
- Pass: `FormField::new(form_state, id, control)` still covers the common Fret-native helper path
  for wiring label, description, control decoration, and error visibility around one field.
- Pass: The API remains intentionally framework-agnostic: upstream RHF-specific render-prop
  patterns are translated into `FormState` + direct model-bound controls instead of mirroring
  `react-hook-form` literally.
- Pass: `FormControl` now approximates upstream `Slot.Root` ownership more closely: a single child
  passes through unchanged, so the form surface no longer injects `FieldContent`'s `flex-1`
  / `min-w-0` / `gap-1.5` defaults into every control.
- Note: Multi-child `FormControl::new([...])` remains supported as a compatibility escape hatch, but
  it now falls back to a zero-gap column without fill defaults. Upstream-style compositions should
  prefer passing a single control root (or wrapping multiple controls explicitly at the call site).

### Composition model

- Pass: `Form` maps to a vertical `FieldSet` container.
- Pass: `FormItem` maps to `Field`.
- Pass: `FormMessage` maps to `FieldError`.
- Pass: `FormField` can decorate common controls with invalid styling and labels based on
  `FormState`.
- Adjusted: `FormControl` is no longer treated as a `FieldContent` alias. This was a public-surface
  drift, because upstream `FormControl` is a slot-like control wrapper rather than a layout column.

## Conclusion

- Result: The main drift was public-surface ownership, not a mechanism-layer gap.
- Result: `FormControl` now keeps layout negotiation caller-owned by default, which matches shadcn
  docs/examples more closely and avoids repeating the `card`/`field` width footgun at the form
  surface.
- Result: Follow-up work should focus on richer resolver/validation recipes only if a concrete
  product need appears; the core composition surface no longer forces `FieldContent` defaults.

## Validation

- `cargo nextest run -p fret-ui-shadcn --lib form_control_is_slot_like_for_single_child form_control_multi_child_fallback_drops_field_content_fill_defaults --status-level fail`
