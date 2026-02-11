# ADR 0154: Ecosystem Crate Taxonomy (Glue vs Kit vs Primitives) and `fret-ui-kit` Split (v1)

Status: Proposed
Scope: Workspace ergonomics and ecosystem boundaries (no kernel contract changes in v1)

## Context

Fret is intentionally layered:

- `crates/*` are kernel contracts and portable mechanisms (ADR 0092).
- `ecosystem/*` are policy-heavy surfaces and reusable libraries that can evolve faster (ADR 0027).
- runner/platform integration lives in backend crates (`fret-launch`, `fret-runner-*`, `fret-platform-*`).

As the ecosystem grows, two friction points become visible:

1) **Glue code duplication across ecosystems**

Multiple ecosystem crates need the same “bridge” logic between stable kernel boundaries. Examples:

- Viewport tools need unit-explicit input mapping and drag state (`ViewportInputEvent` → tool input), independent of gizmo logic.
- Resource caches need a stable “key → bytes → ID → flush point” story (`fret-ui-assets`).
- Accessibility needs a bridge crate (`fret-a11y-accesskit`).

When this glue lives inside a domain crate (e.g. inside `fret-gizmo`) it creates unwanted coupling: other ecosystems
may depend on a domain crate just to reuse generic mapping utilities.

2) **`fret-ui-kit` is becoming a “kitchen sink”**

`ecosystem/fret-ui-kit` currently contains:

- headless primitives (state machines),
- low-level UI primitives,
- overlay orchestration substrates,
- policy conveniences and recipes.

This is convenient early, but it creates pressure:

- “I want one headless primitive” can pull in large policy surfaces.
- Change velocity of policy/recipes can destabilize primitives that should become relatively stable building blocks.

We want a crate taxonomy that supports a large plugin/component ecosystem (egui/imgui-style richness) while keeping
dependencies and stability expectations clear.

## Goals

- Establish a **clear taxonomy** for ecosystem crates:
  - what belongs in kernel vs ecosystem vs glue,
  - how to name new crates consistently.
- Reduce unwanted coupling by extracting reusable glue.
- Enable `fret-ui-kit` to remain ergonomic without forcing all consumers to depend on the full kit surface.
- Preserve the “golden path” story: small apps should stay easy (ADR 0106 / ADR 0110).

## Non-goals

- Renaming existing published crates in v1 (avoid churn).
- Forcing a single component repository strategy (ADR 0037).
- Introducing new `fret-core` / `fret-runtime` contracts as part of this ADR.

## Decision

### 1) Adopt an ecosystem crate taxonomy with naming conventions

We standardize three ecosystem crate categories and suffix conventions:

#### A) **Glue crates** (`*-tooling`, `*-bridge`, `*-app`)

Glue crates exist to connect stable boundaries and eliminate repeated boilerplate. They should be:

- policy-light (no app-specific defaults),
- portable (avoid backend deps unless explicitly a runner glue),
- stable-ish once adopted by multiple ecosystems.

Recommended suffixes:

- `fret-*-tooling`: unit/coordinate/input mapping helpers for “tool” integrations.
  - Example: `fret-viewport-tooling` (Tier A viewport tool input mapping).
- `fret-*-bridge`: platform/third-party bridges.
  - Example: `fret-a11y-accesskit`.
- `fret-*-app`: core-to-core integration glue between app/runtime layers.
  - Example: `fret-ui-app`.

#### B) **Primitives crates** (`*-primitives`, `*-headless`)

Primitives crates provide reusable building blocks with minimal policy:

- headless state machines (roving focus, typeahead, dismissible),
- low-level UI primitives that are not domain-specific.

Recommended suffixes:

- `fret-ui-primitives`: preferred umbrella name for UI-oriented primitives *if/when extracted as a
  standalone crate*.
- `fret-ui-headless`: acceptable if we want a narrower scope explicitly limited to state machines.

#### C) **Kit/policy crates** (`*-kit`, `*-bootstrap`, `*-shadcn`)

