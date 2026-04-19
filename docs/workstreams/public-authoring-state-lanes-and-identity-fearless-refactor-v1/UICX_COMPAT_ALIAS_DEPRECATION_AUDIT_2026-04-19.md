# UiCx Compatibility Alias Deprecation Audit - 2026-04-19

## Question

After the default and advanced preludes stopped teaching `UiCx<'a>` and the grouped helper
extensions moved to `AppRenderActionsExt` / `AppRenderDataExt`, should the root `UiCx<'a>` alias
remain a neutral compatibility noun, or should the repo make its compatibility-only status explicit?

## Findings

### 1) The remaining `UiCx` runtime tail is no longer a first-party authoring need

The current first-party app/example/gallery surface is already off `UiCx<'a>`:

- `fret::app::prelude::*` does not reexport `UiCx`,
- `fret::advanced::prelude::*` now teaches `AppComponentCx<'a>` instead,
- shipped UI Gallery snippets/pages use `AppComponentCx<'a>`,
- grouped app-render helper imports now use `AppRenderActionsExt` / `AppRenderDataExt`.

What remained active was the root alias itself plus stale tooling that still enforced `UiCx<'_>` on
default teaching snippets.

### 2) The still-live tooling had become actively misleading

Before this slice, running:

```bash
python3 tools/gate_no_raw_app_context_in_default_teaching_snippets.py
```

failed across the curated UI Gallery snippet set because the gate still required `UiCx<'_>` even
though those snippets had already migrated to `AppComponentCx<'_>`.

That means the repo had already decided the canonical app-hosted snippet lane, but one of the
release-facing policy scripts was still steering maintainers back to the old name.

### 3) Deletion is still less correct than explicit deprecation

The current evidence justifies shrinking `UiCx<'a>` to a deprecated compatibility seam, not blind
removal:

- external consumers may still import it explicitly,
- the repo has not yet recorded a release-window removal plan,
- and the migration goal is now clear enough that the warning itself can do useful work.

## Decision

Land the next root-alias slice as:

- `UiCx<'a>` stays available temporarily, but is explicitly `#[deprecated]`,
- the deprecation note points callers toward `AppComponentCx<'a>`, `AppRenderCx<'a>`, or
  `AppRenderContext<'a>` depending on helper shape,
- public docs describe `UiCx` as a deprecated compatibility alias rather than neutral app-lane
  vocabulary,
- and the default-snippet gate / pre-release hook now enforce `AppComponentCx<'_>` instead of
  `UiCx<'_>`.

## Outcome

This slice updates:

- `ecosystem/fret/src/lib.rs`
  - marks `UiCx<'a>` deprecated with migration guidance,
- `tools/gate_no_raw_app_context_in_default_teaching_snippets.py`
  - now requires `AppComponentCx<'_>` and rejects `UiCx<'_>` / raw app context spellings,
- `tools/pre_release.py`
  - now reports the gate under the canonical `AppComponentCx` wording,
- public docs / ADRs
  - now describe `UiCx` as a deprecated compatibility alias.

## Follow-on

The next decision is no longer naming. It is release policy:

- keep the deprecated alias for one or more release windows while external consumers migrate, or
- remove it once downstream evidence shows the warning has served its purpose.
