# Bundle Artifact Alias Audit

Status: Draft

Tracking context:

- `docs/workstreams/diag-fearless-refactor-v2/M3_ORCHESTRATION_VOCABULARY_AND_CONTRACT_V1.md`
- `docs/workstreams/diag-fearless-refactor-v2/RESIDUAL_NAMING_AUDIT.md`
- `docs/workstreams/diag-fearless-refactor-v2/ARTIFACT_AND_EVIDENCE_MODEL_V1.md`

## Purpose

This note scopes the remaining `bundle_json` versus `bundle_artifact` naming drift.

Unlike the `script_result_json` case, this surface is not one isolated artifact field. The same
legacy term currently appears across three different layers:

1. persisted run-manifest chunk-index structure,
2. orchestrated output payload aliases,
3. internal UI/MCP variable names.

That means the next step should not be "rename everything called `bundle_json`". The next step
should be deciding which layer is a real persisted contract and which layer is only carrying a
legacy alias for compatibility.

## Main conclusion

The safest next move is:

1. **do not** start with the run-manifest `bundle_json` chunk index rename,
2. first treat orchestrated payload dual-write as the additive cleanup target,
3. keep internal app-state names deferred,
4. only revisit the run-manifest chunk-index field after an explicit contract decision.

Reason:

- the run-manifest chunk index is already consumed directly by lint/doctor flows,
- the payload alias surfaces in `diag_repro`, `diag_repeat`, and many stats payloads are broader
  but lower-risk because they already dual-write both names,
- the UI/MCP state names are not cross-tool contracts.

## Findings by layer

### Layer A — Persisted run-manifest chunk index (`bundle_json`)

Key anchors:

- `crates/fret-diag/src/run_artifacts.rs:177`
- `crates/fret-diag/src/run_artifacts.rs:190`
- `crates/fret-diag/src/run_artifacts.rs:321`
- `crates/fret-diag/src/run_artifacts.rs:351`
- `crates/fret-diag/src/artifact_lint.rs:474`
- `crates/fret-diag/src/commands/doctor.rs:736`

Current state:

- `paths.bundle_artifact` is already canonical for locating the main bundle artifact.
- The same manifest still stores the chunk-index payload under `bundle_json`.
- Chunk files themselves also live under `chunks/bundle_json/...`.

Interpretation:

- This is not only a naming alias. It is a full persisted subtree used for integrity checking and
  recovery.
- Today it behaves more like a storage-format node ("chunked raw bundle.json alias") than a plain
  artifact-location field.

Why this is not the next code slice:

- `artifact_lint` reads it directly.
- `doctor` reads it directly.
- a rename here needs a deliberate contract choice:
  - keep `bundle_json` because it specifically means the raw chunked alias,
  - or introduce a new canonical field for chunked bundle-artifact indexing plus legacy-read
    compatibility.

Recommendation:

- Hold this surface stable for now.
- Revisit only after a short contract note decides whether the chunk index is semantically:
  - "raw `bundle.json` chunk index", or
  - "bundle artifact chunk index".

### Layer B — Orchestrated payload dual-write (`bundle_json` + `bundle_artifact`)

Key anchors:

- `crates/fret-diag/src/diag_repeat.rs:42`
- `crates/fret-diag/src/diag_repeat.rs:810`
- `crates/fret-diag/src/diag_repro.rs:538`
- `crates/fret-diag/src/diag_repro.rs:695`
- `crates/fret-diag/src/diag_repro.rs:821`
- `crates/fret-diag/src/stats/notify_gates.rs:121`

Current state:

- several payloads already expose `bundle_artifact`,
- the same payloads often still dual-write `bundle_json`,
- at least one in-tree consumer (`diag_repeat`) already reads `bundle_artifact` first.

Interpretation:

- this is the closest analogue to the already-landed `triage_artifact` / `triage_json` and
  `share_artifact` / `share_zip` pattern,
- which means it is a good candidate for additive cleanup.

Recommendation:

- prefer `bundle_artifact` as the canonical key in new or touched payloads,
- keep `bundle_json` as a temporary alias where the payload already dual-writes,
- when reading ad hoc payloads, prefer `bundle_artifact` first and add fallback to `bundle_json`
  only where a real reader exists.

Progress update:

