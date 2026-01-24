use fret_core::{Axis, Color, Corners, Edges, FontId, FontWeight, Px, TextStyle};
use fret_runtime::Model;
use fret_ui::element::{
    AnyElement, ContainerProps, CrossAlign, FlexProps, InsetStyle, LayoutStyle, Length, MainAlign,
    PositionStyle, SizeStyle, TextInputProps,
};
use fret_ui::{ElementContext, TextInputStyle, Theme, UiHost};
use fret_ui_kit::declarative::model_watch::ModelWatchExt as _;
use fret_ui_kit::declarative::style as decl_style;
use fret_ui_kit::recipes::input::{InputTokenKeys, resolve_input_chrome};
use fret_ui_kit::{
    ChromeRefinement, ColorRef, LayoutRefinement, MetricRef, Size as ComponentSize, Space, ui,
};
use std::sync::Arc;

fn otp_gap(theme: &Theme) -> Px {
    theme
        .metric_by_key("component.input_otp.gap")
        .unwrap_or_else(|| MetricRef::space(Space::N2).resolve(theme))
}

fn otp_separator_color(theme: &Theme) -> Color {
    theme.color_required("muted-foreground")
}

fn sanitize_otp(input: &str, length: usize, numeric_only: bool) -> String {
    let mut out = String::new();
    out.reserve(length.min(input.len()));
    for ch in input.chars() {
        if out.chars().count() >= length {
            break;
        }
        if ch.is_whitespace() {
            continue;
        }
        if numeric_only && !ch.is_ascii_digit() {
            continue;
        }
        if ch.is_control() {
            continue;
        }
        out.push(ch);
    }
    out
}

#[derive(Clone)]
pub struct InputOtp {
    model: Model<String>,
    length: usize,
    numeric_only: bool,
    group_size: Option<usize>,
    size: ComponentSize,
    chrome: ChromeRefinement,
    layout: LayoutRefinement,
}

impl std::fmt::Debug for InputOtp {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("InputOtp")
            .field("model", &"<model>")
            .field("length", &self.length)
            .field("numeric_only", &self.numeric_only)
            .field("group_size", &self.group_size)
            .field("size", &self.size)
            .field("chrome", &self.chrome)
            .field("layout", &self.layout)
            .finish()
    }
}

impl InputOtp {
    pub fn new(model: Model<String>) -> Self {
        Self {
            model,
            length: 6,
            numeric_only: true,
            group_size: None,
            size: ComponentSize::default(),
            chrome: ChromeRefinement::default(),
            layout: LayoutRefinement::default(),
        }
    }

    pub fn length(mut self, length: usize) -> Self {
        self.length = length.max(1);
        self
    }

    pub fn numeric_only(mut self, numeric_only: bool) -> Self {
        self.numeric_only = numeric_only;
        self
    }

    pub fn group_size(mut self, group_size: Option<usize>) -> Self {
        self.group_size = group_size.and_then(|v| if v >= 1 { Some(v) } else { None });
        self
    }

    pub fn size(mut self, size: ComponentSize) -> Self {
        self.size = size;
        self
    }

    pub fn refine_style(mut self, style: ChromeRefinement) -> Self {
        self.chrome = self.chrome.merge(style);
        self
    }

    pub fn refine_layout(mut self, layout: LayoutRefinement) -> Self {
        self.layout = self.layout.merge(layout);
        self
    }

