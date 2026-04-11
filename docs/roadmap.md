# Fret Roadmap (Draft)


## Upstream references (non-normative)

This document references optional local checkouts under `repo-ref/` for convenience.
Upstream sources:

- fret-ui-precision: (internal reference; no public upstream link)
- gpui-component: https://github.com/longbridge/gpui-component
- Tailwind CSS: https://github.com/tailwindlabs/tailwindcss
- shadcn/ui: https://github.com/shadcn-ui/ui

See `docs/repo-ref.md` for the optional local snapshot policy and pinned SHAs.
This roadmap focuses on building an editor-grade UI framework with docking, tear-off windows, and multiple engine viewports. The goal is to keep early work aligned with long-term constraints (multi-window, wgpu/WebGPU, plugins) to minimize future rewrites.

Key contracts are captured in ADRs under `docs/adr/`.

The runtime contract surface for `crates/fret-ui` is explicitly locked by `docs/adr/0066-fret-ui-runtime-contract-surface.md`. New runtime public APIs must name an authoritative upstream reference and land with tests before broadening usage.

Historical MVP planning snapshots live under `docs/archive/` (e.g. `docs/archive/mvp.md`,
`docs/archive/mvp/active-plan.md`). These are kept for context only and are not maintained as
active plans.

Workstream docs live under `docs/workstreams/` (living implementation trackers; not normative contracts).

Tracker highlights:

- Adaptive layout contract closure closeout record (closed cross-lane authoring taxonomy + proof
  surface lane for container vs environment queries, breakpoint vocabulary, and editor-rail owner
  split): `docs/workstreams/adaptive-layout-contract-closure-v1/TARGET_INTERFACE_STATE.md`,
  `docs/workstreams/adaptive-layout-contract-closure-v1/EVIDENCE_AND_GATES.md`, and
  `docs/workstreams/adaptive-layout-contract-closure-v1/CLOSEOUT_AUDIT_2026-04-10.md`.
- Device-shell strategy surface closeout record (closed narrow lane for explicit desktop/mobile
  shell branching above raw viewport queries, now shipping a crate-local `fret-ui-kit`
  `device_shell_*` helper with two real gallery consumers and explicit dialog/sidebar boundaries):
  `docs/workstreams/device-shell-strategy-surface-v1/TARGET_INTERFACE_STATE.md`,
  `docs/workstreams/device-shell-strategy-surface-v1/M3_SECOND_CONSUMER_PROOF_2026-04-11.md`, and
  `docs/workstreams/device-shell-strategy-surface-v1/CLOSEOUT_AUDIT_2026-04-11.md`.
