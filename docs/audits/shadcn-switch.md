# shadcn/ui v4 Audit - Switch

## Upstream references (non-normative)

This document references optional local checkouts under `repo-ref/` for convenience.
Upstream sources:

- shadcn/ui: https://github.com/shadcn-ui/ui

See `docs/repo-ref.md` for the optional local snapshot policy and pinned SHAs.
This audit compares Fret's shadcn-aligned `Switch` against the current main-worktree
`repo-ref/` snapshot, the new-york-v4 source, secondary Field/Form and Base/Radix registry
examples, and the existing switch web gates.

## Upstream references (source of truth)

- Current docs page: `repo-ref/ui/apps/v4/content/docs/components/switch.mdx`
  - Current docs path exposes the top `switch-demo` preview and `Usage` only.
  - There is no current `content/docs/components/base/switch.mdx` or
    `content/docs/components/radix/switch.mdx` page in the audited snapshot.
- shadcn recipe source: `repo-ref/ui/apps/v4/registry/new-york-v4/ui/switch.tsx`
- Current docs example: `repo-ref/ui/apps/v4/registry/new-york-v4/examples/switch-demo.tsx`
- Secondary new-york-v4 registry examples:
  - `repo-ref/ui/apps/v4/registry/new-york-v4/examples/field-switch.tsx`
  - `repo-ref/ui/apps/v4/registry/new-york-v4/examples/form-rhf-switch.tsx`
  - `repo-ref/ui/apps/v4/registry/new-york-v4/examples/form-tanstack-switch.tsx`
- Secondary Base/Radix registry references:
  - `repo-ref/ui/apps/v4/registry/bases/base/ui/switch.tsx`
  - `repo-ref/ui/apps/v4/registry/bases/base/examples/switch-example.tsx`
  - `repo-ref/ui/apps/v4/registry/bases/radix/ui/switch.tsx`
  - `repo-ref/ui/apps/v4/registry/bases/radix/examples/switch-example.tsx`
- Radix primitive source: `repo-ref/primitives/packages/react/switch/src/switch.tsx`
- Base UI anatomy references:
  - `repo-ref/base-ui/packages/react/src/switch/root/SwitchRoot.tsx`
  - `repo-ref/base-ui/packages/react/src/switch/thumb/SwitchThumb.tsx`
- Existing upstream goldens:
  - `goldens/shadcn-web/v4/new-york-v4/switch-demo.json`
  - `goldens/shadcn-web/v4/new-york-v4/switch-demo.focus.json`
  - `goldens/shadcn-web/v4/new-york-v4/field-switch.json`
  - `goldens/shadcn-web/v4/new-york-v4/form-rhf-switch.json`
  - `goldens/shadcn-web/v4/new-york-v4/form-tanstack-switch.json`

## Fret implementation

- Primitive policy: `ecosystem/fret-ui-kit/src/primitives/switch.rs`
- Component code: `ecosystem/fret-ui-shadcn/src/switch.rs`
- Web-vs-Fret layout gates:
  - `ecosystem/fret-ui-shadcn/tests/web_vs_fret_layout/switch.rs`
  - `ecosystem/fret-ui-shadcn/tests/web_vs_fret_layout/field.rs`
  - `ecosystem/fret-ui-shadcn/tests/web_vs_fret_layout/form.rs`
- Chrome and state gates:
  - `ecosystem/fret-ui-shadcn/tests/web_vs_fret_control_chrome.rs`
  - `ecosystem/fret-ui-shadcn/tests/radix_web_primitives_state.rs`
- Gallery page: `apps/fret-ui-gallery/src/ui/pages/switch.rs`
- Gallery docs tests:
  - `apps/fret-ui-gallery/tests/switch_docs_surface.rs`
  - `apps/fret-ui-gallery/tests/ui_authoring_surface_default_app.rs`
- Diagnostics: `tools/diag-scripts/ui-gallery/switch/*.json`
- Matrix packet: `docs/workstreams/shadcn-component-parity-matrix-v1/artifacts/switch_agent_packet_p0_v1.json`

## Audit checklist

### Authoring surface

- Pass: `Switch::new(model)` plus `size(...)`, `disabled(...)`, `required(...)`,
  `aria_invalid(...)`, `read_only(...)`, `control_id(...)`, `a11y_label(...)`, and command/action
  hooks cover the current docs and registry example surface.
- Pass: `Label::for_control(...)` and `FieldLabel::for_control(...)` cover the current
  `htmlFor`/`id` label-binding path across the docs demo, size rows, and field-based examples.
- Pass: `Switch::from_checked(...)` and `action(...)` / `action_payload(...)` remain available for
  action-first authoring without forcing a `Model<bool>` at every call site.
