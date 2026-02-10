use super::*;

#[path = "popover/fixtures.rs"]
mod fixtures;

fn build_shadcn_popover_demo_page(
    cx: &mut ElementContext<'_, App>,
    open: &Model<bool>,
) -> AnyElement {
    use fret_core::Px;
    use fret_ui::Theme;
    use fret_ui_kit::declarative::stack;
    use fret_ui_kit::{ColorRef, LayoutRefinement, Space, ui};
    use fret_ui_shadcn::{Button, ButtonVariant, Popover, PopoverContent};

    Popover::new(open.clone()).into_element(
        cx,
        |cx| {
            Button::new("Open popover")
                .variant(ButtonVariant::Outline)
                .into_element(cx)
        },
        |cx| {
            let theme = Theme::global(&*cx.app).clone();
            let sm_px = theme.metric_required("font.size");
            let sm_line_height = theme.metric_required("font.line_height");
            let muted_fg = theme.color_required("muted.foreground");

            // popover-demo uses `h4.leading-none.font-medium` (line height = 16px).
            let title = ui::text(cx, "Dimensions")
                .text_size_px(sm_px)
                .line_height_px(Px(16.0))
                .font_medium()
                .nowrap()
                .into_element(cx);
            // popover-demo uses `p.text-sm.text-muted-foreground` (line height = 20px).
            let description = ui::text(cx, "Set the dimensions for the layer.")
                .text_size_px(sm_px)
                .line_height_px(sm_line_height)
                .text_color(ColorRef::Color(muted_fg))
                .into_element(cx);
            let header = stack::vstack(
                cx,
                stack::VStackProps::default().gap(Space::N2),
                move |_cx| vec![title, description],
            );

            fn labeled_input_row<H: fret_ui::UiHost>(
                cx: &mut ElementContext<'_, H>,
                label: &str,
                value: &str,
            ) -> AnyElement {
                use fret_core::Px;
                use fret_ui_kit::declarative::stack;
                use fret_ui_kit::{LayoutRefinement, Space};
                use fret_ui_shadcn::{Input, Label};

                let label_el = Label::new(label).into_element(cx);
                let model = cx.app.models_mut().insert(value.to_string());
                let input_el = Input::new(model)
                    .a11y_label(label)
                    .refine_layout(LayoutRefinement::default().h_px(Px(32.0)).flex_grow(1.0))
                    .into_element(cx);

                stack::hstack(
                    cx,
                    stack::HStackProps::default().gap(Space::N4).items_center(),
                    move |_cx| vec![label_el, input_el],
                )
            }

            let rows = vec![
                labeled_input_row(cx, "Width", "100%"),
                labeled_input_row(cx, "Max. width", "300px"),
                labeled_input_row(cx, "Height", "25px"),
                labeled_input_row(cx, "Max. height", "none"),
            ];
            let fields = stack::vstack(
                cx,
                stack::VStackProps::default().gap(Space::N2),
                move |_cx| rows,
            );

            PopoverContent::new([header, fields])
                .refine_layout(LayoutRefinement::default().w_px(Px(320.0)))
                .into_element(cx)
        },
    )
}