Kit crates bundle defaults, ergonomic helpers, and policy surfaces. They may:

- depend on primitives and glue crates,
- provide re-exports for convenience,
- evolve faster than primitives.

Examples:

- `fret-ui-kit` (policy + ergonomic declarative helpers),
- `fret-bootstrap` (golden-path startup wiring),
- `fret-ui-shadcn` (taxonomy/recipes layer).

### 2) Split `fret-ui-kit` into “primitives” and “policy” surfaces (without breaking paths in v1)

We want a clear distinction between:

- **UI wiring primitives** (Radix-aligned, policy-light, reusable across design systems), and
- **kit/policy surfaces** (tokens, recipes, opinionated helpers).

Implementation note (v1):

- If a standalone primitives crate does not yet represent a real seam (single in-tree consumer and
  no extraction story), keep the primitives surface in `ecosystem/fret-ui-kit/src/primitives/*`
  and `ecosystem/fret-ui-kit/src/declarative/*` to avoid accidental micro-crates.
- If/when extraction becomes justified, the preferred crate name is:
  - **Recommended**: `ecosystem/fret-ui-primitives`

Responsibilities:

- Keep headless state machines in `ecosystem/fret-ui-headless`.
- Keep UI wiring primitives in a stable, Radix-named surface (initially within `fret-ui-kit`, with a
  future extraction to `fret-ui-primitives` if warranted).
- Keep dependencies minimal for primitives (avoid pulling in heavy recipe/policy modules).

`fret-ui-kit` remains:

- the ergonomic policy surface,
- the “golden path” component authoring helper crate,
- and (if `fret-ui-primitives` exists) a re-export facade to keep `fret_ui_kit::primitives::*`
  working.

Migration plan (v1):

- Keep public module paths in `fret-ui-kit` via `pub use` re-exports.
- Move implementations incrementally.
- Only consider a future deprecation/rename once adoption stabilizes (out of scope for v1).

### 3) Keep viewport tooling glue separate from UI kit primitives

Viewport tooling glue is not inherently UI-only: it is used by engine/driver tool loops and domain ecosystems.

Therefore:

- `fret-viewport-tooling` remains its own glue crate.
- `fret-ui-kit` and tool ecosystems may re-export or depend on it for convenience.

### 4) Evaluation rubric for “should this be a new crate?”

Create a new glue/primitives crate when:

- two or more ecosystem crates copy the same mapping logic, and
- the code primarily bridges stable boundaries (units, IDs, effect/event wiring), and
- it can stay policy-light.

Keep it inside a domain crate when:

- it encodes domain semantics (e.g. gizmo picking policy, chart scales),
- or it depends on domain types heavily and would be hard to stabilize.

## Consequences

### Benefits

- Clear expectations for stability and dependencies:
  - primitives stable-ish, kits fast-moving, glue focused and reusable.
- Reduced “ecosystem coupling”: chart/plot/gizmo ecosystems can share glue without depending on each other.
- Better long-term scalability toward an egui-like ecosystem composition model.

### Costs

- More crates to navigate.
- Requires discipline to keep glue/policy boundaries clean.

## Alternatives Considered

### A) Keep everything in `fret-ui-kit`

Pros: fewer crates.

Cons: dependency bloat and unclear stability; harder for third-party component authors to depend on “just primitives”.

### B) Move glue into kernel crates

Rejected: increases churn and risks locking editor-specific policy into hard-to-change contracts.

## References

- Kernel vs ecosystem scope: `docs/adr/0027-framework-scope-and-responsibilities.md`
- Workspace structure: `docs/adr/0092-crate-structure-core-backends-apps.md`
- Golden path: `docs/adr/0106-ecosystem-bootstrap-ui-assets-and-dev-tools.md`, `docs/adr/0110-golden-path-ui-app-driver-and-pipelines.md`
- Viewport tooling glue: `docs/adr/0153-viewport-tooling-host-helpers-and-arbitration-v1.md`
- Component ecosystem conventions: `docs/adr/0148-component-ecosystem-authoring-conventions-v1.md`
