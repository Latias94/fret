# shadcn/ui v4 Audit — Checkbox


## Upstream references (non-normative)

This document references optional local checkouts under `repo-ref/` for convenience.
Upstream sources:

- shadcn/ui: https://github.com/shadcn-ui/ui

See `docs/repo-ref.md` for the optional local snapshot policy and pinned SHAs.
This audit compares Fret's shadcn-aligned `Checkbox` against the current upstream shadcn/ui v4 docs
and new-york-v4 registry implementation in `repo-ref/ui`.

## Upstream references (source of truth)

- Docs page: `repo-ref/ui/apps/v4/content/docs/components/checkbox.mdx`
- Component implementation: `repo-ref/ui/apps/v4/registry/new-york-v4/ui/checkbox.tsx`
- Example compositions: `repo-ref/ui/apps/v4/registry/new-york-v4/examples/checkbox-demo.tsx`, `repo-ref/ui/apps/v4/registry/new-york-v4/examples/checkbox-with-text.tsx`, `repo-ref/ui/apps/v4/registry/new-york-v4/examples/checkbox-disabled.tsx`, `repo-ref/ui/apps/v4/registry/new-york-v4/examples/field-checkbox.tsx`, `repo-ref/ui/apps/v4/registry/new-york-v4/examples/form-rhf-checkbox.tsx`, `repo-ref/ui/apps/v4/registry/new-york-v4/examples/form-tanstack-checkbox.tsx`
- Underlying primitive: Radix `@radix-ui/react-checkbox`

## Fret implementation

- Component code: `ecosystem/fret-ui-shadcn/src/checkbox.rs`
- Shared primitives:
  - Radix checkbox outcomes: `ecosystem/fret-ui-kit/src/primitives/checkbox.rs`
  - Focus ring recipe: `ecosystem/fret-ui-kit/src/declarative/style.rs`
  - Control chrome composition: `ecosystem/fret-ui-kit/src/declarative/chrome.rs`

## Audit checklist

### Interaction

- Pass: Click toggles the bound `Model<bool>`.
- Pass: Source-aligned snapshot/action authoring exists via `Checkbox::from_checked(...)` /
  `from_checked_state(...)` plus `.action(...)`, while `.on_click(...)` remains the lower-level
  command bridge when explicit command routing is actually desired, so the recipe is not forced into
  per-row `Model<bool>` ownership for every copyable example.
- Note: `Checkbox` is a leaf control surface, so Fret intentionally does not add a generic
  `compose()` builder here; the direct control API already matches the important contract.
- Pass: Supports optional state via `Checkbox::new_optional(Model<Option<bool>>)` where `None` maps
  to indeterminate (Radix outcome), and click toggles to `Some(true)`.
- Pass: Disabled state blocks interaction and applies reduced opacity.

### Semantics

- Pass: Exposes `SemanticsRole::Checkbox` and `checked` state.

### Gallery / docs parity

- Pass: `Demo` now mirrors the upstream `checkbox-demo.tsx` teaching surface by keeping the four-row `FieldGroup` composite preview (`Label`, description, disabled, and wrapped title/content) instead of collapsing the first section into a single row.
- Pass: the gallery starts with the current upstream docs path (`Demo`, `Usage`) before keeping
  checked/invalid state, registry-shaped composition, table, RTL, and API reference follow-ups
  explicit.
- Pass: `Checked State` now teaches both the model-backed path and the narrower
  `Checkbox::from_checked(...)` + `.action(...)` snapshot/action path directly in the copyable
  snippet instead of burying that guidance only in prose.
- Pass: `Description` keeps the registry row order (`Checkbox` first, `FieldContent` second) instead
  of teaching a reversed layout.
- Pass: `Group` keeps the current field-registry `FieldSet` / `FieldLegend` / `FieldDescription`
  framing before the checkbox list instead of collapsing the example into an unrelated list layout.
- Pass: `Table` now teaches a derived select-all checkbox with mixed-state behavior on the same
  action-first snapshot path, which is the important source-aligned authoring story for checkbox
  collections.
- Pass: `Label Association` and `With Title` remain as explicit Fret-only follow-ups after the upstream path because they document field/label composition rather than the base checkbox recipe itself.
- Pass: the remaining parity work for this component is page/docs clarity; no extra generic children or `compose()` API is warranted.
- Pass: the dedicated Gallery docs-surface gate now locks the page order through `API Reference`
  before the Fret-only follow-ups, keeps the snippets on curated `Checkbox` / `Field*` surfaces
  without raw/advanced exposure, and anchors the disabled, required-disabled, mixed-state table,
  label-click, and RTL diagnostics.

### Visual parity (new-york)

- Pass: Unchecked state uses `border-input` and transparent background.
- Pass: Checked state uses `primary` background, `primary-foreground` indicator color, and `primary`
  border.
- Pass: Uses `shadow_xs`, matching shadcn’s `shadow-xs` default.
- Pass: Focus ring thickness (`ring-[3px]`) matches shadcn-web focus variant (`checkbox-demo.focus`).

## Validation

- `cargo test -p fret-ui-shadcn --lib checkbox`
- `cargo test -p fret-ui-shadcn --lib field_label_click_mirrors_checkbox_action_sequence --message-format short`
- Web layout gate: `cargo nextest run -p fret-ui-shadcn --test web_vs_fret_layout`
  (`web_vs_fret_layout_checkbox_demo_control_size`).
- Focus ring gate: `cargo nextest run -p fret-ui-shadcn --test web_vs_fret_control_chrome`
  (`web_vs_fret_checkbox_demo_focus_ring_matches`).
- Gallery docs-surface gate:
  `cargo nextest run -p fret-ui-gallery --test checkbox_docs_surface`.
- Gallery default-app authoring gate:
  `cargo nextest run -p fret-ui-gallery --test ui_authoring_surface_default_app checkbox --no-fail-fast`.

## Follow-ups (recommended)

- Pass: Snapshot/action checkboxes now participate in `control_id` / label forwarding without falling back to a model-backed registry entry; label activation mirrors command dispatch, payload forwarding, and state toggles when applicable.
- Pass: Supports Radix `checked="indeterminate"` (tri-state) via `Checkbox::new_tristate`.
  - Note: Semantics currently maps indeterminate to `checked: None`.