- Text system v2 (Parley + attributed spans): `docs/workstreams/standalone/text-system-v2-parley.md` (see also `docs/todo-tracker.md`).
- UI typography presets (stable control text line boxes): `docs/workstreams/ui-typography-presets-v1/ui-typography-presets-v1.md`.
- Text style cascade fearless refactor (mechanism contract + ecosystem migration): `docs/workstreams/text-style-cascade-fearless-refactor-v1/DESIGN.md`, `docs/workstreams/text-style-cascade-fearless-refactor-v1/TODO.md`, and `docs/workstreams/text-style-cascade-fearless-refactor-v1/MILESTONES.md`.
- Headless table engine parity (TanStack Table core): `docs/workstreams/headless-table-tanstack-parity/headless-table-tanstack-parity.md` (TODOs: `docs/workstreams/headless-table-tanstack-parity/headless-table-tanstack-parity-todo.md`).
- Fonts bootstrap + invalidation contract: `docs/adr/0147-font-stack-bootstrap-and-textfontstackkey-v1.md`.
- Font bundle release boundary follow-on (framework bootstrap baseline vs optional CJK/emoji/full bundles + publish closure): `docs/workstreams/font-bundle-release-boundary-v1/DESIGN.md`, `docs/workstreams/font-bundle-release-boundary-v1/TODO.md`, `docs/workstreams/font-bundle-release-boundary-v1/MILESTONES.md`, and `docs/workstreams/font-bundle-release-boundary-v1/EVIDENCE_AND_GATES.md`.
- Icon system extension closeout record (semantic contract + multicolor authoring surface + third-party pack protocol): `docs/workstreams/icon-system-extension-v1/DESIGN.md`, `docs/workstreams/icon-system-extension-v1/TODO.md`, `docs/workstreams/icon-system-extension-v1/MILESTONES.md`, `docs/workstreams/icon-system-extension-v1/EVIDENCE_AND_GATES.md`, and `docs/workstreams/icon-system-extension-v1/CLOSEOUT_AUDIT_2026-04-09.md`.
- Icon install health hardening closeout record (closed narrow follow-on that hardens fail-fast explicit install surfaces vs best-effort partial runtime helpers for the shipped icon contract): `docs/workstreams/icon-install-health-hardening-v1/DESIGN.md`, `docs/workstreams/icon-install-health-hardening-v1/TODO.md`, `docs/workstreams/icon-install-health-hardening-v1/MILESTONES.md`, `docs/workstreams/icon-install-health-hardening-v1/EVIDENCE_AND_GATES.md`, `docs/workstreams/icon-install-health-hardening-v1/BASELINE_AUDIT_2026-04-09.md`, `docs/workstreams/icon-install-health-hardening-v1/M1_CONTRACT_FREEZE_2026-04-09.md`, `docs/workstreams/icon-install-health-hardening-v1/M2_PROOF_SURFACE_2026-04-09.md`, and `docs/workstreams/icon-install-health-hardening-v1/CLOSEOUT_AUDIT_2026-04-09.md`.
- Icon install error reporting closeout record (closed narrow follow-on that standardizes known icon-install failure reports plus diagnostics-aware panic-hook logging without reopening setup/bootstrap return types): `docs/workstreams/icon-install-error-reporting-v1/DESIGN.md`, `docs/workstreams/icon-install-error-reporting-v1/TODO.md`, `docs/workstreams/icon-install-error-reporting-v1/MILESTONES.md`, `docs/workstreams/icon-install-error-reporting-v1/EVIDENCE_AND_GATES.md`, `docs/workstreams/icon-install-error-reporting-v1/BASELINE_AUDIT_2026-04-09.md`, `docs/workstreams/icon-install-error-reporting-v1/M1_CONTRACT_FREEZE_2026-04-09.md`, `docs/workstreams/icon-install-error-reporting-v1/M2_PROOF_SURFACE_2026-04-09.md`, and `docs/workstreams/icon-install-error-reporting-v1/CLOSEOUT_AUDIT_2026-04-09.md`.
- Bootstrap known startup failure taxonomy closeout record (closed narrow follow-on that unifies returned settings/keymap/menu/assets startup failures with panic-only explicit icon install failures under one bootstrap-level taxonomy and diagnostics field family, without reopening lifecycle return types or widening the root `fret` re-export budget): `docs/workstreams/bootstrap-known-startup-failure-taxonomy-v1/DESIGN.md`, `docs/workstreams/bootstrap-known-startup-failure-taxonomy-v1/TODO.md`, `docs/workstreams/bootstrap-known-startup-failure-taxonomy-v1/MILESTONES.md`, `docs/workstreams/bootstrap-known-startup-failure-taxonomy-v1/EVIDENCE_AND_GATES.md`, `docs/workstreams/bootstrap-known-startup-failure-taxonomy-v1/BASELINE_AUDIT_2026-04-09.md`, `docs/workstreams/bootstrap-known-startup-failure-taxonomy-v1/M1_CONTRACT_FREEZE_2026-04-09.md`, `docs/workstreams/bootstrap-known-startup-failure-taxonomy-v1/M2_PROOF_SURFACE_2026-04-09.md`, and `docs/workstreams/bootstrap-known-startup-failure-taxonomy-v1/CLOSEOUT_AUDIT_2026-04-09.md`.
- Iconify import-pack generator closeout record (build-time generated third-party pack crates against the shipped icon contract; v1 now closes on reusable generator + public CLI, local SVG/Iconify snapshot inputs, explicit semantic alias config, and deterministic proof gates): `docs/workstreams/iconify-import-pack-generator-v1/DESIGN.md`, `docs/workstreams/iconify-import-pack-generator-v1/TODO.md`, `docs/workstreams/iconify-import-pack-generator-v1/MILESTONES.md`, `docs/workstreams/iconify-import-pack-generator-v1/EVIDENCE_AND_GATES.md`, `docs/workstreams/iconify-import-pack-generator-v1/M1_CONTRACT_FREEZE_2026-04-09.md`, `docs/workstreams/iconify-import-pack-generator-v1/M2_PROOF_SURFACE_2026-04-09.md`, `docs/workstreams/iconify-import-pack-generator-v1/M4_ICONIFY_COLLECTION_PROOF_2026-04-09.md`, and `docs/workstreams/iconify-import-pack-generator-v1/CLOSEOUT_AUDIT_2026-04-09.md`.
- Iconify acquisition pre-step closeout record (subset-first explicit remote/pinned acquisition that writes local Iconify snapshot + provenance artifacts for the closed generator lane, while keeping `icons import` local-input-only): `docs/workstreams/iconify-acquisition-prestep-v1/DESIGN.md`, `docs/workstreams/iconify-acquisition-prestep-v1/TODO.md`, `docs/workstreams/iconify-acquisition-prestep-v1/MILESTONES.md`, `docs/workstreams/iconify-acquisition-prestep-v1/EVIDENCE_AND_GATES.md`, `docs/workstreams/iconify-acquisition-prestep-v1/M2_PROOF_SURFACE_2026-04-09.md`, and `docs/workstreams/iconify-acquisition-prestep-v1/CLOSEOUT_AUDIT_2026-04-09.md`.
- Generated icon presentation defaults closeout record (closed narrow follow-on for correct `OriginalColors` vs `Mask` defaults in generated/imported icon packs, via explicit versioned generator config rather than runtime or acquisition rewrites): `docs/workstreams/generated-icon-presentation-defaults-v1/DESIGN.md`, `docs/workstreams/generated-icon-presentation-defaults-v1/TODO.md`, `docs/workstreams/generated-icon-presentation-defaults-v1/MILESTONES.md`, `docs/workstreams/generated-icon-presentation-defaults-v1/EVIDENCE_AND_GATES.md`, `docs/workstreams/generated-icon-presentation-defaults-v1/M2_PROOF_SURFACE_2026-04-09.md`, and `docs/workstreams/generated-icon-presentation-defaults-v1/CLOSEOUT_AUDIT_2026-04-09.md`.
- Iconify presentation-defaults suggestion closeout record (closed narrow helper lane for `icons suggest presentation-defaults` as an explicit provenance-driven companion to the shipped generated-pack config contract): `docs/workstreams/iconify-presentation-defaults-suggestion-v1/DESIGN.md`, `docs/workstreams/iconify-presentation-defaults-suggestion-v1/TODO.md`, `docs/workstreams/iconify-presentation-defaults-suggestion-v1/MILESTONES.md`, `docs/workstreams/iconify-presentation-defaults-suggestion-v1/EVIDENCE_AND_GATES.md`, `docs/workstreams/iconify-presentation-defaults-suggestion-v1/M2_PROOF_SURFACE_2026-04-09.md`, and `docs/workstreams/iconify-presentation-defaults-suggestion-v1/CLOSEOUT_AUDIT_2026-04-09.md`.
- Iconify presentation-defaults report closeout record (closed narrow follow-on for optional versioned review-report output from `icons suggest presentation-defaults`, keeping derivation evidence committed without changing import defaults): `docs/workstreams/iconify-presentation-defaults-report-v1/DESIGN.md`, `docs/workstreams/iconify-presentation-defaults-report-v1/TODO.md`, `docs/workstreams/iconify-presentation-defaults-report-v1/MILESTONES.md`, `docs/workstreams/iconify-presentation-defaults-report-v1/EVIDENCE_AND_GATES.md`, `docs/workstreams/iconify-presentation-defaults-report-v1/M2_PROOF_SURFACE_2026-04-09.md`, and `docs/workstreams/iconify-presentation-defaults-report-v1/CLOSEOUT_AUDIT_2026-04-09.md`.
- SVG presentation analysis scaffolding closeout record (closed narrow follow-on for local SVG-directory analysis that scaffolds conservative per-icon `original-colors` overrides and an optional review report without inferring pack-level defaults or changing import behavior): `docs/workstreams/svg-presentation-analysis-scaffolding-v1/DESIGN.md`, `docs/workstreams/svg-presentation-analysis-scaffolding-v1/TODO.md`, `docs/workstreams/svg-presentation-analysis-scaffolding-v1/MILESTONES.md`, `docs/workstreams/svg-presentation-analysis-scaffolding-v1/EVIDENCE_AND_GATES.md`, `docs/workstreams/svg-presentation-analysis-scaffolding-v1/M1_CONTRACT_FREEZE_2026-04-09.md`, `docs/workstreams/svg-presentation-analysis-scaffolding-v1/M2_PROOF_SURFACE_2026-04-09.md`, and `docs/workstreams/svg-presentation-analysis-scaffolding-v1/CLOSEOUT_AUDIT_2026-04-09.md`.
- Icon system closeout + follow-on map: `docs/workstreams/standalone/icon-system-status.md`.
- Docking multi-window parity (ImGui-style tear-off): `docs/workstreams/docking-multiwindow-imgui-parity/docking-multiwindow-imgui-parity.md` (TODOs: `docs/workstreams/docking-multiwindow-imgui-parity/docking-multiwindow-imgui-parity-todo.md`).
- Docking N-ary split graph (canonical form + ops semantics): `docs/workstreams/docking-nary-split-graph-v1/docking-nary-split-graph-v1.md` (TODOs: `docs/workstreams/docking-nary-split-graph-v1/docking-nary-split-graph-v1-todo.md`).
- Immediate-mode compatibility retained surface reduction (closed closeout lane for public/proof retained-compatibility surface deletion): `docs/workstreams/imui-compat-retained-surface-v1/DESIGN.md`, `docs/workstreams/imui-compat-retained-surface-v1/TODO.md`, `docs/workstreams/imui-compat-retained-surface-v1/MILESTONES.md`, `docs/workstreams/imui-compat-retained-surface-v1/BASELINE_AUDIT_2026-03-31.md`, and `docs/workstreams/imui-compat-retained-surface-v1/CLOSEOUT_AUDIT_2026-03-31.md`.
- Immediate-mode stack v2 closeout record (stack reset + editor adapter closure + teaching-surface cleanup): `docs/workstreams/imui-stack-fearless-refactor-v2/CLOSEOUT_AUDIT_2026-03-31.md`.
- Immediate-mode archive map (older `imui` authoring/ecosystem/parity notes): `docs/workstreams/README.md#immediate-mode-workstream-map`.
- Foundation closure tracker (P0 cross-workstream milestones): `docs/workstreams/foundation-closure-p0/foundation-closure-p0.md` (TODOs: `docs/workstreams/foundation-closure-p0/foundation-closure-p0-todo.md`).
- Authoring surface + ecosystem reset (pre-release, no-compat cleanup): `docs/workstreams/authoring-surface-and-ecosystem-fearless-refactor-v1/DESIGN.md`, `docs/workstreams/authoring-surface-and-ecosystem-fearless-refactor-v1/TODO.md`, `docs/workstreams/authoring-surface-and-ecosystem-fearless-refactor-v1/MILESTONES.md`, `docs/workstreams/authoring-surface-and-ecosystem-fearless-refactor-v1/TARGET_INTERFACE_STATE.md`, and `docs/workstreams/authoring-surface-and-ecosystem-fearless-refactor-v1/MIGRATION_MATRIX.md`.
- Ecosystem integration traits budget (install/catalog/router/docking/query seams): `docs/workstreams/ecosystem-integration-traits-v1/DESIGN.md`, `docs/workstreams/ecosystem-integration-traits-v1/TODO.md`, `docs/workstreams/ecosystem-integration-traits-v1/MILESTONES.md`, `docs/workstreams/ecosystem-integration-traits-v1/TARGET_INTERFACE_STATE.md`, and `docs/workstreams/ecosystem-integration-traits-v1/MIGRATION_MATRIX.md`.
- Into-element surface cleanup (closed closeout lane for conversion vocabulary collapse and typed first-party teaching surfaces): `docs/workstreams/into-element-surface-fearless-refactor-v1/DESIGN.md`, `docs/workstreams/into-element-surface-fearless-refactor-v1/TODO.md`, `docs/workstreams/into-element-surface-fearless-refactor-v1/MILESTONES.md`, `docs/workstreams/into-element-surface-fearless-refactor-v1/TARGET_INTERFACE_STATE.md`, `docs/workstreams/into-element-surface-fearless-refactor-v1/MIGRATION_MATRIX.md`, and `docs/workstreams/into-element-surface-fearless-refactor-v1/CLOSEOUT_AUDIT_2026-03-20.md`.
- Post-v1 authoring density reduction (closed closeout lane): `docs/workstreams/authoring-density-reduction-fearless-refactor-v1/DESIGN.md`, `docs/workstreams/authoring-density-reduction-fearless-refactor-v1/TARGET_INTERFACE_STATE.md`, `docs/workstreams/authoring-density-reduction-fearless-refactor-v1/MILESTONES.md`, `docs/workstreams/authoring-density-reduction-fearless-refactor-v1/TODO.md`, `docs/workstreams/authoring-density-reduction-fearless-refactor-v1/CLOSEOUT_AUDIT_2026-03-16.md`, and the post-closeout Todo/scaffold interpretation addendum `docs/workstreams/authoring-density-reduction-fearless-refactor-v1/TODO_LADDER_AUDIT_2026-03-20.md`.
- Dataflow authoring surface (closed closeout lane for selector/query + ecosystem/router boundary conclusions): `docs/workstreams/dataflow-authoring-surface-fearless-refactor-v1/DESIGN.md`, `docs/workstreams/dataflow-authoring-surface-fearless-refactor-v1/TARGET_INTERFACE_STATE.md`, `docs/workstreams/dataflow-authoring-surface-fearless-refactor-v1/MILESTONES.md`, `docs/workstreams/dataflow-authoring-surface-fearless-refactor-v1/TODO.md`, and `docs/workstreams/dataflow-authoring-surface-fearless-refactor-v1/MIGRATION_MATRIX.md`.
- Selector/query authoring density (closed closeout lane after the narrow selector/query density follow-on landed): `docs/workstreams/selector-query-authoring-density-fearless-refactor-v1/DESIGN.md`, `docs/workstreams/selector-query-authoring-density-fearless-refactor-v1/TARGET_INTERFACE_STATE.md`, `docs/workstreams/selector-query-authoring-density-fearless-refactor-v1/MILESTONES.md`, `docs/workstreams/selector-query-authoring-density-fearless-refactor-v1/TODO.md`, `docs/workstreams/selector-query-authoring-density-fearless-refactor-v1/SELECTOR_BORROWED_INPUT_AUDIT_2026-03-20.md`, and `docs/workstreams/selector-query-authoring-density-fearless-refactor-v1/CLOSEOUT_AUDIT_2026-03-20.md`.
- View-locals authoring (closed closeout lane for grouped view-owned `LocalState<T>` organization): `docs/workstreams/view-locals-authoring-fearless-refactor-v1/DESIGN.md`, `docs/workstreams/view-locals-authoring-fearless-refactor-v1/TARGET_INTERFACE_STATE.md`, `docs/workstreams/view-locals-authoring-fearless-refactor-v1/MILESTONES.md`, `docs/workstreams/view-locals-authoring-fearless-refactor-v1/TODO.md`, and `docs/workstreams/view-locals-authoring-fearless-refactor-v1/CLOSEOUT_AUDIT_2026-03-20.md`.
- Action write surface (closed closeout lane for the default app-lane write budget): `docs/workstreams/action-write-surface-fearless-refactor-v1/DESIGN.md`, `docs/workstreams/action-write-surface-fearless-refactor-v1/TARGET_INTERFACE_STATE.md`, `docs/workstreams/action-write-surface-fearless-refactor-v1/MILESTONES.md`, `docs/workstreams/action-write-surface-fearless-refactor-v1/TODO.md`, `docs/workstreams/action-write-surface-fearless-refactor-v1/CLOSEOUT_AUDIT_2026-03-17.md`, and `docs/workstreams/action-write-surface-fearless-refactor-v1/RETAINED_PAYLOAD_SURFACE_AUDIT_2026-03-17.md`.
- App composition density follow-on (closeout / maintenance lane for the default app-lane composition/query shell follow-on): `docs/workstreams/app-composition-density-follow-on-v1/DESIGN.md`, `docs/workstreams/app-composition-density-follow-on-v1/TARGET_INTERFACE_STATE.md`, `docs/workstreams/app-composition-density-follow-on-v1/MILESTONES.md`, and `docs/workstreams/app-composition-density-follow-on-v1/TODO.md`.
- Local-state architecture follow-on (closed decision lane): `docs/workstreams/local-state-architecture-fearless-refactor-v1/DESIGN.md`, `docs/workstreams/local-state-architecture-fearless-refactor-v1/MILESTONES.md`, `docs/workstreams/local-state-architecture-fearless-refactor-v1/TODO.md`, and `docs/workstreams/local-state-architecture-fearless-refactor-v1/CLOSEOUT_AUDIT_2026-03-16.md`.
- Local-state facade boundary hardening (closed maintenance lane): `docs/workstreams/local-state-facade-boundary-hardening-v1/DESIGN.md`, `docs/workstreams/local-state-facade-boundary-hardening-v1/MILESTONES.md`, `docs/workstreams/local-state-facade-boundary-hardening-v1/TODO.md`, `docs/workstreams/local-state-facade-boundary-hardening-v1/SURFACE_INVENTORY_2026-03-16.md`, and `docs/workstreams/local-state-facade-boundary-hardening-v1/CLOSEOUT_AUDIT_2026-03-16.md`.
- Public authoring state lanes + identity contract (active pre-release lane for LocalState-first default teaching, explicit raw-model naming, kernel/facade substrate convergence, and full example migration): `docs/workstreams/public-authoring-state-lanes-and-identity-fearless-refactor-v1/DESIGN.md`, `docs/workstreams/public-authoring-state-lanes-and-identity-fearless-refactor-v1/TODO.md`, `docs/workstreams/public-authoring-state-lanes-and-identity-fearless-refactor-v1/MILESTONES.md`, `docs/workstreams/public-authoring-state-lanes-and-identity-fearless-refactor-v1/MIGRATION_MATRIX.md`, and `docs/adr/0319-public-authoring-state-lanes-and-identity-contract-v1.md`.

