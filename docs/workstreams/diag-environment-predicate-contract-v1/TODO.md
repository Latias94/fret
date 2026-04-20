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
- [ ] DEPC-013 Define the environment-source provenance/timing lane needed before manifest syntax
  lands.
- [ ] DEPC-014 Choose the smallest additive diagnostics manifest shape once the first concrete
  source and timing contract are both ready.

## Living-doc alignment

- [x] DEPC-020 Update the living diagnostics docs so they state that `debug.environment` is not a
  campaign preflight contract.
- [x] DEPC-021 Add a source-policy gate so future refactors cannot silently collapse the taxonomy.

## Deferred by design

- [ ] DEPC-030 Do not implement actual environment predicate execution in this lane until a first
  concrete source/timing/grammar slice is chosen.
