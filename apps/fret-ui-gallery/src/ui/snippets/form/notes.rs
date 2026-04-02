pub const SOURCE: &str = include_str!("notes.rs");

// region: example
use fret::{UiChild, UiCx};
use fret_ui_shadcn::{facade as shadcn, prelude::*};

pub fn render(cx: &mut UiCx<'_>) -> impl UiChild + use<> {
    ui::v_flex(|cx| {
        vec![
            shadcn::raw::typography::muted(
                "Reference baseline: the upstream internal form demo.",
            ).into_element(cx),
            shadcn::raw::typography::muted(
                "API reference: `ecosystem/fret-ui-shadcn/src/form.rs` (`FormControl` is slot-like for a single child), `ecosystem/fret-ui-shadcn/src/field.rs` (`Form`/`FormItem`/helpers), and control primitives: `input.rs`, `textarea.rs`, `checkbox.rs`, `switch.rs`.",
            ).into_element(cx),
            shadcn::raw::typography::muted(
                "The first section mirrors upstream `FormDemo` using `FormState` + `FormRegistry`; the remaining sections are gallery recipes (composition hub).",
            ).into_element(cx),
            shadcn::raw::typography::muted(
                "Fret keeps the shadcn taxonomy (`Form`, `FormField`, `FormItem`, etc.) but maps it onto framework-agnostic field primitives instead of mirroring `react-hook-form` literally; `FormControl` stays a transparent single-control wrapper rather than a layout column.",
            ).into_element(cx),
            shadcn::raw::typography::muted(
                "Field-level required semantics belong on `FormField::required(true)`; keep control-local interaction state such as `disabled` and `read_only` on the concrete control builder.",
            ).into_element(cx),
            shadcn::raw::typography::muted(
                "Invalid decoration also belongs to `FormField`: `FormState` drives the wrapper-level invalid styling and message visibility, while `decorate_control(false)` is the escape hatch for custom multi-control layouts that should keep their own chrome.",
            ).into_element(cx),
            shadcn::raw::typography::muted(
                "There is no standalone upstream `Form` RTL component page/example; Gallery keeps `RTL` as a focused Fret follow-up that validates logical text alignment and explicit horizontal field composition under `DirectionProvider(Rtl)`.",
            ).into_element(cx),
            shadcn::raw::typography::muted(
                "Keep stable test IDs for each recipe so future diag automation can target composition surfaces.",
            ).into_element(cx),
            shadcn::raw::typography::muted(
                "Textarea supports placeholder text; the upstream Bio example uses a placeholder string.",
            ).into_element(cx),
        ]
    })
    .gap(Space::N1)
    .items_start()
    .layout(LayoutRefinement::default().w_full().min_w_0())
    .into_element(cx)
}
// endregion: example