Current pre-release authoring cleanup sequence:

1. `authoring-surface-and-ecosystem-fearless-refactor-v1` shrinks the app/component/advanced lanes and the default teaching surface.
2. `ecosystem-integration-traits-v1` locks the ecosystem trait budget for install/catalog/router/docking/query seams.
3. `into-element-surface-fearless-refactor-v1` finished the remaining conversion-surface cleanup so curated first-party examples, including shadcn/UI Gallery surfaces, converge on one authoring vocabulary.
4. `authoring-density-reduction-fearless-refactor-v1` closes the shorter default-path teaching
   surface and locks it with docs/gates.
5. `local-state-architecture-fearless-refactor-v1` documented the separate follow-on decision
   around the long-term storage/ownership contract behind `LocalState<T>` after the broader
   authoring-surface cleanup chain was already closed.
6. `local-state-facade-boundary-hardening-v1` completed the narrow O1 follow-on:
   it hardened the public facade around default local-state, explicit raw-model seams, and
   explicit bridge APIs without reopening storage-model design.

Treat the first four trackers as one closed execution chain rather than unrelated cleanup notes.
Treat item 5 as a closed architecture decision and item 6 as its completed narrow facade-hardening
follow-on.

The next authoring-focused lane is intentionally narrower:

7. `dataflow-authoring-surface-fearless-refactor-v1` now closes out the selector/query default
   posture plus the reusable-ecosystem/router boundary conclusions for the narrowed dataflow lane.
