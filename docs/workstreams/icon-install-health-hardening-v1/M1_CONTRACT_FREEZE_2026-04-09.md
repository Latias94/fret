# M1 Contract Freeze — 2026-04-09

Status: accepted decision

Related:

- `docs/workstreams/icon-install-health-hardening-v1/DESIGN.md`
- `docs/workstreams/icon-install-health-hardening-v1/BASELINE_AUDIT_2026-04-09.md`
- `docs/workstreams/icon-install-health-hardening-v1/TODO.md`
- `docs/workstreams/icon-install-health-hardening-v1/MILESTONES.md`
- `docs/workstreams/icon-install-health-hardening-v1/EVIDENCE_AND_GATES.md`
- `docs/adr/0065-icon-system-and-asset-packaging.md`
- `docs/adr/IMPLEMENTATION_ALIGNMENT.md`

## Decision

Freeze the install-health split like this:

### 1) Explicit install surfaces are fail-fast

These surfaces must fail fast if the icon contract is broken:

- first-party `crate::app::install(...)`,
- generated pack `crate::app::install(...)`,
- `BootstrapBuilder::register_icon_pack_contract(...)`,
- and bootstrap's raw `register_icon_pack(...)` escape hatch.

Failure reasons in scope:

- registry freeze failure,
- conflicting installed-pack metadata for the same `pack_id`.

Rationale:

- these are explicit author-facing seams;
- publishing a “successful” install with invalid registry state is more misleading than a panic;
- and broad `Result` plumbing is out of scope for this lane.

### 2) Runtime/helper fallback remains best-effort and partial

These paths may remain non-fallible:

- helper-owned lazy freeze/default fallback,
- preload helpers that work against global registry state,
- and similar convenience surfaces that do not own app/bootstrap lifecycle shape.

But their semantics must change from “all-or-nothing default” to:

- keep the valid subset,
- emit diagnostics for invalid entries,
- and never discard usable icons just because unrelated entries are broken.

Rationale:

- these helpers exist precisely because the surrounding API is not fallible;
- preserving the valid subset is the least misleading best-effort behavior.

### 3) Metadata conflict becomes a real contract error

`InstalledIconPacks` must reject the same `pack_id` resolving to different metadata.

Allowed behavior:

- same metadata repeated: dedupe
- conflicting metadata: explicit error

Rationale:

- pack provenance is now explicit contract surface, not only an internal cache.

### 4) No broad fallible setup redesign in this lane

Do not change:

- `.setup(...)`,
- `init_app(...)`,
- or the broader bootstrap/app lifecycle

to return `Result` in this lane.

Rationale:

- that is a much wider integration decision than the install-health problem itself;
- this lane should close on the smallest durable semantic split, not on a speculative platform-wide
  redesign.

## Required proof for closeout

Close this lane only after the proof surface demonstrates:

1. strict explicit install behavior,
2. partial helper fallback behavior,
3. metadata-conflict rejection,
4. first-party + generated pack install alignment,
5. and updated ADR/alignment wording.
