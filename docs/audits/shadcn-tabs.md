# shadcn/ui v4 Audit - Tabs

## Upstream references (non-normative)

This document references optional local checkouts under `repo-ref/` for convenience.
Upstream sources:

- shadcn/ui: https://github.com/shadcn-ui/ui

See `docs/repo-ref.md` for the optional local snapshot policy and pinned SHAs.
This audit compares Fret's shadcn-aligned `Tabs` against the current main-worktree `repo-ref/`
snapshot, the new-york-v4 source, secondary Base/Radix registry examples, and existing Tabs gates.

## Upstream references (source of truth)

- Current docs page: `repo-ref/ui/apps/v4/content/docs/components/tabs.mdx`
  - Current docs path exposes the top `tabs-demo` preview and `Usage` only.
  - There is no current `content/docs/components/base/tabs.mdx` or
    `content/docs/components/radix/tabs.mdx` page in the audited snapshot.
- shadcn recipe source: `repo-ref/ui/apps/v4/registry/new-york-v4/ui/tabs.tsx`
- Current docs example: `repo-ref/ui/apps/v4/registry/new-york-v4/examples/tabs-demo.tsx`
- Secondary Base/Radix registry references:
  - `repo-ref/ui/apps/v4/registry/bases/base/ui/tabs.tsx`
  - `repo-ref/ui/apps/v4/registry/bases/base/examples/tabs-example.tsx`
  - `repo-ref/ui/apps/v4/registry/bases/radix/ui/tabs.tsx`
  - `repo-ref/ui/apps/v4/registry/bases/radix/examples/tabs-example.tsx`
- Radix primitive source: `repo-ref/primitives/packages/react/tabs/src/tabs.tsx`
- Base UI root contract: `repo-ref/base-ui/packages/react/src/tabs/root/TabsRoot.tsx`
- Existing upstream golden: `goldens/shadcn-web/v4/new-york-v4/tabs-demo.json`

## Fret implementation anchors

- Component code: `ecosystem/fret-ui-shadcn/src/tabs.rs`
- Primitive semantics: `ecosystem/fret-ui-kit/src/primitives/tabs.rs`
- Keyboard gates: `ecosystem/fret-ui-shadcn/tests/tabs_keyboard_navigation.rs`
- Web-vs-Fret layout gates: `ecosystem/fret-ui-shadcn/tests/web_vs_fret_layout/tabs.rs`
- Radix primitive state proof: `ecosystem/fret-ui-shadcn/tests/radix_web_primitives_state.rs`
- Gallery page: `apps/fret-ui-gallery/src/ui/pages/tabs.rs`
- Gallery docs tests:
  - `apps/fret-ui-gallery/tests/tabs_docs_surface.rs`
  - `apps/fret-ui-gallery/tests/ui_authoring_surface_default_app.rs`
- Diagnostics: `tools/diag-scripts/ui-gallery/tabs/*.json`
- Matrix packet: `docs/workstreams/shadcn-component-parity-matrix-v1/artifacts/tabs_agent_packet_p0_v1.json`

## Audit checklist

### Composition & control model

- Pass: Exposes composable `TabsRoot` / `TabsList` / `TabsTrigger` / `TabsContent`, and keeps
  `Tabs` + `TabsItem` recipe builder for convenience.
- Pass: Fret intentionally does not add a separate generic `compose()` builder for `Tabs` because
  the composable part surface already matches upstream nested authoring directly; the builder is a
  convenience layer, not a contract gap.
- Pass: Supports controlled selection via `Model<Option<Arc<str>>>` and uncontrolled
  `default_value`.
- Pass: Aligns with Base UI `onValueChange` intent via `Tabs::on_value_change(...)` and
  `TabsRoot::on_value_change(...)`.
- Pass: Source-aware callbacks expose change origin and activation direction through
  `Tabs::on_value_change_with_source(...)`, `TabsRoot::on_value_change_with_source(...)`, and
  event-details variants.
- Pass: Event-details callbacks can cancel pending pointer and roving-active selection writes.

### Keyboard & selection semantics

- Pass: Arrow roving + APG navigation are wired through `RovingFlex` + `cx.roving_nav_apg()`.
- Pass: `TabsActivationMode::Automatic` and `TabsActivationMode::Manual` map to expected behavior.
- Pass: `TabsOrientation::{Horizontal, Vertical}` and looping navigation are supported.
- Pass: `force_mount_content(true)` preserves inactive panels while gating layout/paint/semantics.
- Pass: Radix click-state proof covers click-to-select semantics against the primitive timeline.
- Pass: Gallery diagnostics cover selected state mutation, panel relation edges, keyboard roving,
  RTL key navigation, and stable panel test ids.

