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
- [ ] Move the remaining standalone IMUI demos into `apps/fret-examples-imui` in small packages.
- [ ] Revisit the unconditional `profile.dev.package.fret-examples.incremental = false` setting with
  target-specific evidence.

## Parked

- Removing launched diagnostics gates.
- Changing IMUI sortable table behavior.
- Splitting public framework crates.
