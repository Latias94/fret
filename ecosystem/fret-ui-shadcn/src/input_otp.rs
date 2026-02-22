use fret_core::{Axis, Color, Corners, Edges, FontId, FontWeight, Px};
use fret_icons::ids;
use fret_runtime::Model;
use fret_ui::element::{
    AnyElement, ContainerProps, CrossAlign, FlexProps, InsetStyle, LayoutStyle, Length, MainAlign,
    PositionStyle, SemanticsDecoration, ShadowLayerStyle, ShadowStyle, SizeStyle, TextInputProps,
};
use fret_ui::{ElementContext, TextInputStyle, Theme, UiHost};
use fret_ui_kit::declarative::icon as decl_icon;
use fret_ui_kit::declarative::model_watch::ModelWatchExt as _;
use fret_ui_kit::declarative::scheduling;
use fret_ui_kit::declarative::style as decl_style;
use fret_ui_kit::recipes::input::{InputTokenKeys, resolve_input_chrome};
use fret_ui_kit::typography;
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
    theme.color_token("muted-foreground")
}

fn otp_active_ring_width(theme: &Theme) -> Px {
    theme
        .metric_by_key("component.ring.width")
        .unwrap_or(Px(3.0))
}

fn otp_active_ring_color(theme: &Theme) -> Color {
    theme
        .color_by_key("ring/50")
        .or_else(|| theme.color_by_key("ring"))
        .unwrap_or_else(|| theme.color_token("ring"))
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

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum InputOtpSlotCornerMode {
    #[default]
    Merge,
    All,
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
    test_id_prefix: Option<Arc<str>>,
    container_gap_override: Option<Px>,
    slot_gap_override: Option<Px>,
    slot_size_override: Option<(Px, Px)>,
    slot_text_px_override: Option<Px>,
    slot_line_height_px_override: Option<Px>,
    slot_corner_mode: InputOtpSlotCornerMode,
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
            .field(
                "test_id_prefix",
                &self.test_id_prefix.as_ref().map(|s| s.as_ref()),
            )
            .field("container_gap_override", &self.container_gap_override)
            .field("slot_gap_override", &self.slot_gap_override)
            .field("slot_size_override", &self.slot_size_override)
            .field("slot_text_px_override", &self.slot_text_px_override)
            .field(
                "slot_line_height_px_override",
                &self.slot_line_height_px_override,
            )
            .field("slot_corner_mode", &self.slot_corner_mode)
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
            test_id_prefix: None,
            container_gap_override: None,
            slot_gap_override: None,
            slot_size_override: None,
            slot_text_px_override: None,
            slot_line_height_px_override: None,
            slot_corner_mode: InputOtpSlotCornerMode::default(),
        }
    }

    pub fn test_id_prefix(mut self, prefix: impl Into<Arc<str>>) -> Self {
        self.test_id_prefix = Some(prefix.into());
        self
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

    pub fn container_gap_px(mut self, gap: Px) -> Self {
        self.container_gap_override = Some(gap);
        self
    }

    pub fn slot_gap_px(mut self, gap: Px) -> Self {
        self.slot_gap_override = Some(gap);
        self
    }

    pub fn slot_size_px(mut self, w: Px, h: Px) -> Self {
        self.slot_size_override = Some((w, h));
        self
    }

    pub fn slot_text_px(mut self, px: Px) -> Self {
        self.slot_text_px_override = Some(px);
        self
    }

    pub fn slot_line_height_px(mut self, px: Px) -> Self {
        self.slot_line_height_px_override = Some(px);
        self
    }

    pub fn slot_corner_mode(mut self, mode: InputOtpSlotCornerMode) -> Self {
        self.slot_corner_mode = mode;
        self
    }

    #[track_caller]
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

        let default_slot_w = Px(resolved.min_height.0.max(0.0));
        let default_slot_h = Px(resolved.min_height.0.max(0.0));
        let (slot_w, slot_h) = self
            .slot_size_override
            .unwrap_or((default_slot_w, default_slot_h));

        let container_gap = self
            .container_gap_override
            .unwrap_or_else(|| otp_gap(&theme));
        let slot_gap = self.slot_gap_override.unwrap_or(Px(0.0));
        let slot_gap_is_nonzero = slot_gap.0.abs() >= 0.5;

        let font_line_height = theme.metric_token("font.line_height");
        let slot_line_height = self
            .slot_line_height_px_override
            .unwrap_or(font_line_height);
        let slot_text_px = self.slot_text_px_override.unwrap_or(resolved.text_px);
        let mut slot_text_style =
            typography::fixed_line_box_style(FontId::ui(), slot_text_px, slot_line_height);
        slot_text_style.weight = FontWeight::MEDIUM;

        let root_layout = decl_style::layout_style(&theme, self.layout.relative());
        let separator_color = otp_separator_color(&theme);

        cx.container(
            ContainerProps {
                layout: root_layout,
                ..Default::default()
            },
            |cx| {
                let mut out: Vec<AnyElement> = Vec::new();

                let input_test_id: Option<Arc<str>> = self
                    .test_id_prefix
                    .as_ref()
                    .map(|prefix| Arc::from(format!("{prefix}.input")));

                let group_size = self.group_size.unwrap_or(length).max(1);
                let mut groups: Vec<(usize, usize)> = Vec::new();
                let mut start = 0;
                while start < length {
                    let end = (start + group_size).min(length);
                    groups.push((start, end));
                    start = end;
                }

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
                chrome.preedit_underline_color = Color::TRANSPARENT;

                let mut input = TextInputProps::new(self.model);
                input.chrome = chrome;
                input.text_style = slot_text_style.clone();
                input.test_id = input_test_id.clone();
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
                let input_el = cx.text_input(input);

                let input_id = input_el.id;
                let focused = cx.is_focused_element(input_id);
                let active_slot_idx = focused.then(|| chars.len().min(length.saturating_sub(1)));
                let fake_caret_idx = (focused && chars.len() < length).then_some(chars.len());

                scheduling::set_continuous_frames(cx, focused && fake_caret_idx.is_some());

                let caret_blink_visible = if focused && fake_caret_idx.is_some() {
                    // Approximate shadcn's `animate-caret-blink duration-1000`.
                    // Full cycle ≈ 60 frames; toggles every ≈ 30 frames.
                    (cx.frame_id.0 / 30) % 2 == 0
                } else {
                    false
                };

                let mut slot_ids: Vec<fret_ui::elements::GlobalElementId> =
                    Vec::with_capacity(length);
                let mut slot_corners: Vec<Corners> = Vec::with_capacity(length);
                let mut pieces: Vec<AnyElement> = Vec::new();
                for (group_idx, (start, end)) in groups.iter().copied().enumerate() {
                    let mut slots: Vec<AnyElement> = Vec::new();
                    for idx in start..end {
                        let is_first = idx == start;
                        let is_last = idx + 1 == end;
                        let is_active = active_slot_idx == Some(idx);
                        let has_fake_caret = fake_caret_idx == Some(idx);

                        let ch = chars.get(idx).copied();
                        let text: Arc<str> = ch
                            .map(|c| Arc::from(c.to_string()))
                            .unwrap_or_else(|| Arc::from(""));

                        let bg = resolved.background;
                        let border = resolved.border_width;
                        let border_color = if is_active {
                            resolved.border_color_focused
                        } else {
                            resolved.border_color
                        };
                        let radius = resolved.radius;
                        let fg = resolved.text_color;

                        let all_corners = slot_gap_is_nonzero
                            || matches!(self.slot_corner_mode, InputOtpSlotCornerMode::All);
                        let corner_radii = if all_corners {
                            Corners::all(radius)
                        } else {
                            Corners {
                                top_left: if is_first { radius } else { Px(0.0) },
                                bottom_left: if is_first { radius } else { Px(0.0) },
                                top_right: if is_last { radius } else { Px(0.0) },
                                bottom_right: if is_last { radius } else { Px(0.0) },
                            }
                        };

                        let slot_border = if slot_gap_is_nonzero {
                            Edges::all(border)
                        } else {
                            Edges {
                                top: border,
                                right: border,
                                bottom: border,
                                left: if is_first { border } else { Px(0.0) },
                            }
                        };

                        let slot_text_style_for_slot = slot_text_style.clone();
                        let slot_test_id: Option<Arc<str>> = self
                            .test_id_prefix
                            .as_ref()
                            .map(|prefix| Arc::from(format!("{prefix}.slot.{idx}")));

                        let mut slot_el = cx.container(
                            ContainerProps {
                                layout: LayoutStyle {
                                    size: SizeStyle {
                                        width: Length::Px(slot_w),
                                        height: Length::Px(slot_h),
                                        ..Default::default()
                                    },
                                    ..Default::default()
                                },
                                padding: Edges::all(Px(0.0)),
                                background: Some(bg),
                                shadow: Some(decl_style::shadow_xs(&theme, radius)),
                                border: slot_border,
                                border_color: Some(border_color),
                                corner_radii,
                                ..Default::default()
                            },
                            move |cx| {
                                let mut inner: Vec<AnyElement> = Vec::new();

                                inner.push(cx.flex(
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
                                            label = label
                                                .line_height_px(line_height)
                                                .line_height_policy(
                                                    fret_core::TextLineHeightPolicy::FixedFromStyle,
                                                );
                                        }
                                        if let Some(letter_spacing_em) = style.letter_spacing_em {
                                            label = label.letter_spacing_em(letter_spacing_em);
                                        }
                                        vec![label.into_element(cx)]
                                    },
                                ));

                                if has_fake_caret && caret_blink_visible {
                                    let caret_color = fg;
                                    let caret_h = Px(16.0).min(Px((slot_h.0 - 8.0).max(0.0)));
                                    inner.push(cx.container(
                                        ContainerProps {
                                            layout: LayoutStyle {
                                                position: PositionStyle::Absolute,
                                                inset: InsetStyle {
                                                    left: Some(Px(0.0)),
                                                    top: Some(Px(0.0)),
                                                    right: Some(Px(0.0)),
                                                    bottom: Some(Px(0.0)),
                                                },
                                                ..Default::default()
                                            },
                                            ..Default::default()
                                        },
                                        move |cx| {
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
                                                    padding: Edges::all(Px(0.0)),
                                                    justify: MainAlign::Center,
                                                    align: CrossAlign::Center,
                                                    wrap: false,
                                                },
                                                move |cx| {
                                                    vec![cx.container(
                                                        ContainerProps {
                                                            layout: LayoutStyle {
                                                                size: SizeStyle {
                                                                    width: Length::Px(Px(1.0)),
                                                                    height: Length::Px(caret_h),
                                                                    ..Default::default()
                                                                },
                                                                ..Default::default()
                                                            },
                                                            background: Some(caret_color),
                                                            ..Default::default()
                                                        },
                                                        |_cx| Vec::<AnyElement>::new(),
                                                    )]
                                                },
                                            )]
                                        },
                                    ));
                                }

                                inner
                            },
                        );

                        slot_ids.push(slot_el.id);
                        slot_corners.push(corner_radii);

                        let mut decoration = SemanticsDecoration::default().selected(is_active);
                        if let Some(test_id) = slot_test_id {
                            decoration = decoration.test_id(test_id);
                        }
                        slot_el = slot_el.attach_semantics(decoration);

                        slots.push(slot_el);
                    }

                    pieces.push(cx.flex(
                        FlexProps {
                            layout: LayoutStyle::default(),
                            direction: Axis::Horizontal,
                            gap: slot_gap,
                            padding: Edges::all(Px(0.0)),
                            justify: MainAlign::Start,
                            align: CrossAlign::Center,
                            wrap: false,
                        },
                        move |_cx| slots,
                    ));

                    if group_idx + 1 < groups.len() {
                        pieces.push(cx.flex(
                            FlexProps {
                                layout: LayoutStyle {
                                    size: SizeStyle {
                                        width: Length::Px(Px(24.0)),
                                        height: Length::Px(Px(24.0)),
                                        ..Default::default()
                                    },
                                    ..Default::default()
                                },
                                direction: Axis::Horizontal,
                                gap: Px(0.0),
                                padding: Edges::all(Px(0.0)),
                                justify: MainAlign::Center,
                                align: CrossAlign::Center,
                                wrap: false,
                            },
                            move |cx| {
                                vec![decl_icon::icon_with(
                                    cx,
                                    ids::ui::MINUS,
                                    Some(Px(24.0)),
                                    Some(ColorRef::Color(separator_color)),
                                )]
                            },
                        ));
                    }
                }

                out.push(cx.flex(
                    FlexProps {
                        layout: LayoutStyle::default(),
                        direction: Axis::Horizontal,
                        gap: container_gap,
                        padding: Edges::all(Px(0.0)),
                        justify: MainAlign::Start,
                        align: CrossAlign::Center,
                        wrap: false,
                    },
                    move |_cx| pieces,
                ));

                out.push(input_el);

                if focused {
                    if let Some(active_idx) = active_slot_idx {
                        let root_bounds = cx.last_bounds_for_element(cx.root_id());
                        let slot_bounds = slot_ids
                            .get(active_idx)
                            .and_then(|id| cx.last_bounds_for_element(*id));
                        if let (Some(root_bounds), Some(slot_bounds)) = (root_bounds, slot_bounds) {
                            let ring_w = otp_active_ring_width(&theme);
                            let ring_color = otp_active_ring_color(&theme);
                            let corners = slot_corners
                                .get(active_idx)
                                .copied()
                                .unwrap_or_else(|| Corners::all(resolved.radius));
                            let left =
                                Px((slot_bounds.origin.x.0 - root_bounds.origin.x.0).max(0.0));
                            let top =
                                Px((slot_bounds.origin.y.0 - root_bounds.origin.y.0).max(0.0));

                            out.push(cx.hit_test_gate(false, move |cx| {
                                vec![cx.container(
                                    ContainerProps {
                                        layout: LayoutStyle {
                                            position: PositionStyle::Absolute,
                                            inset: InsetStyle {
                                                left: Some(left),
                                                top: Some(top),
                                                right: None,
                                                bottom: None,
                                            },
                                            size: SizeStyle {
                                                width: Length::Px(slot_bounds.size.width),
                                                height: Length::Px(slot_bounds.size.height),
                                                ..Default::default()
                                            },
                                            ..Default::default()
                                        },
                                        shadow: Some(ShadowStyle {
                                            primary: ShadowLayerStyle {
                                                color: ring_color,
                                                offset_x: Px(0.0),
                                                offset_y: Px(0.0),
                                                blur: Px(0.0),
                                                spread: ring_w,
                                            },
                                            secondary: None,
                                            corner_radii: corners,
                                        }),
                                        ..Default::default()
                                    },
                                    |_cx| Vec::<AnyElement>::new(),
                                )]
                            }));
                        }
                    }
                }

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

    impl fret_core::MaterialService for FakeServices {
        fn register_material(
            &mut self,
            _desc: fret_core::MaterialDescriptor,
        ) -> Result<fret_core::MaterialId, fret_core::MaterialRegistrationError> {
            Ok(fret_core::MaterialId::default())
        }

        fn unregister_material(&mut self, _id: fret_core::MaterialId) -> bool {
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