8. `action-write-surface-fearless-refactor-v1` now closes the remaining default app-lane
   write-side budget on `cx.actions()`, while keeping router/history, selector/query, and
   `LocalState<T>` architecture scope separate as future explicit follow-ons only.
9. `app-composition-density-follow-on-v1` now reads as a narrow closeout lane:
   M1 closes on a no-new-API composition verdict, M2 closes on grouped app-lane query
   invalidation, and only M3 maintenance/lock work remains.
10. `selector-query-authoring-density-fearless-refactor-v1` reopens only the remaining
    selector/query default-path density after the earlier closeouts froze ownership and router
    boundaries.
11. `view-locals-authoring-fearless-refactor-v1` then closes the remaining grouped view-owned
    `LocalState<T>` organization question on the default app lane without adding new framework APIs
    or reopening storage-model design.

Current execution stance on 2026-03-20:

- `dataflow-authoring-surface-fearless-refactor-v1` = closed closeout lane:
  selector/query conclusions and ecosystem/router boundary notes are locked there; do not reopen
  default write-side questions on that tracker.
- `action-write-surface-fearless-refactor-v1` = closed closeout lane:
  the default app-lane write-side budget is now frozen there; keep `payload_local_update_if` as
  the taught default keyed row-write path, keep raw
  `on_payload_action_notify::<A>(...)` as the explicit payload-side advanced fallback, and reopen
  only through a new narrower lane if fresh cross-surface evidence appears.
- `app-composition-density-follow-on-v1` = closeout / maintenance lane:
  M1 and M2 are now closed; keep the lane limited to source-gate/docs lockstep and reopen only if
  fresh cross-surface evidence shows a new default app-lane gap beyond first-party drift.
- `selector-query-authoring-density-fearless-refactor-v1` = closed closeout lane:
  keep the shipped query semantic projections and the no-new-API selector verdict; reopen only if
  fresh cross-surface evidence exceeds the closed closeout audit.
- `view-locals-authoring-fearless-refactor-v1` = closed closeout lane:
  keep the shipped `1-2 inline / 3+ bundle` rule and the no-new-API verdict; reopen only if
  fresh cross-surface evidence exceeds the closeout audit.
- `authoring-surface-and-ecosystem-fearless-refactor-v1` = closeout lane:
  keep deleting stale aliases, tightening gates, and cleaning docs, but do not reopen broad
  surface redesign here.
- `ecosystem-integration-traits-v1` = maintenance closeout lane:
  most core trait decisions are landed, `QueryAdapter` is now explicitly deferred in v1, and the
  remaining work is root-export/docs cleanup plus final mixed-posture auditing.
- `into-element-surface-fearless-refactor-v1` = closeout / maintenance lane:
  the broad conversion-surface migration is now landed; remaining work is explicit seam inventory
  and source-gate maintenance.
- `authoring-density-reduction-fearless-refactor-v1` = closeout / maintenance lane:
  the shorter default-path teaching surface is landed; remaining work is wording/gate maintenance
  rather than another helper-growth pass.
- `local-state-architecture-fearless-refactor-v1` = closed / maintenance lane:
  the lane now closes on `O1` (keep model-backed storage, harden the facade boundary, keep
  `AppUiRawModelExt::raw_model::<T>()` as the explicit raw-model seam); reopen only if fresh
  cross-surface evidence shows that the storage model itself has become the bottleneck.
- `local-state-facade-boundary-hardening-v1` = closed / maintenance lane:
  the initial wording/gate hardening batch is landed; the repo now consistently classifies
  `AppUiRawModelExt::raw_model::<T>()` as the advanced raw-model seam and
  `LocalState::{model, clone_model, *_in, watch_in}` as explicit bridge APIs.

Recommended order from here:

1. keep `action-write-surface-fearless-refactor-v1` closed unless fresh cross-surface evidence can
   justify a narrower re-promotion or delete-ready follow-on,
2. keep any further retained payload-seam notes inside
   `action-write-surface-fearless-refactor-v1/RETAINED_PAYLOAD_SURFACE_AUDIT_2026-03-17.md`
   unless they genuinely grow into a larger lane; the remaining payload-side advanced fallback
   there is now raw `on_payload_action_notify::<A>(...)`,
3. keep the remaining ecosystem-trait docs/export cleanup and the conversion-surface
   inventories/gates aligned as maintenance rather than reopening broad redesign,
4. keep `app-composition-density-follow-on-v1` in closeout / maintenance mode unless new
   cross-surface evidence appears,
5. keep `view-locals-authoring-fearless-refactor-v1` closed unless fresh cross-surface evidence
   shows that the shipped grouped-locals rule is no longer sufficient,
6. keep `selector-query-authoring-density-fearless-refactor-v1` closed unless fresh cross-surface
   evidence exceeds the shipped closeout audit,
7. keep the default authoring closeout lanes stable rather than reopening helper growth from stale
   wording drift,
8. keep `local-state-architecture-fearless-refactor-v1` closed on the O1 decision rather than
   treating it as another open-ended state-surface lane,
9. keep `local-state-facade-boundary-hardening-v1` closed unless fresh evidence shows that wording
   and current gates are no longer sufficient,
10. only reopen storage-model refactors if fresh evidence can name both the bottleneck and the
   proof surfaces + gates in advance.

For the “foundation-first, component-validated” execution loop (Plan C), see `docs/foundation-first-workflow.md`.

Subsystem roadmaps (living docs, not ADRs):

