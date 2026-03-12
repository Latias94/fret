# Into-Element Surface — Target Interface State

Status: target state for the pre-release conversion-surface reset
Last updated: 2026-03-12

This document records the intended end state for the authoring conversion surface.

It answers four concrete questions:

1. Which return/conversion names should ordinary app authors learn?
2. Which conversion contract should reusable component authors use?
3. Which raw types remain intentionally explicit?
4. Which current public-looking names should disappear?

## Public Surface Tiers

| Tier | Intended audience | Canonical import | Conversion vocabulary |
| --- | --- | --- | --- |
| App | ordinary app authors | `use fret::app::prelude::*;` | `Ui`, `UiChild`, `.into_element(cx)` as an operation, not a trait taxonomy |
| Component | reusable component authors | `use fret::component::prelude::*;` | one public conversion trait generic over `H: UiHost` |
| Advanced | power users / runtime/interop code | explicit raw imports | `AnyElement`, `Elements`, raw `ElementContext<'_, H>` |

## App Surface

### Target nouns

- `Ui`
- `UiChild`
- `UiCx`
- `AppUi`

### Target rule

Default app-facing docs, templates, and examples should teach:

- `fn render(&mut self, cx: &mut AppUi<'_, '_>) -> Ui`
- `fn helper(cx: &mut UiCx<'_>) -> impl UiChild`

They should not teach:

- `AnyElement`
- `Elements`
- `UiChildIntoElement<App>`
- `UiIntoElement`
- `UiHostBoundIntoElement`
- `UiBuilderHostBoundIntoElementExt`

### Hidden but allowed implementation detail

The app prelude may anonymously import the unified conversion trait needed to keep
`.into_element(cx)` method syntax working.

That import support is not part of the taught product vocabulary.

Transitional note on 2026-03-12:

- when an app-facing helper wants `impl UiChild` but the caller currently holds a host-bound
  builder value, one explicit landing step such as `let card = card.into_element(cx);` is an
  acceptable temporary seam.
- keep that seam explicit at the call site rather than widening app helpers to teach component
  conversion trait names prematurely.

## Component Surface

### Target public conversion contract

The curated component surface should expose one public conversion trait.

Working name:

```rust
pub trait IntoUiElement<H: UiHost>: Sized {
    fn into_element(self, cx: &mut ElementContext<'_, H>) -> AnyElement;
}
```

The final name can still shift during implementation, but the target contract must remain:

- host parameter lives on the trait,
- both host-agnostic values and host-bound builders implement it,
- `.into_element(cx)` keeps one obvious meaning,
- no second public bridge trait is needed just to recover method syntax.

### Target companion types

Keep on the component surface:

- `UiBuilder`
- `UiPatchTarget`
- `UiSupportsChrome`
- `UiSupportsLayout`
- layout/style refinement types
- semantics helpers
- `AnyElement` as an explicit raw type

### Target non-exports

The curated component surface should no longer teach or re-export:

- `UiIntoElement`
- `UiHostBoundIntoElement`
- `UiChildIntoElement`
- `UiBuilderHostBoundIntoElementExt`

If some of those names remain internally for a short migration window, they should be treated as
temporary scaffolding rather than stable surface.

Current execution note on 2026-03-12:

- `UiBuilderHostBoundIntoElementExt` is already deleted from the codebase.
- `UiHostBoundIntoElement<H>` is already deleted from the codebase.
- `UiIntoElement` now survives only as `fret_ui_kit::ui_builder::UiIntoElement`, marked
  `#[doc(hidden)]` and treated as internal landing scaffolding rather than public vocabulary.
- `UiChildIntoElement<H>` still survives under `fret_ui_kit::ui` as the child-pipeline bridge,
  but it is no longer root-exported from `fret-ui-kit` and no longer sits behind `fret::UiChild`.

## Advanced/Raw Surface

Keep explicit and legal:

- `AnyElement`
- `Elements`
- raw `ElementContext<'_, H>`
- raw overlay/controller/helper internals

Target rule:

- raw landed-element surfaces remain available,
- but they are not the default authoring story for ordinary apps or reusable component examples.

## Target Helper Signatures

### App-facing helper

```rust
fn footer(cx: &mut UiCx<'_>) -> impl UiChild
```

Transitional equivalent while the child pipeline is still migrating:

```rust
fn page(cx: &mut UiCx<'_>, content: impl UiChild) -> AnyElement
```

### Reusable generic helper

```rust
fn footer<H: UiHost>(cx: &mut ComponentCx<'_, H>) -> impl IntoUiElement<H>
```

### Raw helper

```rust
fn footer_raw<H: UiHost>(cx: &mut ElementContext<'_, H>) -> AnyElement
```

## Target Return-Type Rules

Use these rules in first-party docs/examples:

| Situation | Preferred return type |
| --- | --- |
| View `render(...)` | `Ui` |
| App-facing extracted helper | `impl UiChild` |
| Reusable generic component helper | `impl IntoUiElement<H>` |
| Raw landed helper, low-level overlay internals, diagnostics, explicit IR plumbing | `AnyElement` |

## Delete Targets

These names are delete targets for the curated public product surface:

- `UiIntoElement`
- `UiHostBoundIntoElement`
- `UiChildIntoElement`
- `UiBuilderHostBoundIntoElementExt`

These raw names are **not** delete targets:

- `AnyElement`
- `Elements`

They remain explicit raw tools, not default teaching terms.
