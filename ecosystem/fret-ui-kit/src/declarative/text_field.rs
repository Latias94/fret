#![cfg(feature = "recipes")]

use fret_core::{Axis, Edges, FontId, Px, TextStyle};
use fret_icons::{IconId, ids};
use fret_runtime::{CommandId, Model};
use fret_ui::Invalidation;
use fret_ui::element::{ContainerProps, FlexProps, Length, SizeStyle, TextInputProps};
use fret_ui::{ElementContext, Theme, UiHost};

use crate::declarative::action_hooks::ActionHooksExt;
use crate::declarative::icon;
use crate::declarative::style as decl_style;
use crate::recipes::input::{InputTokenKeys, resolve_input_chrome};
use crate::style::ChromeRefinement;
use crate::{Items, Justify, LayoutRefinement, Size, Space};

#[track_caller]
pub fn text_field_with_leading_icon_and_clear<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    model: Model<String>,
    size: Size,
    leading_icon: IconId,
    clear_command: CommandId,
    cancel_command: Option<CommandId>,
) -> fret_ui::element::AnyElement {
    cx.scope(|cx| {
        let has_value = cx
            .read_model_ref(&model, Invalidation::Layout, |s| !s.is_empty())
            .unwrap_or(false);

        let theme = Theme::global(&*cx.app).clone();
        let resolved = resolve_input_chrome(
            &theme,
            size,
            &ChromeRefinement::default(),
            InputTokenKeys {
                padding_x: Some("component.text_field.padding_x"),
                padding_y: Some("component.text_field.padding_y"),
                min_height: Some("component.text_field.min_height"),
                radius: Some("component.text_field.radius"),
                border_width: Some("component.text_field.border_width"),
                bg: Some("component.text_field.bg"),
                border: Some("component.text_field.border"),
                border_focus: Some("component.text_field.border_focus"),
                fg: Some("component.text_field.fg"),
                text_px: Some("component.text_field.text_px"),
                selection: Some("component.text_field.selection"),
            },
        );

        let slot_w = Px(size.input_h(&theme).0.max(0.0));
        let base_px = resolved.padding.left;
        let base_py = resolved.padding.top;
        let left_pad = Px((base_px.0 + slot_w.0).max(0.0));
        let right_pad = Px((base_px.0 + slot_w.0).max(0.0));

        let mut chrome = fret_ui::TextInputStyle::from_theme(theme.snapshot());
        chrome.padding = Edges {
            top: base_py,
            right: right_pad,
            bottom: base_py,
            left: left_pad,
        };
        chrome.corner_radii = fret_core::Corners::all(resolved.radius);
        chrome.border = Edges::all(resolved.border_width);
        chrome.background = resolved.background;
        chrome.border_color = resolved.border_color;
        chrome.border_color_focused = resolved.border_color_focused;
        chrome.text_color = resolved.text_color;
        chrome.caret_color = resolved.text_color;
        chrome.selection_color = resolved.selection_color;

        let font_line_height = theme
            .metric_by_key("font.line_height")
            .unwrap_or_else(|| theme.metric_required("font.line_height"));
        let text_style = TextStyle {
            font: FontId::default(),
            size: resolved.text_px,
            line_height: Some(font_line_height),
            ..Default::default()
        };

        let mut input = TextInputProps::new(model);
        input.chrome = chrome;
        input.text_style = text_style;
        input.cancel_command = cancel_command;
        input.layout.size = SizeStyle {
            width: Length::Fill,
            min_height: Some(resolved.min_height),
            ..Default::default()
        };

        let root_layout =
            decl_style::layout_style(&theme, LayoutRefinement::default().relative().w_full());

        cx.container(
            ContainerProps {
                layout: root_layout,
                ..Default::default()
            },
            |cx| {
                let mut out = Vec::new();
                out.push(cx.text_input(input));

                let left_layout = decl_style::layout_style(
                    &theme,
                    LayoutRefinement::default()
                        .absolute()
                        .left(Space::N0)
                        .top(Space::N0)
                        .bottom(Space::N0)
                        .w_px(slot_w)
                        .h_full(),
                );
                out.push(cx.flex(
                    FlexProps {
                        layout: left_layout,
                        direction: Axis::Horizontal,
                        gap: Px(0.0),
                        padding: Edges::symmetric(base_px, Px(0.0)),
                        justify: Justify::Center.to_main_align(),
                        align: Items::Center.to_cross_align(),
                        wrap: false,
                    },
                    |cx| vec![icon::icon(cx, leading_icon)],
                ));

                if has_value {
                    let right_layout = decl_style::layout_style(
                        &theme,
                        LayoutRefinement::default()
                            .absolute()
                            .right(Space::N0)
                            .top(Space::N0)
                            .bottom(Space::N0)
                            .w_px(slot_w)
                            .h_full(),
                    );

                    out.push(cx.pressable(
                        fret_ui::element::PressableProps {
                            layout: right_layout,
                            ..Default::default()
                        },
                        |cx, _st| {
                            cx.pressable_dispatch_command_if_enabled(clear_command.clone());
                            vec![cx.flex(
                                FlexProps {
                                    layout: decl_style::layout_style(
                                        &theme,
                                        LayoutRefinement::default().size_full(),
                                    ),
                                    direction: Axis::Horizontal,
                                    gap: Px(0.0),
                                    padding: Edges::symmetric(base_px, Px(0.0)),
                                    justify: Justify::Center.to_main_align(),
                                    align: Items::Center.to_cross_align(),
                                    wrap: false,
                                },
                                |cx| vec![icon::icon(cx, ids::ui::CLOSE)],
                            )]
                        },
                    ));
                }

                out
            },
        )
    })
}