- Node graph: `docs/node-graph-roadmap.md` (tracked TODOs for `ecosystem/fret-node`; contracts in ADR 0126).
- Layout engine: `docs/layout-engine-refactor-roadmap.md`.
- Renderer: `docs/renderer-refactor-roadmap.md`.
- OS menubar: `docs/workstreams/standalone/os-menubar.md`.

For the declarative layout refactor phases (P0–P5) toward a window-scoped Taffy engine + viewport roots, see:
`docs/layout-engine-refactor-roadmap.md`.

## Toolchain Baseline

- `wgpu 28.x` currently requires Rust `1.92+` (enforced via `rust-toolchain.toml` and `workspace.package.rust-version`).

## Status Legend (Roadmap Notation)

- `(done)`: merged on `main` and considered a stable foundation.
- `(prototype implemented)`: exists in code but not yet stabilized as a long-term contract; expect refactors.
- `(MVP done ...)`: minimal version exists; expected to evolve (quality/perf/edge cases not complete).

## Priorities

- **P0**: Foundational architecture that is hard to change later (windowing, host-provided GPU context, UI execution model, multi-root overlays, event routing, display list ordering semantics, renderer resource ownership, dock manager, persistence contracts).
- **P1**: Editor usability essentials (theme/tokens, docking UX polish, panels, menus, shortcuts, basic text).
- **P2**: Editor completeness (code editor-grade text, IME, accessibility hooks, advanced rendering effects).
- **P3**: Portability extensions (wasm/WebGPU, mobile).

## Refactors to Do Early (Avoid Big Rewrites)

These items are intentionally scheduled early because they define “hard-to-change” semantics:

- P0: Adopt the primary UI authoring/runtime model (declarative elements + externalized state) (ADR 0028).
- P0: Adopt the app-owned model store + borrow-friendly update API (ADR 0031).
- P0: Adopt typed style tokens + theme resolution rules (ADR 0032).
- P0: Align baseline theme semantics with gpui-component/shadcn vocabulary (semantic palette + typography), so the component ecosystem can reuse upstream conventions without inflating `fret-ui` into a UI kit. (prototype implemented: semantic palette aliases + typography expansion; ADR 0050 follow-up)
- P0: Adopt a component-level size/density system so Tailwind-like primitives converge on one contract (ADR 0056). (prototype implemented; MVP 47)
- P0: Adopt component-level style composition ergonomics (`StyleRefinement` + `StyledExt`) so Tailwind/shadcn-like recipes and variants can be expressed as typed, composable “style patches” without leaking UI kit semantics into `fret-ui` runtime. (MVP 45)
- P0: Harden Tailwind-like authoring semantics so they remain trustworthy as the component ecosystem scales:
  - split style patches into `ChromeRefinement` vs `LayoutRefinement` to avoid “layout no-ops” on retained widgets,
  - standardize semantic token key vocabulary (e.g. list row hover/active) to avoid theme alignment drift,
  - make `merge` semantics match Tailwind edge accumulation (e.g. `mt-*` + `ml-*` composes without dropping edges).
- P0: Tighten the **framework vs components** boundary so Tailwind/shadcn sizing/variants can converge without fighting runtime widgets:
  - keep `fret-ui` as the runtime substrate (tree, routing, layers, docking, performance primitives),
  - move the “standard UI kit surfaces” (popover/dialog/menu/tooltip/toast/command palette/menubar) to the components layer (`fret-ui-shadcn` surface built on `fret-ui-kit`),
  - remove hard-coded control heights/spacing from runtime primitives (notably `TextInput`) so `Size` (ADR 0056) stays component-owned. (MVP 48; in progress: `TextInput` done; overlay surfaces moved)
- P0: Eliminate **interaction policy leaks** from `crates/fret-ui` so the runtime contract surface stays small and optimizable. (done; ADR 0074 / MVP 68)
  - Removed runtime “shortcut model writes” (pressable toggle/set variants, dismiss-by-model patterns) and roving/typeahead coupling.
  - Components must express policy via action hooks + component-owned headless helpers (`fret-ui-kit` / `fret-ui-shadcn`).
- P0: Move **Docking UI** out of `crates/fret-ui` (policy-heavy, hard to optimize) into a dedicated component/app layer crate, while keeping runtime substrate clean.
  - Keep dock **data model + ops + persistence shapes** in `fret-core` (as stable contracts).
  - Keep `fret-ui` responsible only for **generic mechanisms** (event routing, drag routing hooks, overlay roots/layers, viewport embedding).
  - Provide a feature-gated retained bridge (`fret-ui/unstable-retained-bridge`) to migrate docking UI out without a rewrite; treat it as explicitly unstable.
  - Move “editor/viewport overlays” (gizmos, marquee, selection rects) into `fret-editor` / app-layer code (ADR 0027).
  - Provide a minimal, generic internal-drag routing extension point so docking can preserve tear-off / cross-window drag UX without hard-coding dock types into the runtime.
- P0: Adopt semantics tree + accessibility bridge boundary (A11y-ready infrastructure) (ADR 0033).
- P0/P1: Lock composite-widget semantics needed for cmdk-style UX without a DOM:
  - active descendant (announce active result while focus remains in the text field) (ADR 0073),
  - avoid depending on virtualized AT semantics until the virtualization strategy is locked (ADR 0070).
- P0: Adopt timers/animation/redraw scheduling (event-driven + continuous mode) (ADR 0034).
  - Contract shape locked (GPUI-aligned): one-shot `request_frame`, `request_animation_frame`, and refcounted RAII `begin_continuous_frames` leases (see ADR 0034).
- P0: Adopt layout constraints + optional Flex/Grid integration boundary (ADR 0035).
- P0: Fix declarative composable sizing semantics (fit-content by default, typed size constraints, and a Taffy-backed `Flex` container) so Tailwind/shadcn component composition is viable without per-widget layout hacks. (done; ADR 0057 / MVP 52)
- P0: Harden declarative Flex/Grid layout performance (persistent Taffy trees + measure memoization). (done; ADR 0076 / MVP 61)
- P0: Replace the legacy `Effect::UiInvalidateLayout` escape hatch with model-driven invalidation (MVP 66), then remove it once remaining callers are migrated. (done)
- P0: Adopt Tailwind layout vocabulary as framework-level `LayoutStyle` semantics (margin, position/inset, grid, aspect ratio) so shadcn-style composition can express common patterns (badge overlays, input icons, simple grids) without bespoke per-widget layout hacks. (ADR 0062)
- P0: Adopt rounded clipping / `overflow-hidden` semantics as a first-class renderer feature (ADR 0063), so shadcn-style surfaces (Popover/HoverCard/ScrollArea) do not rely on per-component “mask hacks”. (done; MVP 60)
- P0: Adopt a “behavior reference stack” for component parity without DOM/CSS dependencies:
  - APG (keyboard/focus semantics baseline),
  - Radix primitives (dismissal/focus/portal outcomes),
  - Floating UI (placement + collision algorithms),
  - cmdk (command palette interaction details; implies active-descendant semantics, ADR 0073; Phase A implemented).
  - See: `docs/reference-stack-ui-behavior.md`.
