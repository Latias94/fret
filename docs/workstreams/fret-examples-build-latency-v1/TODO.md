# Fret Examples Build Latency v1 - TODO

Status: active

- [x] Start a dedicated workstream for examples/demo build latency.
- [x] Move the sortable table source-marker gate out of the monolithic `fret-examples` unit test.
- [x] Move the control-discoverability source-marker gate out of the monolithic `fret-examples`
  unit test.
- [x] Move the IMUI facade / teaching-surface source-marker gate package out of the monolithic
  `fret-examples` unit test.
- [x] Move the table/datatable source-marker gate package out of the monolithic `fret-examples`
  unit test.
- [x] Validate the new lightweight source gate.
- [x] Validate `fret-examples` still compiles after deleting the redundant unit test.
- [x] Audit the remaining source-marker tests and rank migration candidates by compile impact.
- [x] Move the source-tree policy gate package out of the monolithic `fret-examples` unit test.
- [x] Decide whether `fret-demo` needs a split examples crate, feature families, or direct demo-local
  bins for heavy families.
- [x] Move `imui_response_signals_demo` and `imui_interaction_showcase_demo` into
  `apps/fret-examples-imui` in a separate documentation-aware slice.
- [x] Revisit the unconditional `profile.dev.package.fret-examples.incremental = false` setting with
  target-specific evidence.
- [x] Move the IMUI editor proof theme/preset source markers out of the monolithic
  `fret-examples` unit test.
- [x] Move the IMUI local-state and workspace-shell entry source markers out of the monolithic
  `fret-examples` unit test.
- [x] Move the workspace shell capability-helper source markers out of the monolithic
  `fret-examples` unit test.
- [x] Move the broad view-runtime entry source markers out of the monolithic `fret-examples`
  unit test.
- [x] Move the authoring/import source markers out of the monolithic `fret-examples` unit test.
- [x] Move the theme-read source markers out of the monolithic `fret-examples` unit test.
- [x] Move the local-state bridge source markers out of the monolithic `fret-examples` unit test.
- [x] Move the model-read and asset-helper tail source markers out of the monolithic
  `fret-examples` unit test.
- [x] Move the advanced/reference roster source markers out of the monolithic `fret-examples`
  unit test.
- [x] Move the default app surface source markers out of the monolithic `fret-examples` unit test.
- [x] Split the examples source-tree gate implementation behind the existing command entrypoint.
- [x] Move the query, markdown, and editor notes source markers out of the monolithic
  `fret-examples` unit test.
- [x] Move the todo and async playground source markers out of the monolithic `fret-examples`
  unit test.
- [x] Move the API workbench lite source markers out of the monolithic `fret-examples` unit test.
- [x] Split app-facing examples source-policy matrices into an owner module behind the stable gate
  entrypoint.
- [x] Move the low-level interop direct-leaf source markers out of the monolithic `fret-examples`
  unit test.
- [x] Move the manual UiTree root-wrapper source markers out of the monolithic `fret-examples`
  unit test.
- [x] Move the components gallery owner-split source/document markers out of the monolithic
  `fret-examples` unit test.
- [x] Move the selected raw-owner escape-hatch source markers out of the monolithic
  `fret-examples` unit test.
- [x] Move the IMUI editor proof non-raw helper, official adapter, and app-owner source markers out
  of the monolithic `fret-examples` unit test.
- [x] Move the IMUI interaction showcase layout and grouped state/action source markers out of the
  monolithic `fret-examples` unit test.
- [x] Move the IMUI response signals lifecycle, canonical trigger, and grouped state/action source
  markers out of the monolithic `fret-examples` unit test.
- [x] Move the IMUI P0 response/key-owner workstream document freeze markers out of the monolithic
  `fret-examples` unit test.
- [x] Move the IMUI collection/pane proof workstream document and proof-surface markers out of the
  monolithic `fret-examples` unit test.
- [x] Move the IMUI facade internal modularization workstream document/index markers out of the
  monolithic `fret-examples` unit test.
- [x] Move the IMUI collection box-select workstream document/index markers out of the monolithic
  `fret-examples` unit test while keeping the real `proof_collection_*` unit tests in Rust.
- [x] Move the IMUI collection keyboard-owner workstream document/index markers out of the
  monolithic `fret-examples` unit test while keeping the real `proof_collection_keyboard_*` unit
  tests in Rust.
- [x] Move the IMUI collection delete-action workstream document/index markers out of the
  monolithic `fret-examples` unit test while keeping the real `proof_collection_delete_*` unit
  tests in Rust.
