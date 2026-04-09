# M1 Contract Freeze — 2026-04-09

Status: accepted v1 decision

Related:

- `docs/workstreams/iconify-acquisition-prestep-v1/DESIGN.md`
- `docs/workstreams/iconify-acquisition-prestep-v1/BASELINE_AUDIT_2026-04-09.md`
- `docs/workstreams/iconify-acquisition-prestep-v1/TODO.md`
- `docs/workstreams/iconify-acquisition-prestep-v1/MILESTONES.md`
- `docs/workstreams/iconify-acquisition-prestep-v1/EVIDENCE_AND_GATES.md`
- `docs/workstreams/iconify-import-pack-generator-v1/M1_CONTRACT_FREEZE_2026-04-09.md`
- `docs/workstreams/iconify-import-pack-generator-v1/CLOSEOUT_AUDIT_2026-04-09.md`
- `crates/fret-icons-generator/src/contracts.rs`
- `crates/fret-icons-generator/src/iconify.rs`
- `crates/fret-icons-generator/src/templates.rs`
- `crates/fretboard/src/icons/contracts.rs`
- `crates/fretboard/src/icons/mod.rs`
- `ecosystem/fret-icons/src/lib.rs`
- `repo-ref/dioxus-iconify/README.md`
- `repo-ref/dioxus-iconify/src/api.rs`

## Purpose

Freeze the smallest correct v1 contract for explicit remote Iconify acquisition before any code
lands. This note decides:

- what local artifact shape acquisition emits,
- where the public acquisition surface should live,
- which provenance facts must remain reviewable,
- and what the first proof should target.

## Frozen decisions

### 1) Acquisition emits a generator-compatible Iconify collection snapshot plus a separate provenance sidecar

Decision:

- the normative local artifact produced by acquisition is an Iconify-collection-shaped JSON file
  that the existing generator can consume without contract changes;
- acquisition also emits a separate provenance sidecar for facts that do not belong in the
  generator input contract;
- the snapshot may represent:
  - a full collection,
  - or a requested subset in the same collection schema;
- the artifact boundary stays “local files first,” not opaque caches.

Why:

- the closed generator lane already froze local Iconify collection snapshots as a supported input;
- reusing that shape avoids reopening `fret-icons-generator`;
- and a sidecar keeps acquisition-only facts explicit instead of silently overloading pack output or
  relying on ignored JSON fields.

Operational consequence:

- generation remains a second explicit step that reads the local snapshot file;
- provenance survives review even if the generator never reads it.

### 2) The public user-facing entrypoint should be a separate `fretboard icons acquire ...` family, not another `import` mode

Decision:

- acquisition must not be hidden under `fretboard icons import ...`;
- the public CLI family should be distinct, for example `fretboard icons acquire iconify-collection ...`;
- implementation ownership should remain separate from the local-input generator ownership, with a
  reusable helper/library surface behind the CLI rather than packing remote-fetch concerns into
  `fret-icons-generator`.

Why:

- `import` already means “consume local pinned source material and emit a pack crate”;
- acquisition has different failure modes, provenance obligations, and testing needs;
- and keeping the helper/library split avoids another extraction refactor when tests or tools want
  the same acquisition planner without shelling out to a CLI.

Important boundary:

- this decision freezes the separation of concerns, not the final crate name of the helper surface.

### 3) Provenance must record deterministic acquisition facts, not only pack-generation facts

Decision:

The committed provenance sidecar must record at least:

- source kind and URL/template used for acquisition,
- collection prefix,
- whether the request targeted the full collection or an explicit subset,
- the requested icon names or subset specification,
- the resolved icon/alias set written into the snapshot,
- and a digest of the emitted snapshot artifact.

When available, it should also record review-relevant upstream metadata such as:

- author / license details,
- palette or multicolor metadata,
- and collection totals or title metadata.

Determinism rule:

- do not require wall-clock fetch timestamps in the committed contract;
- prefer stable content/digest/source facts that avoid unnecessary churn in repo-committable
  artifacts.

Why:

- the current `pack-provenance.json` only explains generation, not acquisition;
- acquisition needs its own audit trail for review, licensing, and reproducibility;
- and deterministic facts fit Fret's repo-committable posture better than “fetched at” noise.

### 4) The first proof should target one explicit subset snapshot, not a full-collection default

Decision:

- M2 should prove the workflow with one explicit subset snapshot first;
- full-collection acquisition may remain supported later, but it is not the first proof target.

Why:

- subset snapshots are smaller and easier to review,
- they still prove the important contract edge that remote acquisition can produce valid local input
  for the generator,
- and they avoid prematurely blessing “vendor the whole upstream set” as the assumed default user
  workflow.

Operational consequence:

- the first proof must still preserve generator compatibility with the same collection schema,
- but it only needs one small, explicit collection slice.

## Rejected alternatives

### Emit only a provenance file and keep fetched SVG data in a cache

Rejected because:

- it would hide the real generator input behind tool-owned state,
- it weakens reviewability,
- and it fights the existing local-file producer contract.

### Hide remote fetch inside `fretboard icons import iconify-collection`

Rejected because:

- it conflates acquisition with generation,
- it makes failures and provenance harder to reason about,
- and it reopens a generator decision that was already closed.

### Put acquisition metadata into the snapshot JSON as ad-hoc extra fields only

Rejected because:

- the generator does not own those semantics,
- they would be too easy to ignore or drift,
- and a separate sidecar is clearer for humans and tooling.

### Treat full-collection vendoring as the default proof target

Rejected because:

- it increases artifact size and review cost,
- it does not prove more contract surface than a well-formed subset snapshot,
- and it biases the public workflow toward bulk vendoring too early.

## Immediate consequences

From this point forward:

1. acquisition work must preserve the existing local Iconify snapshot input contract;
2. acquisition provenance must be explicit and separate from pack-generation provenance;
3. public CLI design must keep `acquire` separate from `import`;
4. the first implementation slice should prove one subset snapshot end-to-end;
5. multicolor presentation defaults remain out of scope for this lane unless they directly block
   acquisition proof.
