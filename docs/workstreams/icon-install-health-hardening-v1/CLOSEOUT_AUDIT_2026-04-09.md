# Closeout Audit — 2026-04-09

Status: closed closeout record

Related:

- `docs/workstreams/icon-install-health-hardening-v1/DESIGN.md`
- `docs/workstreams/icon-install-health-hardening-v1/BASELINE_AUDIT_2026-04-09.md`
- `docs/workstreams/icon-install-health-hardening-v1/M1_CONTRACT_FREEZE_2026-04-09.md`
- `docs/workstreams/icon-install-health-hardening-v1/M2_PROOF_SURFACE_2026-04-09.md`
- `docs/workstreams/icon-install-health-hardening-v1/TODO.md`
- `docs/workstreams/icon-install-health-hardening-v1/MILESTONES.md`
- `docs/workstreams/icon-install-health-hardening-v1/EVIDENCE_AND_GATES.md`
- `docs/workstreams/icon-system-extension-v1/CLOSEOUT_AUDIT_2026-04-09.md`
- `docs/workstreams/generated-icon-presentation-defaults-v1/CLOSEOUT_AUDIT_2026-04-09.md`
- `docs/adr/0065-icon-system-and-asset-packaging.md`
- `docs/adr/IMPLEMENTATION_ALIGNMENT.md`
- `ecosystem/fret-icons/src/lib.rs`
- `ecosystem/fret-ui-kit/src/declarative/icon.rs`
- `ecosystem/fret-bootstrap/src/lib.rs`
- `ecosystem/fret-icons-lucide/src/app.rs`
- `ecosystem/fret-icons-radix/src/app.rs`
- `crates/fret-icons-generator/src/templates.rs`

## Verdict

This lane is now closed.

It successfully landed the narrow follow-on that the closed icon-contract lane left open:

- fail-fast explicit install seams,
- explicit metadata-conflict rejection for installed packs,
- best-effort partial helper fallback instead of destructive empty-registry fallback,
- and proof gates that keep first-party, generated, bootstrap, and helper surfaces aligned.

## What shipped

### 1) Explicit install seams now have the correct failure posture

The shipped rule is now explicit:

- explicit install surfaces fail fast when the icon contract is broken,
- rather than publishing a misleading frozen/installed state.

This covers both registry-freeze failure and conflicting installed-pack metadata.

### 2) Helper fallback is still available, but no longer destructive

The shipped helper rule is now explicit too:

- helper-owned non-fallible paths may remain best-effort,
- but they must preserve the valid subset and emit diagnostics.

This closes the old gap where one bad alias loop could erase unrelated valid icons from the helper
snapshot.

### 3) The lane stayed narrow and did not widen into bootstrap redesign

This follow-on did not reopen:

- the runtime icon mechanism,
- generated/imported presentation-default policy,
- or a broad `Result` conversion of the app/bootstrap setup lifecycle.

That keeps the hardening in the correct layer and avoids reopening a much larger integration
decision without evidence.

### 4) ADR and alignment wording now match the shipped semantics

ADR 0065 and the implementation alignment note now explicitly state:

- strict explicit install seams,
- partial helper fallback,
- and the guarantee that helper fallback must not discard a usable registry because of unrelated
  invalid entries.

## Gates that define the shipped surface

- `cargo nextest run -p fret-icons -p fret-ui-kit -p fret-bootstrap -p fret-icons-lucide -p fret-icons-radix -p fret-icons-generator -p fretboard`
- `python3 tools/check_layering.py`
- `git diff --check`

## Follow-on policy

Do not reopen this lane for:

- broad `Result` plumbing across `.setup(...)` / bootstrap lifecycle,
- new diagnostics/catalog features that consume installed-pack metadata,
- or presentation-policy changes for imported/generated packs.

If future work is needed, open a narrower follow-on such as:

1. a dedicated app/bootstrap error-reporting surface,
2. tooling that consumes `InstalledIconPacks` for diagnostics or package listing,
3. or a new proof lane for richer authored-icon demos/diagnostics that does not change install
   semantics.