    pub fn into_element<H: UiHost>(self, cx: &mut ElementContext<'_, H>) -> AnyElement {
        let theme = Theme::global(&*cx.app).clone();

        let length = self.length;
        let mut value = cx.watch_model(&self.model).cloned().unwrap_or_default();
        let sanitized = sanitize_otp(&value, length, self.numeric_only);
        if sanitized != value {
            let next = sanitized.clone();
            let _ = cx.app.models_mut().update(&self.model, |v| *v = next);
            value = sanitized;
        }

        let chars: Vec<char> = value.chars().collect();
        let resolved =
            resolve_input_chrome(&theme, self.size, &self.chrome, InputTokenKeys::none());

        let slot_w = Px(self.size.input_h(&theme).0.max(0.0));
        let slot_h = Px(resolved.min_height.0.max(0.0));
        let gap = otp_gap(&theme);

        let font_line_height = theme.metric_required("font.line_height");
        let slot_text_style = TextStyle {
            font: FontId::default(),
            size: resolved.text_px,
            weight: FontWeight::MEDIUM,
            slant: Default::default(),
            line_height: Some(font_line_height),
            letter_spacing_em: None,
        };

        let root_layout = decl_style::layout_style(&theme, self.layout.relative());
        let separator_color = otp_separator_color(&theme);

        cx.container(
            ContainerProps {
                layout: root_layout,
                ..Default::default()
            },
            |cx| {
                let mut out: Vec<AnyElement> = Vec::new();

                let mut slots: Vec<AnyElement> = Vec::new();
                for idx in 0..length {
                    if idx > 0 && self.group_size.is_some_and(|g| g >= 1 && idx % g == 0) {
                        let sep_layout = LayoutStyle {
                            size: SizeStyle {
                                width: Length::Px(Px(12.0)),
                                height: Length::Px(slot_h),
                                ..Default::default()
                            },
                            ..Default::default()
                        };
                        let slot_text_style_for_sep = slot_text_style.clone();
                        slots.push(cx.flex(
                            FlexProps {
                                layout: sep_layout,
                                direction: Axis::Horizontal,
                                gap: Px(0.0),
                                padding: Edges::all(Px(0.0)),
                                justify: MainAlign::Center,
                                align: CrossAlign::Center,
                                wrap: false,
                            },
                            move |cx| {
                                let style = slot_text_style_for_sep.clone();
                                let mut label = ui::label(cx, Arc::from("•"))
                                    .text_size_px(style.size)
                                    .font_weight(style.weight)
                                    .text_color(ColorRef::Color(separator_color))
                                    .nowrap();
                                if let Some(line_height) = style.line_height {
                                    label = label.line_height_px(line_height);
                                }
                                if let Some(letter_spacing_em) = style.letter_spacing_em {
                                    label = label.letter_spacing_em(letter_spacing_em);
                                }
                                vec![label.into_element(cx)]
                            },
                        ));
                    }

                    let ch = chars.get(idx).copied();
                    let text: Arc<str> = ch
                        .map(|c| Arc::from(c.to_string()))
                        .unwrap_or_else(|| Arc::from(""));

                    let slot_layout = LayoutStyle {
                        size: SizeStyle {
                            width: Length::Px(slot_w),
                            height: Length::Px(slot_h),
                            ..Default::default()
                        },
                        ..Default::default()
                    };

                    let bg = resolved.background;
                    let border = resolved.border_width;
                    let border_color = resolved.border_color;
                    let radius = resolved.radius;
                    let fg = resolved.text_color;

                    let slot_text_style_for_slot = slot_text_style.clone();
                    slots.push(cx.container(
                        ContainerProps {
                            layout: slot_layout,
                            padding: Edges::all(Px(0.0)),
                            background: Some(bg),
                            shadow: None,
                            border: Edges::all(border),
                            border_color: Some(border_color),
                            corner_radii: Corners::all(radius),
                            ..Default::default()
                        },
                        move |cx| {
                            vec![cx.flex(
                                FlexProps {
                                    layout: LayoutStyle::default(),
                                    direction: Axis::Horizontal,
                                    gap: Px(0.0),
                                    padding: Edges::all(Px(0.0)),
                                    justify: MainAlign::Center,
                                    align: CrossAlign::Center,
                                    wrap: false,
                                },
                                move |cx| {
                                    let style = slot_text_style_for_slot.clone();
                                    let mut label = ui::label(cx, text)
                                        .text_size_px(style.size)
                                        .font_weight(style.weight)
                                        .text_color(ColorRef::Color(fg))
                                        .nowrap();
                                    if let Some(line_height) = style.line_height {
                                        label = label.line_height_px(line_height);
                                    }
                                    if let Some(letter_spacing_em) = style.letter_spacing_em {
                                        label = label.letter_spacing_em(letter_spacing_em);
                                    }
                                    vec![label.into_element(cx)]
                                },
                            )]
                        },
                    ));
                }

                out.push(cx.flex(
                    FlexProps {
                        layout: LayoutStyle::default(),
                        direction: Axis::Horizontal,
                        gap,
                        padding: Edges::all(Px(0.0)),
                        justify: MainAlign::Start,
                        align: CrossAlign::Center,
                        wrap: false,
                    },
                    move |_cx| slots,
                ));

                let mut chrome = TextInputStyle::from_theme(theme.snapshot());
                chrome.padding = Edges::all(Px(0.0));
                chrome.corner_radii = Corners::all(Px(0.0));
                chrome.border = Edges::all(Px(0.0));
                chrome.background = Color::TRANSPARENT;
                chrome.border_color = Color::TRANSPARENT;
                chrome.border_color_focused = Color::TRANSPARENT;
                chrome.text_color = Color::TRANSPARENT;
                chrome.selection_color = Color::TRANSPARENT;
                chrome.caret_color = Color::TRANSPARENT;
                chrome.preedit_color = Color::TRANSPARENT;

                let mut input = TextInputProps::new(self.model);
                input.chrome = chrome;
                input.text_style = slot_text_style;
                input.layout = LayoutStyle {
                    position: PositionStyle::Absolute,
                    inset: InsetStyle {
                        left: Some(Px(0.0)),
                        top: Some(Px(0.0)),
                        right: Some(Px(0.0)),
                        bottom: Some(Px(0.0)),
                    },
                    size: SizeStyle {
                        width: Length::Fill,
                        height: Length::Fill,
                        ..Default::default()
                    },
                    overflow: fret_ui::element::Overflow::Clip,
                    ..Default::default()
                };

                out.push(cx.text_input(input));
                out
            },
        )
    }
}

