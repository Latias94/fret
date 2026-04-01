# Resource Loading Release Readiness Fearless Refactor v1 — Milestones

## M0 — Release-readiness audit lane is explicit

Deliverables:

- this follow-on workstream exists as the canonical release-readiness closure lane,
- the release-risk findings are named explicitly,
- the TODO tracker distinguishes hard blockers from follow-on hardening.

Exit criteria:

- a maintainer can answer “what still blocks truthful cross-platform resource loading?” by reading
  this folder alone.

## M1 — wasm bundled-only baseline is locked

Deliverables:

- renderer bootstrap uses bundled-only shaping on wasm,
- the implementation no longer depends on environment-variable coincidence,
- one regression test locks the platform split.

Exit criteria:

- the web runner’s “no system fonts” contract is true in code, not just in comments and ADR text.

## M2 — URL and capability truthfulness are closed

Deliverables:

- built-in product surfaces no longer imply default URL support unless a first-party URL resolver
  actually exists,
- capability docs and default authoring guidance match the shipped host stack.

Exit criteria:

- a contributor can answer “does `AssetLocator::url(...)` work by default?” with one accurate
  sentence and one evidence anchor.

## M3 — Font identity vs load-path drift is reduced

Deliverables:

- startup/runtime font loading has one explicit owner model,
- the relationship between bundled asset identity and renderer byte injection is documented and
  gated,
- release-facing APIs no longer imply a stronger asset-pipeline unification than what actually
  ships.

Status note (2026-03-30):

- stage-1 convergence is landed: startup bundled baselines now publish bundled asset identity,
  resolve startup bytes through the shared runtime asset resolver, and keep post-startup runtime
  font loading on `TextAddFontAssets`.

Exit criteria:

- diagnostics, startup docs, and runtime behavior all describe the same font-loading story.

## M4 — Release closure or explicit deferral

Deliverables:

- web `serif` guarantee is shipped on the default bundled startup lane,
- the web image decode tradeoff is explicitly documented,
- release-facing cache/setup guidance no longer keeps misleading partial-install naming on default
  product surfaces,
- every remaining gap is classified as “closed” or “deferred with named limitation”.

Exit criteria:

- there is no silent release risk left in this area: every known gap is either fixed or openly
  carried as a documented limitation.