- [x] Move the IMUI collection context-menu workstream document/index markers out of the
  monolithic `fret-examples` unit test while keeping the real `proof_collection_context_menu_*`
  unit tests in Rust.
- [x] Move the IMUI collection zoom workstream document/index markers out of the monolithic
  `fret-examples` unit test while keeping the real `proof_collection_layout_metrics_*` and
  `proof_collection_zoom_request_*` unit tests in Rust.
- [x] Move the IMUI collection select-all workstream document/index markers out of the monolithic
  `fret-examples` unit test while keeping the real `proof_collection_select_all_*` unit tests in
  Rust.
- [x] Move the IMUI collection rename workstream document/index markers out of the monolithic
  `fret-examples` unit test while keeping the real `proof_collection_*rename*` unit tests in Rust.
- [x] Move the IMUI collection inline-rename workstream document/index markers out of the
  monolithic `fret-examples` unit test while keeping the real rename/inline-rename unit tests in
  Rust.
- [x] Move the IMUI collection modularization workstream document/source-boundary markers out of
  the monolithic `fret-examples` unit test while keeping the real collection behavior unit tests in
  Rust.
- [x] Move the IMUI collection command-package workstream document/index markers out of the
  monolithic `fret-examples` unit test while keeping the real duplicate/rename behavior unit tests
  in Rust.
- [x] Move the IMUI collection second proof-surface workstream document/source-shape markers out of
  the monolithic `fret-examples` unit test while keeping the real shell-mounted surface tests in
  Rust.
- [x] Move the IMUI collection helper-readiness workstream document/no-helper-widening markers out
  of the monolithic `fret-examples` unit test while keeping the real proof-surface tests in Rust.
- [x] Move the IMUI editor-notes inspector command workstream document/source-shape markers out of
  the monolithic `fret-examples` unit test while keeping the real editor rail surface test in Rust.
- [x] Move the IMUI editor-notes dirty status workstream document/source-shape markers out of the
  monolithic `fret-examples` unit test while keeping the real editor rail/device shell surface
  tests in Rust.
- [x] Move the IMUI next-gap audit workstream decision markers out of the monolithic
  `fret-examples` unit test; this closed audit has no real Rust behavior test.
- [x] Move the IMUI editor-notes draft actions workstream document/source-shape markers out of the
  monolithic `fret-examples` unit test while keeping the real editor rail/device shell surface
  tests in Rust.
- [x] Move the IMUI TextField draft-buffer contract audit document/source-shape markers out of the
  monolithic `fret-examples` unit test; this closed no-public-API audit has no real Rust behavior
  test.
- [x] Move the IMUI TextField draft-controller API proof document/source-shape markers out of the
  monolithic `fret-examples` unit test while keeping the real API smoke, editor surface, and
  launched diagnostics gates outside that source freeze.
- [x] Move the IMUI child-region depth workstream document/index markers out of the monolithic
  `fret-examples` unit test while keeping the real `fret-ui-kit`, `fret-imui`, and pane-proof
  behavior gates outside that source freeze.
- [x] Move the IMUI menu/tab trigger response-surface workstream document markers out of the
  monolithic `fret-examples` unit test while keeping the real `fret-imui` and demo/source floors.
- [x] Move the IMUI menu/tab trigger response canonicalization workstream document markers out of
  the monolithic `fret-examples` unit test while keeping the real canonical helper behavior and
  IMUI facade teaching source gates.
- [x] Move the IMUI workbench shell closure workstream source-policy package out of the monolithic
  `fret-examples` unit test while keeping the real shell surface tests and launched diagnostics
  floor.
- [x] Move the IMUI P2 diagnostics/tooling source-policy package out of the monolithic
  `fret-examples` unit test while keeping the real `fret-diag`, launched diagnostics, DevTools,
  and campaign doctor gates.
- [x] Move the IMUI P3 multi-window runner-gap and bounded campaign package source-policy checks
  out of the monolithic `fret-examples` unit test while keeping real campaign validate/run gates.
- [x] Move the docking P3 source-policy subset out of the monolithic `fret-examples` unit test
  while keeping real owner-crate, campaign, and host-admitted behavior gates.
- [x] Move the docking mixed-DPI support note source-policy checks out of the monolithic
  `fret-examples` unit test while keeping the real host-admitted mixed-DPI proof surfaces.
- [x] Move the diagnostics environment source-policy checks out of the monolithic
  `fret-examples` unit test while keeping the real owner-crate behavior gates.

## Parked

- Removing launched diagnostics gates.
- Changing IMUI sortable table behavior.
- Splitting public framework crates.
