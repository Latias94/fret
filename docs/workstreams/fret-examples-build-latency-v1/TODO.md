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

## Parked

- Removing launched diagnostics gates.
- Changing IMUI sortable table behavior.
- Splitting public framework crates.
