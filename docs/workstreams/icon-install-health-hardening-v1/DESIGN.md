# Icon Install Health Hardening v1

Status: Closed
Last updated: 2026-04-09

Related:

- `TODO.md`
- `MILESTONES.md`
- `EVIDENCE_AND_GATES.md`
- `BASELINE_AUDIT_2026-04-09.md`
- `M1_CONTRACT_FREEZE_2026-04-09.md`
- `M2_PROOF_SURFACE_2026-04-09.md`
- `CLOSEOUT_AUDIT_2026-04-09.md`
- `docs/workstreams/icon-system-extension-v1/CLOSEOUT_AUDIT_2026-04-09.md`
- `docs/workstreams/generated-icon-presentation-defaults-v1/CLOSEOUT_AUDIT_2026-04-09.md`
- `docs/adr/0065-icon-system-and-asset-packaging.md`
- `docs/adr/IMPLEMENTATION_ALIGNMENT.md`

Status note (2026-04-09): this lane is now closed on a narrow hardening split:

- explicit install seams fail fast on registry-freeze failure and installed-pack metadata
  conflicts,
- helper/runtime fallback preserves the valid subset and emits diagnostics,
- and the broader app/bootstrap lifecycle remains non-fallible in this slice.

Read the landed proof in `M2_PROOF_SURFACE_2026-04-09.md` and the final verdict in
`CLOSEOUT_AUDIT_2026-04-09.md`.

This lane is a narrow follow-on to the closed `icon-system-extension-v1` lane.
It does not reopen the shipped icon contract, the multicolor runtime split, or the generated-pack
presentation-defaults contract.

It owns one narrower question:

> once Fret has explicit icon-pack metadata, explicit install seams, and a frozen icon registry,
> how should it handle invalid icon entries or conflicting pack metadata so apps fail correctly
> without letting one bad entry silently poison the whole icon surface?

## Why this lane exists

The shipped icon work closed the big contract questions:

- semantic `IconId`,
- explicit pack metadata and install surfaces,
- explicit `SvgIcon` vs `SvgImage` runtime behavior,
- and generated/imported pack output that carries render intent.

That closure made a smaller install-health gap visible:

- explicit install surfaces still had “best-effort” fallback behavior in some paths,
- `InstalledIconPacks` still treated metadata conflicts as debug-only invariants,
- and the helper fallback that existed for non-fallible runtime convenience would replace a
  partially valid registry with an empty one.

Those outcomes are wrong in different directions:

- app/bootstrap install seams should fail loudly when the contract is broken,
- but runtime helpers that cannot change their surrounding return type should keep the valid subset
  and emit diagnostics rather than destroying a usable registry snapshot.

## Assumptions-first baseline

### 1) Lane ownership

- Area: workstream ownership
- Assumption: this should be a narrow follow-on rather than a reopening of the closed icon contract
  lane.
- Evidence:
  - `docs/workstreams/icon-system-extension-v1/CLOSEOUT_AUDIT_2026-04-09.md`
  - `docs/roadmap.md`
  - `docs/workstreams/README.md`
- Confidence: Confident
- Consequence if wrong: we would blur shipped v1 contract closure with a new execution queue.

### 2) Explicit install seams are allowed to fail fast

- Area: author-facing install contract
- Assumption: `crate::app::install(...)` and bootstrap pack registration are explicit contract
  seams, so a bad pack or metadata conflict should stop installation rather than quietly publish a
  misleading success state.
- Evidence:
  - `ecosystem/fret-bootstrap/src/lib.rs`
  - `ecosystem/fret-icons-lucide/src/app.rs`
  - `ecosystem/fret-icons-radix/src/app.rs`
  - `crates/fret-icons-generator/src/templates.rs`
- Confidence: Confident
- Consequence if wrong: the lane would need a broader fallible app/bootstrap redesign.

### 3) Broad fallible setup is not the right fix in this slice

- Area: integration shape
- Assumption: the surrounding install/setup chain remains `FnOnce(&mut App)` and this lane should
  not widen into a `Result`-based bootstrap redesign.
- Evidence:
  - `ecosystem/fret-bootstrap/src/lib.rs`
  - `crates/fret-launch/src/runner/desktop/runner/run.rs`
  - `docs/workstreams/icon-system-extension-v1/CLOSEOUT_AUDIT_2026-04-09.md`
- Confidence: Confident
- Consequence if wrong: this lane would under-scope the real authoring contract problem.

### 4) Runtime helpers still need a best-effort path

- Area: non-fallible helper surfaces
- Assumption: helper APIs such as lazy freeze/default fallback or preload should remain
  non-panicking and should preserve valid entries when only unrelated icons are broken.
- Evidence:
  - `ecosystem/fret-icons/src/lib.rs`
  - `ecosystem/fret-ui-kit/src/declarative/icon.rs`
  - `docs/adr/0065-icon-system-and-asset-packaging.md`
- Confidence: Likely
- Consequence if wrong: we would force invasive API changes into helper-owned code paths.

### 5) Metadata conflict must become a real contract violation

- Area: provenance contract
- Assumption: once `InstalledIconPacks` is public contract surface, the same `pack_id` cannot
  silently map to different metadata in release builds.
- Evidence:
  - `ecosystem/fret-icons/src/lib.rs`
  - `docs/workstreams/icon-system-extension-v1/CLOSEOUT_AUDIT_2026-04-09.md`
  - `docs/adr/0065-icon-system-and-asset-packaging.md`
- Confidence: Confident
- Consequence if wrong: future tooling would consume inconsistent pack provenance.

## In scope

- Freeze the correct split between fail-fast explicit install seams and best-effort runtime helpers.
- Harden `InstalledIconPacks` so conflicting metadata becomes explicit failure.
- Keep runtime helper fallback useful by preserving the valid subset and warning on bad entries.
- Leave proof gates on first-party packs, generated packs, bootstrap, and helper preload behavior.

## Out of scope

- Redesigning `.setup(...)`, `init_app(...)`, or the broader app bootstrap lifecycle to return
  `Result`.
- Reopening the runtime `SvgIcon` vs `SvgImage` mechanism split.
- Changing presentation-default policy for generated/imported packs.
- Broad diagnostics UX or package-catalog features that may consume installed-pack metadata later.

## Owning layers

- `ecosystem/fret-icons`
  - registry freeze behavior and installed-pack metadata semantics
- `ecosystem/fret-ui-kit`
  - best-effort helper behavior that consumes frozen registries
- `ecosystem/fret-bootstrap`
  - explicit bootstrap-side install contract
- first-party / generated pack install seams
  - `ecosystem/fret-icons-lucide`
  - `ecosystem/fret-icons-radix`
  - `crates/fret-icons-generator`

## Target shipped state

When this lane is done, the following must be true:

1. explicit install seams panic/fail fast on registry-freeze failure;
2. explicit install seams panic/fail fast on installed-pack metadata conflicts;
3. runtime best-effort helpers keep the valid subset and emit diagnostics when some entries are
   broken;
4. no helper path silently clears a usable icon surface because of unrelated invalid entries;
5. ADR 0065 and alignment docs state this split explicitly;
6. proof gates cover registry fallback, preload behavior, first-party install seams, generated
   pack install templates, bootstrap, and layering.
