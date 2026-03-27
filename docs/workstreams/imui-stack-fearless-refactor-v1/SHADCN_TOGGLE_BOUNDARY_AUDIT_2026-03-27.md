# imui vs shadcn Toggle Boundary Audit — 2026-03-27

Status: boundary audit note
Last updated: 2026-03-27

Related:

- `docs/workstreams/imui-stack-fearless-refactor-v1/DESIGN.md`
- `docs/shadcn-declarative-progress.md`
- `docs/workstreams/authoring-surface-and-ecosystem-fearless-refactor-v1/SHADCN_RAW_MODEL_ALLOWLIST_AUDIT_2026-03-19.md`
- `ecosystem/fret-ui-shadcn/src/surface_policy_tests.rs`

## Why this note exists

The `imui` fearless refactor deleted `toggle_model(...)` from the immediate-mode boolean control
lane and kept `switch_model(...)` as the canonical surface.

At the same time, `fret-ui-shadcn` still exposes builder methods such as
`Button::toggle_model(...)` and `InputGroupButton::toggle_model(...)`.

That naming overlap can look like unfinished cleanup unless the ownership split is recorded
explicitly.

## Current conclusion

The overlap is intentional because the two surfaces own different concepts:

- `imui::switch_model(...)` is a boolean field control.
- `shadcn::Button::toggle_model(...)` is an activation-policy hook on a trigger/control surface.

The right outcome is therefore:

- keep `imui` on `switch_model(...)` / `switch_model_with_options(...)`,
- keep shadcn trigger/button surfaces on `toggle_model(...)`,
- and avoid renaming shadcn trigger hooks to `switch_model(...)`, which would mis-teach them as
  form-field surfaces rather than activation policy.

## Axis-by-axis split

### `imui` lane

Layer:

- `ecosystem/fret-ui-kit::imui`

Concept:

- immediate-mode boolean form control

Canonical verbs:

- `switch_model(...)`
- `switch_model_with_options(...)`

Representative evidence:

- `ecosystem/fret-ui-kit/src/imui.rs`
- `ecosystem/fret-imui/src/tests/models.rs`

### shadcn lane

Layer:

- `ecosystem/fret-ui-shadcn`

Concept:

- component-owned activation policy for externally owned open/active state

Canonical verbs:

- `Button::toggle_model(...)`
- `InputGroupButton::toggle_model(...)`

Representative evidence:

- `ecosystem/fret-ui-shadcn/src/button.rs`
- `ecosystem/fret-ui-shadcn/src/input_group.rs`
- `apps/fret-ui-gallery/src/ui/snippets/dialog/demo.rs`
- `apps/fret-ui-gallery/src/ui/snippets/button_group/demo.rs`

## Why shadcn keeps `toggle_model(...)`

The builder is attached to a trigger-like component, not a form field:

- the button still owns click/activate semantics first,
- the model flip is only one possible activation-side effect,
- first-party callers mostly use it for dialog/popover/drawer/sheet/date-picker open state and
  similar trigger choreography,
- and source-policy tests already treat it as an explicit allowlisted seam instead of a default
  model-first lane.

Implementation evidence:

- `ecosystem/fret-ui-shadcn/src/button.rs` uses `cx.pressable_toggle_bool(&model)` during
  activation wiring,
- `ecosystem/fret-ui-shadcn/src/input_group.rs` mirrors the same trigger policy,
- `ecosystem/fret-ui-shadcn/src/surface_policy_tests.rs` keeps both signatures on the audited
  allowlist,
- `apps/fret-ui-gallery/tests/ui_authoring_surface_default_app.rs` still accepts
  `.toggle_model(...)` on curated trigger snippets.

## Maintainer rule

If a future surface review sees `toggle_model(...)`, ask which concept it owns:

1. If it is a boolean field/editor in `imui`, keep or migrate it to `switch_model(...)`.
2. If it is a trigger/action component flipping externally owned state, `toggle_model(...)` is
   still acceptable on the component layer.
3. Do not use the shadcn trigger hook as justification for reopening deleted `imui`
   compatibility names.

## Landable follow-up posture

- Keep the current shadcn trigger hook names.
- Keep guarding them through `surface_policy_tests.rs` and UI Gallery source-policy tests.
- Route any future pressure for denser trigger semantics to shadcn/component docs or narrower
  builder hooks, not back into the `imui` boolean field naming lane.