- P0: Adopt observability strategy (tracing + inspector hooks + renderer metrics) (ADR 0036).
- P0: Adopt workspace/repo boundaries and external `fret-components` strategy (ADR 0037).
- P0: Adopt component authoring model (Render/RenderOnce + IntoElement) (ADR 0039).
- P0: Make the declarative authoring model an end-to-end, usable implementation path (not only a cross-frame state store), so virtualization and complex components can be expressed as composition rather than runtime widgets. (MVP 49/50)
- P0: Introduce a `UiHost` boundary so `fret-ui` can be embedded by third-party hosts (ADR 0052). (done; `fret-runtime` + `fret-ui-app`)
- P0: Make external file drag payload portable (no `PathBuf` in core events) (ADR 0053). (done)
- P0: Introduce runtime platform capability matrix for portability (ADR 0054). (done)
- P0: Introduce a portable system cursor boundary (cursor icon + per-window cursor routing + overlay precedence).
  - Rationale: editor-grade resizing/docking needs OS cursor affordances; without a stable boundary, each widget
    invents its own heuristics and portability will suffer.
  - Implemented as MVP 46 (prototype; see `docs/archive/mvp/active-plan.md`).
- P0: Renderer must preserve `Scene.ops` ordering across primitive kinds (ADR 0009).
- P0: Multi-root overlays (menus, drag previews, popups, modals) must be first-class (ADR 0011).
- P0: Lock anchored overlay geometry under `render_transform` (ADR 0082) so popovers/tooltips track what the user sees.
  - Migrate component anchors to use `visual_bounds_for_element` (not layout-only bounds).
  - Keep a runtime-level regression test in `fret-ui`, plus a small component-level unit test guarding anchor selection (visual vs layout) to prevent drift.
  - References: `docs/adr/0064-overlay-placement-contract.md`, `docs/adr/0082-render-transform-hit-testing.md`.
- P0: Keyboard/IME split: physical keys for shortcuts, text input for editing (ADR 0012).
- P0: Canonical physical key representation for shortcuts + keymap persistence (ADR 0018).
- P0: Shortcut arbitration + AltGr semantics + pending bindings (avoid keymap/API breaking changes) (ADR 0043).
- P0: Focus + command routing semantics (widget/window/app scopes) are fixed early (ADR 0020).
- P0: Keymap file format + conflict/override semantics are fixed early (ADR 0021).
- P0: `when` expression model is shared by keymap + command gating (ADR 0022).
- P0: Unified command metadata powers menus + palette + shortcuts (ADR 0023).
- P0: Host-provided `WgpuContext` so both editor-hosted and engine-hosted topologies are supported (ADR 0010).
- P0: Canonical frame lifecycle + explicit engine/UI submission ordering (ADR 0015).
- P0: Color management and compositing rules are fixed early (linear compositor + viewport encoding metadata) (ADR 0040).
- P0: Dock persistence and stable panel identity (`PanelKind`) with versioned layout format (ADR 0013).
- P0: Internal drag sessions + clipboard boundary are fixed early (cross-window docking UX) (ADR 0041).
- P0: Scene state stack extension points (transform/opacity/layers) are reserved early (ADR 0019).
- P0: Resource lifetime/eviction/budgets are defined at the handle boundary (ADR 0004).
- P0: Plugin and panel boundaries are app-owned and renderer-free (ADR 0016).
- P0: Multi-window DPI semantics are explicit and portable (ADR 0017).
- P0: Viewport input forwarding contract is fixed early (ADR 0025).
- P0: Text shaping/atlas strategy is decided before shipping text-heavy widgets (ADR 0029).
- P0: Shape semantics (borders/shadows/AA rules) are defined before building docking chrome visuals (ADR 0030).
- P0: Remove layout-engine dependencies from `fret-core` (align with ADR 0035 / ADR 0037). (done)

## Decision Gates (Before Scaling Widget Count)

Before investing in a large widget library, the project should “lock in” the following.

Already locked (Accepted):

- `docs/adr/0028-declarative-elements-and-element-state.md`
- `docs/adr/0029-text-pipeline-and-atlas-strategy.md`
- `docs/adr/0030-shape-rendering-and-sdf-semantics.md`
- `docs/adr/0035-layout-constraints-and-optional-taffy-integration.md`
- `docs/adr/0038-engine-render-hook-and-submission-coordinator.md`
- `docs/adr/0040-color-management-and-compositing-contracts.md`
- `docs/adr/0041-drag-and-drop-clipboard-and-cross-window-drag-sessions.md`

Current policy:

- Treat these `Accepted` ADRs as “hard contracts”.
- If a new hard-to-change design decision appears during implementation, add a new ADR as `Status: Proposed`,
  review it, then promote to `Accepted` before expanding surface area.
- Prefer **updating** an existing ADR section over creating many micro-ADRs (keep the index usable).

When to write (or significantly update) an ADR:

- Cross-crate boundaries or dependency direction (`fret-core`/`fret-ui`/runner/platform/render).
- Long-lived file formats (layout/theme/keymap/project/scene) or their versioning/migration rules.
- Input/focus/capture/command routing semantics or window/app scope rules.
- Display list / renderer semantics (`SceneOp` meaning, ordering, blending, clipping, AA, pixels).
- Portability contracts (wasm/WebGPU capability gaps, payload shapes, “no PathBuf” style constraints).
- Caching/invalidation semantics that affect correctness, determinism, or replay/recording behavior.

MVP guidance:

- Each MVP item in `docs/archive/mvp/active-plan.md` should link to the ADRs that define its non-negotiable contracts.
- If a prototype reveals an incorrect assumption, update the ADR first, then adjust the implementation.

## Framework Capability Promotions (From Demo/Widgets to Core)

These are recurring “editor-grade UX” needs that should be expressed as **portable framework contracts**
(`fret-core`/`fret-runtime` + runner effects), not re-implemented ad-hoc in individual widgets or demos.

The goal is a GPUI-like reuse story: third-party hosts and component ecosystems can rely on the same behavior.

P0 / near-term (schedule as MVPs/refactors):

- **System cursor + pointer feedback**: portable cursor icons, per-window cursor routing, overlay precedence, and
  reusable resize-handle primitives (planned as MVP 46 in `docs/archive/mvp/active-plan.md`).
- **Resizable layout primitive**: a reusable split/resize container (panel group) so “dock splits”, “inspector
  sidebars”, and “data table column resize” share hit-testing and cursor behavior (pairs naturally with MVP 46).
  - Current prototype: runtime-owned panel group with `gap` (layout) + `hit_thickness` (pointer target) and shared
    drag/min-size behavior across shadcn `Resizable*` and docking splits (ADR 0077).
  - Follow-up: decide whether handles should become first-class child elements (vs internal painted affordances) once
    focus rings + a11y semantics are finalized; see gpui-component’s `resize_handle` usage in
    `repo-ref/gpui-component/crates/ui/src/resizable/panel.rs`.
- **Pointer lock / relative motion** (viewport navigation): a portable effect boundary for “orbit/pan camera”
  interactions where OS cursor constraints differ (desktop vs web); capability-gated and opt-in.
- **Overlay hit-testing policies**: pointer-transparent overlays (click-through “empty” overlay regions) and
  deterministic layering for transient surfaces (tooltips/toasts/drag hints). (prototype implemented in runtime; used by toast)

P1 / medium-term (app polish; after primitives settle):

- **Tooltip / hover help service**: delayed tooltips, follow-cursor placement, and consistent overlay ordering
  (build on existing overlay roots and `PopoverService`).
- **Window policy hooks**: standardized “raise/focus on create”, window activation transfer, and predictable
  z-order behavior across platforms (reduce demo-specific fixes).

P2 / later (portability/product completeness):

- **Native dialogs**: open/save file dialogs, open folder, reveal-in-file-manager (effect-driven and capability-gated).
- **External integrations**: open URL, system notifications (portable subset).
- **Rich clipboard**: images/files (beyond text), with explicit portability constraints.

