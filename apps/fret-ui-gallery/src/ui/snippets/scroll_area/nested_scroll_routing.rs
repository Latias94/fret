pub const SOURCE: &str = include_str!("nested_scroll_routing.rs");

// region: example
use fret::app::AppActivateExt as _;
use fret::{UiChild, UiCx};
use fret_core::{Point, Px};
use fret_ui::element::SemanticsDecoration;
use fret_ui::scroll::ScrollHandle;
use fret_ui_kit::IntoUiElement;
use fret_ui_shadcn::{facade as shadcn, prelude::*};
fn row<H: UiHost>(cx: &mut ElementContext<'_, H>, i: usize) -> impl IntoUiElement<H> + use<H> {
    let zebra = (i % 2) == 0;
    let theme = cx.theme();
    let bg = if zebra {
        theme.color_token("muted")
    } else {
        theme.color_token("background")
    };

    let props = decl_style::container_props(
        theme,
        ChromeRefinement::default()
            .bg(ColorRef::Color(bg))
            .rounded(Radius::Sm)
            .p(Space::N2),
        LayoutRefinement::default().w_full().h_px(Px(32.0)),
    );
    ui::container_props(props, |_cx| Vec::<AnyElement>::new())
}

pub fn render(cx: &mut UiCx<'_>) -> impl UiChild + use<> {
    cx.named("ui-gallery.scroll_area.nested_scroll_routing", |cx| {
        #[derive(Clone)]
        struct NestedScrollRoutingHandles {
            outer: ScrollHandle,
            inner: ScrollHandle,
        }

        impl Default for NestedScrollRoutingHandles {
            fn default() -> Self {
                Self {
                    outer: ScrollHandle::default(),
                    inner: ScrollHandle::default(),
                }
            }
        }

        let handles = cx.slot_state(NestedScrollRoutingHandles::default, |h| h.clone());
        let outer_handle = handles.outer.clone();
        let inner_handle = handles.inner.clone();

        let reset = {
            let outer_handle = outer_handle.clone();
            let inner_handle = inner_handle.clone();
            shadcn::Button::new("Reset")
                .variant(shadcn::ButtonVariant::Secondary)
                .listen(cx, move |host, action_cx| {
                    outer_handle.scroll_to_offset(Point::new(Px(0.0), Px(0.0)));
                    inner_handle.scroll_to_offset(Point::new(Px(0.0), Px(0.0)));
                    host.request_redraw(action_cx.window);
                })
                .test_id("ui-gallery-scroll-area-nested-reset")
                .into_element(cx)
        };

        let inner_rail = ui::h_row(|cx| {
            (0..24)
                .map(|i| {
                    let card = shadcn::Skeleton::new()
                        .refine_style(ChromeRefinement::default().rounded(Radius::Md))
                        .refine_layout(LayoutRefinement::default().w_px(Px(160.0)).h_px(Px(96.0)))
                        .into_element(cx);
                    let caption =
                        shadcn::raw::typography::muted(format!("Item {i}")).into_element(cx);
                    ui::v_stack(|_cx| vec![card, caption])
                        .gap(Space::N2)
                        .items_start()
                        .layout(LayoutRefinement::default().flex_none())
                        .into_element(cx)
                })
                .collect::<Vec<_>>()
        })
        .gap(Space::N4)
        .items_center()
        .layout(LayoutRefinement::default().w_px(Px(960.0)))
        .into_element(cx);

        let inner = shadcn::scroll_area(cx, |_cx| [inner_rail])
            .axis(fret_ui::element::ScrollAxis::X)
            .scroll_handle(inner_handle)
            .viewport_test_id("ui-gallery-scroll-area-nested-inner-viewport")
            .refine_layout(LayoutRefinement::default().w_full().h_px(Px(140.0)))
            .into_element(cx)
            .attach_semantics(
                SemanticsDecoration::default()
                    .role(fret_core::SemanticsRole::Group)
                    .test_id("ui-gallery-scroll-area-nested-inner"),
            );

        let outer_body = ui::v_flex(|cx| {
            let mut out: Vec<AnyElement> = Vec::new();
            // Keep the nested horizontal rail visible without requiring an outer scroll first;
            // diagnostics scripts target the inner viewport directly.
            for i in 0..2 {
                out.push(row(cx, i).into_element(cx));
            }
            out.push(inner);
            for i in 2..36 {
                out.push(row(cx, i).into_element(cx));
            }
            out
        })
        .layout(LayoutRefinement::default().w_full())
        .gap(Space::N3)
        .into_element(cx);

        let outer = shadcn::scroll_area(cx, |_cx| [outer_body])
            .axis(fret_ui::element::ScrollAxis::Y)
            .scroll_handle(outer_handle)
            .viewport_test_id("ui-gallery-scroll-area-nested-outer-viewport")
            .refine_layout(LayoutRefinement::default().w_full().h_px(Px(240.0)))
            .into_element(cx)
            .attach_semantics(
                SemanticsDecoration::default()
                    .role(fret_core::SemanticsRole::Group)
                    .test_id("ui-gallery-scroll-area-nested-outer"),
            );

        let theme = cx.theme().clone();
        let body = cx
            .container(
                decl_style::container_props(
                    &theme,
                    ChromeRefinement::default()
                        .border_1()
                        .rounded(Radius::Md)
                        .p(Space::N3),
                    LayoutRefinement::default().w_full().max_w(Px(520.0)),
                ),
                move |_cx| [outer],
            )
            .test_id("ui-gallery-scroll-area-nested-body");

        ui::v_flex(|_cx| [reset, body])
            .gap(Space::N2)
            .into_element(cx)
            .attach_semantics(
                SemanticsDecoration::default()
                    .role(fret_core::SemanticsRole::Group)
                    .test_id("ui-gallery-scroll-area-nested-scroll-routing"),
            )
    })
}
// endregion: example
