use fret_components_icons::IconId;
use fret_core::{Axis, Edges, FontId, Px, TextStyle};
use fret_runtime::{CommandId, Model};
use fret_ui::element::{
    ContainerProps, CrossAlign, FlexProps, LayoutStyle, Length, MainAlign, PositionStyle,
    SizeStyle, TextInputProps,
};
use fret_ui::widget::Invalidation;
use fret_ui::{ElementCx, Theme, UiHost};

use crate::Size;
use crate::declarative::icon;
use crate::recipes::input::{InputTokenKeys, resolve_input_chrome};
use crate::style::StyleRefinement;

#[track_caller]
pub fn text_field_with_leading_icon_and_clear<H: UiHost>(
    cx: &mut ElementCx<'_, H>,
    model: Model<String>,
    size: Size,
    leading_icon: IconId,
    clear_command: CommandId,
) -> fret_ui::element::AnyElement {
    cx.scope(|cx| {
        cx.observe_model(model, Invalidation::Layout);
        let has_value = cx.app.models().get(model).is_some_and(|s| !s.is_empty());

        let theme = Theme::global(&*cx.app);
        let resolved = resolve_input_chrome(
            theme,
            size,
            &StyleRefinement::default(),
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

        let slot_w = Px(size.input_h(theme).0.max(0.0));
        let left_pad = Px((resolved.padding_x.0 + slot_w.0).max(0.0));
        let right_pad = Px((resolved.padding_x.0 + slot_w.0).max(0.0));

        let mut chrome = fret_ui::primitives::TextInputStyle::from_theme(theme.snapshot());
        chrome.padding = Edges {
            top: resolved.padding_y,
            right: right_pad,
            bottom: resolved.padding_y,
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
            .unwrap_or(theme.metrics.font_line_height);
        let text_style = TextStyle {
            font: FontId::default(),
            size: resolved.text_px,
            line_height: Some(font_line_height),
            ..Default::default()
        };

        let mut input = TextInputProps::new(model);
        input.chrome = chrome;
        input.text_style = text_style;
        input.layout.size = SizeStyle {
            width: Length::Fill,
            min_height: Some(resolved.min_height),
            ..Default::default()
        };

        let mut root_layout = LayoutStyle::default();
        root_layout.size.width = Length::Fill;
        root_layout.position = PositionStyle::Relative;

        cx.container(
            ContainerProps {
                layout: root_layout,
                ..Default::default()
            },
            |cx| {
                let mut out = Vec::new();
                out.push(cx.text_input(input));

                let mut left_layout = LayoutStyle::default();
                left_layout.position = PositionStyle::Absolute;
                left_layout.inset.left = Some(Px(0.0));
                left_layout.inset.top = Some(Px(0.0));
                left_layout.inset.bottom = Some(Px(0.0));
                left_layout.size = SizeStyle {
                    width: Length::Px(slot_w),
                    height: Length::Fill,
                    ..Default::default()
                };
                out.push(cx.flex(
                    FlexProps {
                        layout: left_layout,
                        direction: Axis::Horizontal,
                        gap: Px(0.0),
                        padding_x: resolved.padding_x,
                        padding_y: Px(0.0),
                        justify: MainAlign::Center,
                        align: CrossAlign::Center,
                        wrap: false,
                    },
                    |cx| vec![icon::icon(cx, leading_icon)],
                ));

                if has_value {
                    let mut right_layout = LayoutStyle::default();
                    right_layout.position = PositionStyle::Absolute;
                    right_layout.inset.right = Some(Px(0.0));
                    right_layout.inset.top = Some(Px(0.0));
                    right_layout.inset.bottom = Some(Px(0.0));
                    right_layout.size = SizeStyle {
                        width: Length::Px(slot_w),
                        height: Length::Fill,
                        ..Default::default()
                    };

                    out.push(cx.pressable(
                        fret_ui::element::PressableProps {
                            layout: right_layout,
                            on_click: Some(clear_command),
                            ..Default::default()
                        },
                        |cx, _st| {
                            vec![cx.flex(
                                FlexProps {
                                    layout: LayoutStyle {
                                        size: SizeStyle {
                                            width: Length::Fill,
                                            height: Length::Fill,
                                            ..Default::default()
                                        },
                                        ..Default::default()
                                    },
                                    direction: Axis::Horizontal,
                                    gap: Px(0.0),
                                    padding_x: resolved.padding_x,
                                    padding_y: Px(0.0),
                                    justify: MainAlign::Center,
                                    align: CrossAlign::Center,
                                    wrap: false,
                                },
                                |cx| vec![icon::icon(cx, IconId::new("x"))],
                            )]
                        },
                    ));
                }

                out
            },
        )
    })
}