## Example Editor App Notes (Out of Scope for Fret Framework)

These are important for building a full engine editor, but they are *application* concerns and
should not be treated as Fret framework deliverables (see ADR 0027):

- `docs/adr/0024-undo-redo-and-edit-transactions.md`
- `docs/adr/0026-asset-database-and-import-pipeline.md`
- `docs/adr/0048-inspector-property-protocol-and-editor-registry.md`
- `docs/adr/0049-viewport-tools-input-capture-and-overlays.md`

## Example Editor Workflows (Demo-Driven, P1)

These are not framework commitments (ADR 0027), but they are essential to reaching Unity/Godot-like “feel” early and to
validate that the framework contracts are sufficient.

- Inspector P0: engine-agnostic property protocol + custom editor registry + minimal built-in editors (ADR 0048). (prototype implemented in demo; see `docs/archive/mvp/active-plan.md` MVP 17; two-column inline rows + Alt+drag scrubbing)
- Viewport tools P0: input capture + tool routing + overlay rendering over viewport surfaces (ADR 0049, built on ADR 0025). (prototype implemented in demo; see `docs/archive/mvp/active-plan.md` MVP 18)
- Viewport picking P0: click-to-select + selection highlight overlays (demo-driven; see `docs/archive/mvp/active-plan.md` MVP 19).
- Gizmo P0: translate tool with explicit drag phases, capture rules, and basic polish (axis constraints + snapping stub) (prototype implemented in demo; see `docs/archive/mvp/active-plan.md` MVP 20 + MVP 25).
- Viewport navigation P0: pan/orbit stub + wheel zoom (prototype implemented in demo; see `docs/archive/mvp/active-plan.md` MVP 26).
- Gizmo P0: rotate tool stub (prototype implemented in demo; see `docs/archive/mvp/active-plan.md` MVP 27).
- Multi-viewport roles P0: Scene (editor) vs Game (preview) gating patterns (prototype implemented in demo; see `docs/archive/mvp/active-plan.md` MVP 30).
- Play mode stub P0: preview loop with RAF scheduling while a Game viewport is visible (prototype implemented in demo; see `docs/archive/mvp/active-plan.md` MVP 31).
- Undo/redo P0: command stack + transaction coalescing boundary (ADR 0024). (prototype implemented in demo; see `docs/archive/mvp/active-plan.md` MVP 22 + MVP 24).
- Scene documents P0: `.scene` open/new/save/save-as + unsaved changes guard (demo-driven; see `docs/archive/mvp/active-plan.md` MVP 39–42).

## Milestones

### M0 — Workspace & Boot

- P0: Cargo workspace and crate boundaries (`fret-core`, `fret-app`, `fret-ui`, `fret-platform`, `fret-render`).
- P0: Enforce crate dependency direction rules and keep contracts backend-agnostic (ADR 0037).
- P0: Minimal `fret-demo` that opens a window and presents via wgpu. (done)
- P0: Runner crate that wires `winit` + `wgpu` + renderer without pulling renderer into `fret-platform`. (done)

### M1 — App Runtime Foundations

- P0: `App` runtime with global services container (type map).
- P0: `Model<T>` store with typed handles and explicit update closures (ADR 0031).
- P0: Lease-based model updates (`App` can pass `&mut App` + `&mut T` safely) (ADR 0031).
- P0: Command registry (`CommandId`, metadata, discovery hooks).
- P0: Effects queue + fixed-point draining (redraw/window requests; effects enqueued from callbacks). (done)
- P0: Timers and animation frame requests via effects (ADR 0034).
- P1: Plugin registry scaffolding (panel factories, command registration).

### M2 — UI Runtime Core (Single Window)

- P0: Decide and implement the primary UI execution model (ADR 0028), while preserving layout/input semantics (ADR 0005). (prototype implemented: declarative element ids + cross-frame state store; `UiTree` remains the internal host, but component authoring is declarative-only)
- P0: Invalidation flags: `NeedsLayout`, `NeedsPaint`, `NeedsHitTestRebuild`.
- P0: Model observation → UI invalidation propagation (avoid manual cross-panel invalidation; align with GPUI `observe`/`notify` mental model). (ADR 0051) (prototype implemented)
  - Promote ADR 0051 to "Accepted": add conformance tests and remove any remaining demo/component-level manual invalidation glue.
- P0: Event routing: hit-test, focus, capture, bubble. (prototype implemented; see ADR 0005)
- P1: External OS file drag-and-drop routing skeleton (winit file DnD + hit-test routing). (prototype implemented; see ADR 0041)
  - Note (macOS/winit): hover/move positions are not continuously available; per-widget drop targets will require a native backend (see ADR 0041).
- P0: Multi-root overlays + z-order + modal blocking model. (see ADR 0011)
- P0: Semantics tree output (UI-only; platform bridge deferred) (ADR 0033).
- P1: Base declarative elements: `Container`, `Stack`, `Flex`, `Grid`, `Scroll`, `VirtualList`. (prototype implemented)
- P1: Scrollbar UX: draggable thumb + track clicking. (prototype implemented)
- P1: Editor-scale list widgets: `VirtualList` (runtime primitive), `TreeView` (legacy, gated; prototype implemented)
- P1: Layout contract: `layout_in(child, rect)` stores child bounds for hit-test/paint. (prototype implemented)
- P1: Optional `Flex`/`Grid` widget backed by `taffy` (defer until needed; no `UiTree` refactor).
- P0: Virtualization contract for editor-scale lists/tables/editors (no unbounded children in layout engines). (ADR 0042) (prototype implemented: `fret-ui` `VirtualList`)
- P1: Theme/tokens (typed core tokens + extensible namespaced keys for component ecosystems) (ADR 0032 / ADR 0050). (prototype implemented)
- P0: Component size/density system (Tailwind-like scales) so control heights/paddings converge before scaling the UI kit (ADR 0056). (prototype implemented; see `docs/archive/mvp/active-plan.md` MVP 47)
- P1: Anchored overlays (popover-style) for component primitives.
  - Note: overlay surfaces now live in the components layer (`fret-ui-shadcn` surface on top of `fret-ui-kit`), while `fret-ui` retains the overlay-layer mechanism.
- P1: Spin up `fret-components` repo workspace and establish a **general-purpose** component library baseline (ADR 0037):
  - `fret-ui-kit` (token-driven infrastructure + composition helpers) and `fret-ui-shadcn` (shadcn/ui v4-aligned component surface) (prototype implemented in-tree),
  - `fret-icons` (icon registry + sets; atlas-friendly; renderer-agnostic) (prototype implemented in-tree),
  - optional later: `fret-components-editor` (editor-only patterns: inspector/table/tree wrappers).

Notes:

- Prefer “Tailwind-like primitives” as an internal authoring model: small typed tokens + recipe/variant composition (not CSS strings).
- Keep the primitive vocabulary explicit and typed (e.g. `Space`, `Radius`) backed by theme extension tokens
  (`component.space.*`, `component.radius.*`), so component APIs stay stable and theme authors can tune scales.
- `repo-ref/fret-ui-precision` is the design reference for token taxonomy and component recipes; `repo-ref/gpui-component` is the Rust ecosystem reference for themes + component ergonomics.
- `repo-ref/ui` (shadcn/ui) and `repo-ref/tailwindcss` provide upstream vocabulary for component variants and token scales.

### M3 — Display List Contract + Renderer MVP

