# Diag Environment Predicate Contract v1 - TODO

Status: Active

## Baseline and taxonomy

- [x] DEPC-001 Record the current environment-lane taxonomy and why `requires_capabilities`
  remains the only shipped preflight contract.
- [x] DEPC-002 Freeze the no-erased-runtime-family verdict for existing environment snapshots.

## Contract design

- [x] DEPC-010 Define the owner split for future diagnostics environment predicates.
- [x] DEPC-011 Define the admission rule for promoting an environment source into a
  predicate-capable source.
- [x] DEPC-012 Choose the first concrete source candidate and record why syntax remains deferred.
- [x] DEPC-013 Define the environment-source provenance/timing lane needed before manifest syntax
  lands.
- [x] DEPC-015 Land additive protocol + filesystem loader foundations for an environment-source
  catalog without introducing manifest consumers.
- [x] DEPC-016 Publish `host.monitor_topology` as a launch-time filesystem source at the
  diagnostics `out_dir` root.
- [x] DEPC-017 Surface `environment_sources_path`,
  `environment_source_catalog_provenance`, and `environment_sources` in campaign
  summary/result/aggregate artifacts without introducing manifest consumers.
- [x] DEPC-018 Land a transport/session environment-source query foundation before manifest syntax
  so existing DevTools sessions can publish admitted sources without overloading hello/session
  capabilities.
- [x] DEPC-014 Choose the smallest additive diagnostics manifest shape once the first concrete
  source and timing contract are both ready.

## Living-doc alignment

- [x] DEPC-020 Update the living diagnostics docs so they state that `debug.environment` is not a
  campaign preflight contract.
- [x] DEPC-021 Add a source-policy gate so future refactors cannot silently collapse the taxonomy.

## Deferred by design

- [x] DEPC-030 Implement the first concrete environment predicate execution slice only after the
  first source/timing/grammar decision is frozen.
- [ ] DEPC-031 Do not widen `requires_environment` into a generic boolean expression language until
  a second admitted source proves the need.