- the first Layer B cleanup is now landed in `diag_repro` and `diag_repeat`,
- `diag_repro` run rows and packed-bundle rows now write `bundle_artifact` first while still
  dual-writing `bundle_json`,
- `diag_repeat` regression-item materialization now accepts legacy `bundle_json` as a read alias
  while its emitted payload keeps `bundle_artifact` as the primary field,
- a first shared stats helper now also drives canonical-first dual-write in the notify/gc/stale
  evidence payload family, so those gate payload builders no longer each hand-roll the same
  `bundle_artifact` / `bundle_json` pair inline,
- a second stats adoption slice is now also landed for:
  - `debug_stats_gates`,
  - `frames_index_gates`,
  - `gc_gates_streaming`,
  - `retained_vlist_gates`,
  - `view_cache_gates`,
  - `vlist`,
  - `windowed_rows`,
  which means the shared helper now covers both the initial notify/gc/stale family and a broader
  set of persisted gate-evidence payload builders.
- a third stats adoption slice is now also landed for `ui_gallery_code_editor`, which removes one
  of the largest remaining hand-written Layer B payload families from the residual naming set.
- a fourth stats adoption slice is now also landed for `ui_gallery_markdown_editor`, which removes
  the other large ui-gallery hand-written Layer B payload family from the residual naming set.
- the remaining direct hand-written `bundle_path.display().to_string()` tail in
  `stats/stale.rs` is now also removed, so the `crates/fret-diag/src/stats` tree no longer has
  any direct `bundle_json` payload writes that bypass the shared helper.

Suggested first code slice in this layer:

1. normalize `diag_repro` top-level and per-script payloads so canonical fields are listed first,
2. normalize `diag_repeat` payloads the same way,
3. add one small helper for stats payload builders if we want to avoid repeated hand-written
   dual-write blocks later.

Why this is the best next implementation target:

- it improves user-visible payload consistency,
- it follows the additive migration policy already used elsewhere,
- it avoids breaking the manifest chunk-index contract prematurely.

### Layer C — Internal UI / MCP variable names

Key anchors:

- `apps/fret-devtools/src/native.rs:126`
- `apps/fret-devtools/src/native.rs:138`
- `apps/fret-devtools-mcp/src/native.rs:1686`

Current state:

- names such as `last_bundle_json` or `bundle_json: Option<String>` still exist in app state.

Interpretation:

- these are local implementation names,
- they are not persisted diagnostics contracts by themselves.

Recommendation:

- defer,
- only rename when those modules are already being changed for behavior or UI reasons.

## Priority decision

### P0 — Contract decision, not code churn

Decide whether run-manifest `bundle_json` is:

- an intentionally format-specific chunk-index node, or
- a legacy name that should eventually be replaced by a canonical chunk-index field.

Until that decision is written down, changing Layer A is likely to create churn without reducing
ambiguity.

### P1 — Additive payload cleanup

Use `bundle_artifact` as the clear canonical term across:

- `diag_repro`,
- `diag_repeat`,
- selected stats payloads that already dual-write both keys.

This is the best next landable slice.

### P2 — Internal naming cleanup

Defer DevTools / MCP internal renames.

## Recommended execution order

1. Write one short contract clarification for run-manifest chunk-index semantics.
2. In code, start with orchestrated payload dual-write cleanup, not manifest chunk-index renaming.
3. Add canonical-first reading where any in-tree payload consumer still depends on legacy
   `bundle_json`.
4. Continue Layer B in small batches, preferring remaining stats payload families or targeted
   orchestrated payload readers over any Layer A manifest churn.
5. Treat `stats/*` Layer B payload adoption as effectively complete for the current helper-based
   cleanup goal, and move the next follow-up batch to non-stats payload readers or internal naming
   audits if they still buy clarity.
6. Reassess the DevTools / MCP internal naming cleanup only after Layer B payload adoption is
   materially broader.
7. Leave internal app-state names alone unless another feature already requires touching those
   files.

## Definition of done for the next implementation slice

- `diag_repro` and `diag_repeat` present `bundle_artifact` as the primary field,
- any retained `bundle_json` field is explicitly treated as a legacy alias,
- no run-manifest chunk-index contract is changed accidentally,
- docs can explain why `bundle_json` still exists in Layer A while `bundle_artifact` is canonical
  for Layer B payloads.