- P0: Backend-agnostic `Scene/DisplayList` contract in `fret-core`.
- P0: `Scene.ops` ordering is authoritative; renderer batching preserves order. (see ADR 0009)
- P0: Renderer consumes display list and draws:
  - instanced quads,
  - rounded rect via SDF, borders (pending),
  - clip rect stack (initially via scissor; later refine). (MVP done for quads + scissor)
- P0: Dynamic GPU buffer strategy (anticipate multi-buffering/pools for async presentation).
- P1: Atlas allocation and uploads (images, glyphs).
- P0: Render target registry for engine viewports (contract skeleton). (done)

### M4 — Docking (Multi-Window + Tear-off)

- P0: `DockManager` (App-level) owns dock graph for all windows.
- P0: Dock UX in a `DockSpace` widget (split + tabs + drag drop zones, tab insert/reorder, split handle dragging). (MVP done in demo; multi-window tear-off supported)
- P0: Dock panel content wiring (panel kind/key → UI root per window). (MVP done in demo; `DockPanelContentService`)
- P0: Floating windows are first-class: tear-off and merge back. (MVP done in demo via `DockOp` + `WindowRequest`)
- P0: Cross-window drag state and drop target rendering. (done; app-scoped internal `DragSession` + drop overlay)
- P0: Dock persistence (layout.json v1) + stable panel identity. (MVP done in demo; persists/restores)
- P0: Layout reset + minimal layout preset save/load (“last”). (done in demo; menu-backed actions)

Remaining work (still P0, but can iterate after MVP2):

- Persist/restore window placement (monitor + DPI-aware geometry) separate from the logical dock layout. (prototype implemented; stored as `DockLayoutWindow.placement`)
- Improve hit-testing + drop-zone heuristics and polish the UX (snap thresholds, preview animations).
  - “Product UI” affordances to match Unity/Godot:
    - dock drag hint overlay and tab drop indicators (Godot-style),
    - dock context menu actions (float, close, move left/right, etc.),
    - debounced layout persistence (delay disk writes during interactive drags).
    - system cursor affordances for split/resize handles (MVP 46).

### M5 — Engine Viewports

- P0: Host-provided `WgpuContext` to support both editor-hosted and engine-hosted integration. (see ADR 0010)
- P0: Viewport widget that displays an engine texture/render target.
- P0: Viewport mapping + input event contract (window -> uv/px). (prototype implemented)
- P0: Engine frame hook returns command buffers + explicit render target deltas; runner applies deltas before UI render. (prototype implemented; ADR 0038)
- P1: Overlay composition primitives: UI can render overlays over the viewport; editor apps can build gizmos/selection on top.
- P1: Input forwarding and capture rules (mouse/keyboard routed to viewport when focused).

Notes:

- A minimal overlay prototype exists (viewport hover crosshair + marquee selection rectangle in dock panels).

### M6 — Text System Upgrade

- P1: Basic text for inspector/property panels (layout + glyph atlas). (MVP done in demo; see `docs/archive/mvp-archive.md`)
- P0: Text system boundary (`TextBlobId` + metrics contract). (done; see ADR 0006)
- P2: Parley-based text system v2 (attributed spans + wrapper-owned layout) (ADR 0142). (in progress)
- P2: Shaped-run caching + incremental atlas uploads for large documents.
- P1: IME plumbing (winit `Ime` events + `set_ime_cursor_area` feedback path). (prototype implemented; see ADR 0012)
- P2: IME composition UI (inline preedit rendering in text widgets). (MVP done for single-line)
- P2: High-performance text widgets (code editor view, large buffers).

Immediate next step:

- MVP10 is prototype implemented (text editing baseline).
- MVP11 contract is locked (ADRs 0045/0046) and a multiline validation probe is prototype implemented in `fret-demo` (see `docs/archive/mvp/active-plan.md`).

### M7 — Portability (wasm/WebGPU)

- P2: Platform layer for web canvas surfaces.
- P2: Input/clipboard limitations documented and handled.
- P3: Mobile planning (out of scope for early phases).

### M8 — Settings, Keymap, and Persistence

- P0: File-based configuration model + strong types. (see ADR 0014)
- P0: Dock layout persistence format with versioning. (see ADR 0013)
- P1: Settings UI primitives (token-driven) for inspector + app settings.
- P0: Keymap MVP (bind + route + persist) is implemented in `fret-demo` (see `docs/archive/mvp-archive.md` / ADR 0021 / ADR 0022).
- P0: Keymap v2 sequences + pending bindings (ADR 0043) are prototype implemented.

### M9 — Command UI + Focus + Clipboard (Editor Usability Core)

These are the “you can actually drive the editor” foundations. They are intentionally scheduled early so that
all later UI work inherits the same command/focus/clipboard semantics instead of bespoke widget logic.

- P0: Command palette overlay + minimal menu data model (ADR 0023). (MVP done in demo; see `docs/archive/mvp/active-plan.md` MVP 7)
- P0: Focus traversal and focus scopes (Tab navigation, modal focus trap) (ADR 0020). (see `docs/archive/mvp/active-plan.md` MVP 8)
- P0: Clipboard boundary + text editing commands (text-only first) (ADR 0041). (see `docs/archive/mvp/active-plan.md` MVP 9)

## Module Breakdown (Crates)

### `fret-core` (P0)

- IDs and geometry (`Px`, `Rect`, `NodeId`, resource IDs).
- Dock graph data structures.
- Display list / scene primitives contract.
- Cross-platform input event types (including IME events as data-only).

### `fret-runtime` (P0)

- Host-facing runtime boundary traits and portable value types used by `fret-ui` (ADR 0052).
- Defines the “UI -> host outputs” (`Effect`) and input/command/theme/menu boundary types.

### `fret-app` (P0)

- `App` runtime: globals, models, command registry, plugin registry.
- Scheduling/executor abstraction (no hard dependency on Tokio).

### `fret-ui` (P0)

- UI runtime (retained widget tree prototype today; declarative elements planned via ADR 0028).
- Invalidation, layout widgets, hit-testing.
- Focus, capture, command routing.
- Dock UI (`DockSpace`) as a widget consuming `DockManager`.

### `fret-ui-app` (P0)

- Optional integration convenience layer that binds `fret-ui` to `fret-app::App` (type aliases + re-exports).
- Keeps first-party app/demo code ergonomic while the core UI runtime stays embeddable.

### `fret-platform` (P0)

- Platform backend integration (winit today; web later) focused on IO:
  - window creation primitives,
  - event translation,
  - platform services (clipboard, IME, drag-and-drop).

### `fret-launch` (P0)

- Desktop glue that wires `winit` + `wgpu` + `fret-render` + `fret-ui` together.
- Owns the winit event loop and window lifecycle.
- Drains `App::flush_effects()` and drives redraw/raf/timers deterministically across windows.
- Owns surfaces + presentation and coordinates submission order (see ADR 0015 / ADR 0038).
- Provides an engine render hook for recording command buffers, submitted by the runner before UI sampling (ADR 0038).

### `fret-render` (P0)

- wgpu device/surface setup.
- Resource registries for atlas/textures/render targets.
- Pipelines for quads/SDF/text/images.
- Supports updating registered viewport targets (resize/view updates) to keep embedded viewports crisp.

## Definition of Done (per milestone)

- Builds with `cargo check` on macOS/Linux/Windows.
- A runnable demo for user-visible milestones (M0/M2/M3/M4/M5).
- Public API changes documented in `docs/architecture.md` and/or ADR notes.
