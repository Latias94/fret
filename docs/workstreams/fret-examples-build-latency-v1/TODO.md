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

## Parked

- Removing launched diagnostics gates.
- Changing IMUI sortable table behavior.
- Splitting public framework crates.
