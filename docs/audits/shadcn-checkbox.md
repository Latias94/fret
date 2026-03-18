# shadcn/ui v4 Audit — Checkbox


## Upstream references (non-normative)

This document references optional local checkouts under `repo-ref/` for convenience.
Upstream sources:

- shadcn/ui: https://github.com/shadcn-ui/ui

See `docs/repo-ref.md` for the optional local snapshot policy and pinned SHAs.
This audit compares Fret's shadcn-aligned `Checkbox` against the upstream shadcn/ui v4 docs and the
base implementation in `repo-ref/ui`.

## Upstream references (source of truth)

- Docs page: `repo-ref/ui/apps/v4/content/docs/components/base/checkbox.mdx`
- Component implementation: `repo-ref/ui/apps/v4/examples/base/ui/checkbox.tsx`
- Example compositions: `repo-ref/ui/apps/v4/examples/base/checkbox-demo.tsx`, `repo-ref/ui/apps/v4/examples/base/checkbox-basic.tsx`, `repo-ref/ui/apps/v4/examples/base/checkbox-description.tsx`, `repo-ref/ui/apps/v4/examples/base/checkbox-disabled.tsx`, `repo-ref/ui/apps/v4/examples/base/checkbox-group.tsx`, `repo-ref/ui/apps/v4/examples/base/checkbox-invalid.tsx`, `repo-ref/ui/apps/v4/examples/base/checkbox-table.tsx`, `repo-ref/ui/apps/v4/examples/base/checkbox-rtl.tsx`
- Underlying primitive: Base UI `@base-ui/react/checkbox`

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

- Pass: the gallery now mirrors the upstream docs path first: `Demo`, `Usage`, `Checked State`, `Invalid State`, `Basic`, `Description`, `Disabled`, `Group`, `Table`, `RTL`, and `API Reference`.
- Pass: `Checked State` now teaches both the model-backed path and the narrower
  `Checkbox::from_checked(...)` + `.action(...)` snapshot/action path directly in the copyable
  snippet instead of burying that guidance only in prose.
- Pass: `Description` now matches the upstream row order (`Checkbox` first, `FieldContent` second)
  instead of teaching a reversed layout.
- Pass: `Group` now restores the upstream `FieldSet` / `FieldLegend` / `FieldDescription` framing
  before the checkbox list instead of collapsing the example into an unrelated list layout.
- Pass: `Table` now teaches a derived select-all checkbox with mixed-state behavior on the same
  action-first snapshot path, which is the important source-aligned authoring story for checkbox
  collections.
- Pass: `Label Association` and `With Title` remain as explicit Fret-only follow-ups after the upstream path because they document field/label composition rather than the base checkbox recipe itself.
- Pass: the remaining parity work for this component is page/docs clarity; no extra generic children or `compose()` API is warranted.

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

## Follow-ups (recommended)

- Pass: Snapshot/action checkboxes now participate in `control_id` / label forwarding without falling back to a model-backed registry entry; label activation mirrors command dispatch, payload forwarding, and state toggles when applicable.
- Pass: Supports Radix `checked="indeterminate"` (tri-state) via `Checkbox::new_tristate`.
  - Note: Semantics currently maps indeterminate to `checked: None`.
