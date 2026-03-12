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

Advanced/manual-assembly note on 2026-03-12:

- examples that intentionally use `fret::advanced::prelude::*` do not currently teach `UiChild`
  as their primary helper vocabulary.
- in that lane, the preferred non-raw helper signature is:

```rust
fn helper(cx: &mut UiCx<'_>) -> impl fret_ui_kit::IntoUiElement<KernelApp> + use<>
```

- do not default those helpers to `AnyElement` unless the function is actually raw composition
  glue (overlay/effect/manual-assembly internals).
- current first-party examples on that lane now include `custom_effect_v1_demo.rs::{plain_lens,
  custom_effect_lens}` and `custom_effect_v2_demo.rs::{plain_lens, custom_effect_lens}`; the raw
  `lens_shell(..., body: AnyElement)` seam remains explicit because it owns effect-layer/body
  assembly.
- Rust 2024 precise captures note: if the helper wants `+ use<...>` and also accepts a flexible
  label/input argument, prefer a named generic such as `fn helper<L>(..., label: L) -> impl
  IntoUiElement<KernelApp> + use<L> where L: Into<Arc<str>>` rather than argument-position
  `impl Into<Arc<str>>`, because precise captures currently require all type parameters to appear
  in the `use<...>` list.
- semantics ergonomics follow-up: explicit `.into_element(cx).attach_semantics(...)` is an
  acceptable local workaround, but if that pattern becomes common the framework should add a
  unified decorator helper on top of public `IntoUiElement<H>` rather than multiplying builder-
  specific semantics methods across ecosystem components.

Default-app reusable helper note on 2026-03-12:

- if a helper lives on the default app-facing surface (`use fret::UiCx;`) but is still authored as
  a reusable typed landing helper rather than `impl UiChild`, prefer:

```rust
fn helper(cx: &mut UiCx<'_>) -> impl fret_ui_kit::IntoUiElement<fret_app::App> + use<>
```

- do not teach `KernelApp` on that lane: it is not a public type on the top-level `fret` facade,
  so Gallery/default-app snippets should spell `fret_app::App` when they need the concrete host
  type.

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
- `UiChildIntoElement<H>` is now deleted from the codebase; heterogeneous child collection in
  `fret_ui_kit::ui` / `imui` lands directly through `IntoUiElement<H>`.

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

Wrapper/composer helper rule:

```rust
fn centered<H: UiHost, B>(body: B) -> impl IntoUiElement<H> + use<H, B>
where
    B: IntoUiElement<H>
```

If a helper only wraps/layouts child content, prefer accepting `IntoUiElement<H>` instead of
forcing callers to pre-land `AnyElement`.

First-party UI Gallery examples already using this rule now include:

- `src/ui/snippets/ai/{context_default,context_demo}.rs::centered(...)`
- `src/ui/snippets/ai/{file_tree_basic,file_tree_expanded}.rs::preview(...)`
- `src/ui/snippets/ai/test_results_demo.rs::progress_section(...)`
- `src/ui/snippets/avatar/{with_badge,fallback_only,sizes,dropdown}.rs::wrap_row(...)`
- `src/ui/snippets/avatar/{demo,group,sizes,group_count}.rs::avatar_with_image(...)`
- `src/ui/snippets/avatar/{demo,with_badge}.rs::avatar_with_badge(...)`
- `src/ui/snippets/avatar/{group,group_count}.rs::{group(...), group_with_count(...)}`
- `src/ui/snippets/avatar/{badge_icon,group_count_icon}.rs::icon(...)`
- `src/ui/snippets/button/{demo,size,with_icon,link_render,rtl,loading,variants,button_group,rounded}.rs::wrap_row(...)`
- `src/ui/snippets/button/size.rs::row(...)`
- `src/ui/snippets/breadcrumb/dropdown.rs::dot_separator(...)`
- `src/ui/snippets/item/extras_rtl.rs::{outline_button_sm(...), item_basic(...)}`

Those examples keep explicit `.into_element(cx)` seams only where the surrounding API still
intentionally consumes raw landed children, such as `DocSection::new(...)`, child arrays, and
`ItemActions::new(...)`.

Implementation fallback rule:

- if an ecosystem builder or recipe type does not yet implement `IntoUiElement<H>` directly,
  it is still acceptable for the helper to land internally with `.into_element(cx)` and expose the
  result as `impl IntoUiElement<H>`.
- keep that fallback inside the helper so callers still see the unified conversion contract rather
  than a raw `AnyElement` signature.

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