- Pass: Field/Form composition stays caller-owned. `Field`, `FieldContent`, `FieldLabel`, and
  `FieldDescription` cover the secondary registry examples without widening `Switch` into a generic
  children API.
- Pass: Radix/Base UI `Root` / `Thumb` anatomy remains a lower-level mechanism/headless reference.
  It does not require widening the copyable shadcn `Switch` surface unless a real custom-parts
  product case appears.
- Pass: `Switch` remains a leaf control; no extra generic `compose()` / `asChild` surface is needed
  for the current docs-path slice.

### Layout & default-style ownership

- Pass: track/thumb chrome, focus ring, disabled opacity, hover/active paint, checked/unchecked
  colors, and intrinsic switch sizes remain recipe-owned because the upstream switch source defines
  those defaults on the component itself.
- Pass: surrounding width caps, field/card stacking, and page/grid negotiation remain caller-owned
  and stay on gallery/example compositions.
- Pass: the docs demo mirrors the current upstream `Switch id="airplane-mode"` plus
  `Label htmlFor="airplane-mode"` row through `ControlId` and `Label::for_control(...)`.
- Pass: Field/Form switch goldens keep secondary registry composition honest without reclassifying
  those examples as current docs-path sections.
- Note: `SwitchStyle` remains a focused Fret follow-up for token-safe color overrides rather than
  part of the upstream docs path.

### Gallery / docs parity

- Pass: the gallery now mirrors the current shadcn Switch docs path first with `Demo` and `Usage`.
- Pass: `Description (Registry)` and `Invalid (Registry)` are labeled as current new-york-v4
  registry follow-ups rather than current docs-page sections.
- Pass: `Disabled (Base/Radix)` and `Size (Base/Radix)` are labeled as secondary Base/Radix
  registry references.
- Pass: `Choice Card (Fret)`, `Read Only (Fret)`, `Command Gate (Fret)`, `RTL (Fret)`,
  `Label Association (Fret)`, `Style Override (Fret)`, and `API Reference (Fret)` remain explicit
  Fret teaching or diagnostics follow-ups.
- Pass: the page notes record that the current copyable shadcn lane is a leaf-control API; custom
  `Root`/`Thumb` anatomy should live in a lower-level primitive/raw surface if needed later.

## Validation

- `python -m json.tool docs/workstreams/shadcn-component-parity-matrix-v1/artifacts/switch_agent_packet_p0_v1.json | Out-Null`
- `Get-ChildItem -Path tools/diag-scripts/ui-gallery/switch -Filter *.json | ForEach-Object { python -m json.tool $_.FullName | Out-Null }`
- `python -m json.tool tools/parity-discovery/manifests/shadcn_parity_coverage_v2.json | Out-Null`
- `python -m json.tool docs/workstreams/shadcn-component-parity-matrix-v1/artifacts/shadcn_component_harness_matrix_v1.json | Out-Null`
- `python -m json.tool docs/workstreams/shadcn-component-parity-matrix-v1/WORKSTREAM.json | Out-Null`
- `python -m py_compile tools/parity-discovery/shadcn_component_harness_matrix.py`
- `rustfmt --edition 2024 --check apps/fret-ui-gallery/src/ui/pages/switch.rs apps/fret-ui-gallery/tests/switch_docs_surface.rs apps/fret-ui-gallery/tests/ui_authoring_surface_default_app.rs ecosystem/fret-ui-shadcn/src/switch.rs`
- `cargo nextest run -p fret-ui-kit --lib --status-level fail switch`
- `cargo nextest run -p fret-ui-shadcn --lib --status-level fail switch`
- `cargo nextest run -p fret-ui-shadcn --features web-goldens --test web_vs_fret_layout --status-level fail switch`
- `cargo nextest run -p fret-ui-shadcn --features web-goldens --test web_vs_fret_layout --status-level fail web_vs_fret_layout_field_geometry_matches_web_fixtures`
- `cargo nextest run -p fret-ui-shadcn --features web-goldens --test web_vs_fret_layout --status-level fail form_rhf_switch`
- `cargo nextest run -p fret-ui-shadcn --features web-goldens --test web_vs_fret_layout --status-level fail form_tanstack_switch`
- `cargo nextest run -p fret-ui-shadcn --features web-goldens --test web_vs_fret_control_chrome --status-level fail switch`
- `cargo nextest run -p fret-ui-shadcn --test radix_web_primitives_state --status-level fail switch`
- `cargo nextest run -p fret-ui-gallery --test switch_docs_surface --status-level fail`
- `cargo nextest run -p fret-ui-gallery --test ui_authoring_surface_default_app --status-level fail switch`
- `python tools/parity-discovery/shadcn_component_harness_matrix.py`
- `python tools/check_workstream_catalog.py`
- `git diff --check`