pub fn input_otp<H: UiHost>(cx: &mut ElementContext<'_, H>, otp: InputOtp) -> AnyElement {
    otp.into_element(cx)
}

#[cfg(test)]
mod tests {
    use super::*;
    use fret_app::App;
    use fret_core::{
        AppWindowId, Point, Rect, TextBlobId, TextConstraints, TextMetrics, TextService,
    };
    use fret_core::{PathCommand, SvgId, SvgService};
    use fret_core::{PathConstraints, PathId, PathMetrics, PathService, PathStyle};
    use fret_ui::tree::UiTree;

    #[derive(Default)]
    struct FakeServices;

    impl TextService for FakeServices {
        fn prepare(
            &mut self,
            _input: &fret_core::TextInput,
            _constraints: TextConstraints,
        ) -> (TextBlobId, TextMetrics) {
            (
                TextBlobId::default(),
                TextMetrics {
                    size: fret_core::Size::new(Px(0.0), Px(0.0)),
                    baseline: Px(0.0),
                },
            )
        }

        fn release(&mut self, _blob: TextBlobId) {}
    }

    impl PathService for FakeServices {
        fn prepare(
            &mut self,
            _commands: &[PathCommand],
            _style: PathStyle,
            _constraints: PathConstraints,
        ) -> (PathId, PathMetrics) {
            (PathId::default(), PathMetrics::default())
        }

        fn release(&mut self, _path: PathId) {}
    }

    impl SvgService for FakeServices {
        fn register_svg(&mut self, _bytes: &[u8]) -> SvgId {
            SvgId::default()
        }

        fn unregister_svg(&mut self, _svg: SvgId) -> bool {
            true
        }
    }

    fn render(
        ui: &mut UiTree<App>,
        app: &mut App,
        services: &mut dyn fret_core::UiServices,
        model: Model<String>,
    ) {
        let window = AppWindowId::default();
        ui.set_window(window);

        let bounds = Rect::new(
            Point::new(Px(0.0), Px(0.0)),
            fret_core::Size::new(Px(480.0), Px(120.0)),
        );

        let root =
            fret_ui::declarative::render_root(ui, app, services, window, bounds, "otp", |cx| {
                vec![InputOtp::new(model).length(6).into_element(cx)]
            });
        ui.set_root(root);
        ui.layout_all(app, services, bounds, 1.0);
    }

    #[test]
    fn input_otp_sanitizes_and_clamps_value() {
        let mut app = App::new();
        let mut ui: UiTree<App> = UiTree::new();
        let mut services = FakeServices::default();

        let model = app.models_mut().insert("12a 34-5678".to_string());
        render(&mut ui, &mut app, &mut services, model.clone());

        assert_eq!(app.models().get_cloned(&model).as_deref(), Some("123456"));
    }
}