### Visual defaults (new-york-v4 parity)

- Pass: Root/list/trigger/content tokens align with current new-york-v4 defaults: root
  `flex flex-col gap-2`, list `inline-flex h-9 w-fit rounded-lg p-[3px]`, trigger active
  background/border/shadow, disabled opacity, focus-visible ring, trigger text metrics, and content
  `flex-1 outline-none`.
- Pass: Default-style ownership remains split on purpose: root width constraints such as upstream
  `className="w-[400px]"` and the demo shell `w-full max-w-sm` stay caller-owned, while
  list/trigger/content chrome remains recipe-owned.
- Pass: Web-vs-Fret layout gates cover tab-list height, active trigger height/inset, active/inactive
  trigger text paint, and the panel gap against `tabs-demo`.
- Pass: Trigger content remains rich through `TabsTrigger::children(...)` and
  `TabsItem::trigger_children(...)`.

### Docs surface & composable authoring

- Pass: the UI Gallery page now mirrors the current shadcn Tabs docs path first with `Demo` and
  `Usage`.
- Pass: `Line (Base/Radix)`, `Vertical (Base/Radix)`, `Disabled (Base/Radix)`, `Icons (Base/Radix)`,
  and `List (Base/Radix)` are labeled as secondary Base/Radix registry references rather than
  current docs-page sections.
- Pass: `RTL (Fret)`, `API Reference (Fret)`, `Composable Parts (Fret)`, `Vertical Line (Fret)`,
  `Extras (Fret)`, and `Notes` remain explicit Fret teaching or diagnostics follow-ups.
- Pass: The lead UI Gallery `Demo` snippet keeps the upstream `w-full max-w-sm` shell and does not
  force a full-width `TabsList`.
- Pass: The docs `Usage` snippet keeps the current `w-[400px]` caller-owned width lane.
- Pass: The RTL diagnostic script now follows the current four-tab RTL snippet ids instead of stale
  two-tab `preview`/`code` ids.

## Known gaps

- None for the component-matrix docs-path slice. Broader Base/Radix registry examples such as
  dropdown composition and input/button toolbars remain optional future expansion, not blockers for
  current docs-path regression lock.

## Validation

- `python -m json.tool docs/workstreams/shadcn-component-parity-matrix-v1/artifacts/tabs_agent_packet_p0_v1.json | Out-Null`
- `Get-ChildItem -Path tools/diag-scripts/ui-gallery/tabs -Filter *.json | ForEach-Object { python -m json.tool $_.FullName | Out-Null }`
- `python -m json.tool tools/parity-discovery/manifests/shadcn_parity_coverage_v2.json | Out-Null`
- `python -m json.tool docs/workstreams/shadcn-component-parity-matrix-v1/artifacts/shadcn_component_harness_matrix_v1.json | Out-Null`
- `python -m json.tool docs/workstreams/shadcn-component-parity-matrix-v1/WORKSTREAM.json | Out-Null`
- `python -m py_compile tools/parity-discovery/shadcn_component_harness_matrix.py`
- `rustfmt --edition 2024 --check apps/fret-ui-gallery/src/ui/pages/tabs.rs apps/fret-ui-gallery/src/ui/snippets/tabs/rtl.rs apps/fret-ui-gallery/tests/tabs_docs_surface.rs apps/fret-ui-gallery/tests/ui_authoring_surface_default_app.rs`
- `cargo nextest run -p fret-ui-kit --lib --status-level fail tabs`
- `cargo nextest run -p fret-ui-shadcn --lib --status-level fail tabs`
- `cargo nextest run -p fret-ui-shadcn --test tabs_keyboard_navigation --status-level fail`
- `cargo nextest run -p fret-ui-shadcn --features web-goldens --test web_vs_fret_layout --status-level fail tabs`
- `cargo nextest run -p fret-ui-shadcn --test web_vs_fret_misc_targeted --status-level fail shadcn_misc_goldens_are_targeted_gates`
- `cargo nextest run -p fret-ui-shadcn --test snapshots --status-level fail snapshot_tabs_default`
- `cargo nextest run -p fret-ui-shadcn --test radix_web_primitives_state --status-level fail tabs`
- `cargo nextest run -p fret-ui-gallery --test tabs_docs_surface --status-level fail`
- `cargo nextest run -p fret-ui-gallery --test ui_authoring_surface_default_app --status-level fail tabs`
- `python tools/parity-discovery/shadcn_component_harness_matrix.py`
- `python tools/check_workstream_catalog.py`
- `git diff --check`
