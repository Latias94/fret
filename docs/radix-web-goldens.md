# Radix Primitives Web Goldens (JSON)

This document proposes a “Radix primitives golden” layer that complements the existing shadcn
component goldens.

## Do we need it?

Yes, if the goal is to replicate Radix primitives behavior headlessly (ADR 0089) with high
confidence.

shadcn goldens primarily answer: “does the recipe look/lay out like upstream?”

Radix primitives goldens answer: “does the state machine behave like upstream under keyboard,
pointer, focus, and accessibility semantics?”

You *can* rely on shadcn pages to implicitly exercise Radix behaviors, but dedicated Radix goldens
are useful because:

- they isolate a primitive (smaller repro, easier diff/debug),
- they define interaction sequences (open/close, roving focus, escape/outside-press),
- they validate ARIA semantics (roles, `aria-*`, `data-state`) independent of styling.

## What to capture (recommended)

Prefer behavior + semantics over pixel diffs:

- **Semantics snapshot**: `role`, `aria-*`, `data-state`, `disabled`, `selected`, `expanded`,
  and a stable element selector for each node.
- **Focus snapshot**: active element identity (selector/path) and key attributes.
- **Geometry (optional)**: bounding boxes for key nodes, mainly to validate layout-driven behaviors
  (e.g. popover placement anchoring).

The current extractor can optionally capture per-node viewport-relative rects (`getBoundingClientRect`)
for every “interesting” node included in the snapshot tree. This is intended for **layout-driven**
contracts (e.g. overlay placement + available-space clamping) and should use tolerances.

## Golden format (suggested)

Use a timeline format so we can express interaction sequences:

```json
{
  "version": 1,
  "primitive": "dialog",
  "scenario": "open-close-keyboard",
  "steps": [
    { "action": { "kind": "load" }, "snapshot": { "...": "..." } },
    { "action": { "kind": "press", "key": "Enter" }, "snapshot": { "...": "..." } },
    { "action": { "kind": "press", "key": "Escape" }, "snapshot": { "...": "..." } }
  ]
}
```

## Extraction approach (web)

Use Playwright/Puppeteer against `repo-ref/ui/apps/v4`:

1. Load a dedicated primitive demo page (or a shadcn registry example that isolates the primitive).
2. Execute a scripted sequence (click, keydown, pointer down outside, tab traversal).
3. After each step, extract:
   - subtree semantics (`role`, `aria-*`, `data-state`),
   - focus (`document.activeElement`),
   - key rects (optional).

## Validation approach (Fret)

Mirror the same sequence in Fret tests:

- drive Fret events (pointer/key) against the headless primitive,
- produce a Fret semantics snapshot (ADR 0033),
- normalize to the same JSON schema as the web extractor,
- compare snapshots per step with tolerances/normalization rules.

## Relationship to shadcn goldens

- shadcn goldens: “style/layout conformance for composed components”
- radix goldens: “interaction/semantics conformance for primitives”

Both are valuable, and they fail for different reasons.
