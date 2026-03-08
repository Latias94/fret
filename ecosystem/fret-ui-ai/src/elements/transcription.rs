//! AI Elements-aligned `Transcription` surfaces.
//!
//! Upstream reference: `repo-ref/ai-elements/packages/elements/src/transcription.tsx`.

use std::sync::Arc;

use fret_core::{Color, Edges, Px, SemanticsRole, TextOverflow, TextStyle, TextWrap};
use fret_runtime::Model;
use fret_ui::action::{ActionCx, OnActivate};
use fret_ui::element::{
    AnyElement, FlexProps, LayoutStyle, Length, PressableProps, SemanticsDecoration, TextProps,
};
use fret_ui::{ElementContext, Invalidation, Theme, UiHost};
use fret_ui_kit::LayoutRefinement;
use fret_ui_kit::declarative::controllable_state;
use fret_ui_kit::declarative::style as decl_style;
use fret_ui_kit::typography;

pub type OnTranscriptionSeek =
    Arc<dyn Fn(&mut dyn fret_ui::action::UiActionHost, ActionCx, f32) + 'static>;

#[derive(Debug, Clone, PartialEq)]
pub struct TranscriptionSegmentData {
    pub start_second: f32,
    pub end_second: f32,
    pub text: Arc<str>,
}

impl TranscriptionSegmentData {
    pub fn new(start_second: f32, end_second: f32, text: impl Into<Arc<str>>) -> Self {
        Self {
            start_second,
            end_second,
            text: text.into(),
        }
    }
}

#[derive(Clone)]
pub struct TranscriptionController {
    pub current_time: Model<f32>,
    pub segments: Arc<[TranscriptionSegmentData]>,
    pub on_seek: Option<OnTranscriptionSeek>,
}

impl std::fmt::Debug for TranscriptionController {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("TranscriptionController")
            .field("current_time", &self.current_time.id())
            .field("segments_len", &self.segments.len())
            .field("has_on_seek", &self.on_seek.is_some())
            .finish()
    }
}

#[derive(Debug, Default, Clone)]
struct TranscriptionProviderState {
    controller: Option<TranscriptionController>,
}

pub fn use_transcription_controller<H: UiHost>(
    cx: &ElementContext<'_, H>,
) -> Option<TranscriptionController> {
    cx.inherited_state::<TranscriptionProviderState>()
        .and_then(|st| st.controller.clone())
}

fn alpha_mul(mut c: Color, mul: f32) -> Color {
    c.a = (c.a * mul).clamp(0.0, 1.0);
    c
}

fn muted_fg(theme: &Theme) -> Color {
    theme
        .color_by_key("muted-foreground")
        .or_else(|| theme.color_by_key("muted_foreground"))
        .unwrap_or_else(|| theme.color_token("foreground"))
}

fn primary(theme: &Theme) -> Color {
    theme
        .color_by_key("primary")
        .unwrap_or_else(|| theme.color_token("foreground"))
}

fn is_blank(s: &str) -> bool {
    s.trim().is_empty()
}

/// Root provider aligned with AI Elements `Transcription`.
#[derive(Clone)]
pub struct Transcription {
    segments: Arc<[TranscriptionSegmentData]>,
    current_time: Option<Model<f32>>,
    default_current_time: f32,
    on_seek: Option<OnTranscriptionSeek>,
    test_id_root: Option<Arc<str>>,
    layout: LayoutRefinement,
}

impl std::fmt::Debug for Transcription {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Transcription")
            .field("segments_len", &self.segments.len())
            .field("has_current_time", &self.current_time.is_some())
            .field("default_current_time", &self.default_current_time)
            .field("has_on_seek", &self.on_seek.is_some())
            .field("test_id_root", &self.test_id_root.as_deref())
            .field("layout", &self.layout)
            .finish()
    }
}

impl Transcription {
    pub fn new(segments: impl IntoIterator<Item = TranscriptionSegmentData>) -> Self {
        Self::from_arc(segments.into_iter().collect::<Vec<_>>().into())
    }

    pub fn from_arc(segments: Arc<[TranscriptionSegmentData]>) -> Self {
        Self {
            segments,
            current_time: None,
            default_current_time: 0.0,
            on_seek: None,
            test_id_root: None,
            layout: LayoutRefinement::default().min_w_0(),
        }
    }

    pub fn current_time_model(mut self, model: Model<f32>) -> Self {
        self.current_time = Some(model);
        self
    }

    pub fn default_current_time(mut self, time: f32) -> Self {
        self.default_current_time = time.max(0.0);
        self
    }

    pub fn on_seek(mut self, cb: OnTranscriptionSeek) -> Self {
        self.on_seek = Some(cb);
        self
    }

    pub fn test_id_root(mut self, id: impl Into<Arc<str>>) -> Self {
        self.test_id_root = Some(id.into());
        self
    }

    pub fn refine_layout(mut self, layout: LayoutRefinement) -> Self {
        self.layout = self.layout.merge(layout);
        self
    }

    pub fn into_element<H: UiHost + 'static>(self, cx: &mut ElementContext<'_, H>) -> AnyElement {
        self.into_element_with_children(cx, |cx, seg, index| {
            TranscriptionSegment::new(seg, index).into_element(cx)
        })
    }

    pub fn into_element_with_children<H: UiHost + 'static>(
        self,
        cx: &mut ElementContext<'_, H>,
        children: impl for<'a> Fn(
            &mut ElementContext<'a, H>,
            TranscriptionSegmentData,
            usize,
        ) -> AnyElement
        + 'static,
    ) -> AnyElement {
        let children: Arc<
            dyn for<'a> Fn(
                    &mut ElementContext<'a, H>,
                    TranscriptionSegmentData,
                    usize,
                ) -> AnyElement
                + 'static,
        > = Arc::new(children);

        let theme = Theme::global(&*cx.app).clone();

        let controlled_time = self.current_time.clone();
        let default_time = self.default_current_time;
        let on_seek = self.on_seek.clone();
        let segments = self.segments.clone();
        let layout = self.layout;
        let test_id_root = self.test_id_root.clone();

        let root = cx.flex(
            FlexProps {
                layout: decl_style::layout_style(&theme, layout),
                direction: fret_core::Axis::Horizontal,
                gap: Px(4.0).into(),
                padding: Edges::all(Px(0.0)).into(),
                justify: fret_ui::element::MainAlign::Start,
                align: fret_ui::element::CrossAlign::Start,
                wrap: true,
            },
            move |cx| {
                let current_time =
                    controllable_state::use_controllable_model(cx, controlled_time.clone(), || {
                        default_time
                    })
                    .model();

                let controller = TranscriptionController {
                    current_time,
                    segments: segments.clone(),
                    on_seek: on_seek.clone(),
                };

                cx.with_state(TranscriptionProviderState::default, |st| {
                    st.controller = Some(controller.clone());
                });

                let mut out: Vec<AnyElement> = Vec::new();
                for (raw_index, seg) in segments.iter().cloned().enumerate() {
                    if is_blank(seg.text.as_ref()) {
                        continue;
                    }
                    let children = children.clone();
                    out.push(
                        cx.keyed(format!("transcription-seg-{raw_index}"), move |cx| {
                            children(cx, seg.clone(), raw_index)
                        }),
                    );
                }
                out
            },
        );

        let Some(test_id) = test_id_root else {
            return root;
        };
        root.attach_semantics(
            SemanticsDecoration::default()
                .role(SemanticsRole::Group)
                .test_id(test_id),
        )
    }
}

impl Default for Transcription {
    fn default() -> Self {
        Self::from_arc(Arc::from([]))
    }
}

/// Inline segment aligned with AI Elements `TranscriptionSegment`.
#[derive(Clone)]
pub struct TranscriptionSegment {
    segment: TranscriptionSegmentData,
    index: usize,
    test_id: Option<Arc<str>>,
    text_style: Option<TextStyle>,
    on_activate: Option<OnActivate>,
    layout: LayoutRefinement,
}

impl std::fmt::Debug for TranscriptionSegment {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("TranscriptionSegment")
            .field("index", &self.index)
            .field("start_second", &self.segment.start_second)
            .field("end_second", &self.segment.end_second)
            .field("text", &self.segment.text)
            .field("test_id", &self.test_id.as_deref())
            .field("text_style", &self.text_style)
            .field("has_on_activate", &self.on_activate.is_some())
            .finish()
    }
}

impl TranscriptionSegment {
    pub fn new(segment: TranscriptionSegmentData, index: usize) -> Self {
        Self {
            segment,
            index,
            test_id: None,
            text_style: None,
            on_activate: None,
            layout: LayoutRefinement::default(),
        }
    }

    pub fn test_id(mut self, id: impl Into<Arc<str>>) -> Self {
        self.test_id = Some(id.into());
        self
    }

    pub fn refine_layout(mut self, layout: LayoutRefinement) -> Self {
        self.layout = self.layout.merge(layout);
        self
    }

    /// Override the segment text style while keeping the default active/past/future coloring.
    pub fn text_style(mut self, style: TextStyle) -> Self {
        self.text_style = Some(style);
        self
    }

    /// Compose additional activation behavior after the upstream-aligned `on_seek` callback.
    pub fn on_activate(mut self, on_activate: OnActivate) -> Self {
        self.on_activate = Some(on_activate);
        self
    }

    pub fn into_element<H: UiHost + 'static>(self, cx: &mut ElementContext<'_, H>) -> AnyElement {
        let Some(controller) = use_transcription_controller(cx) else {
            return cx.text("");
        };

        let theme = Theme::global(&*cx.app).clone();
        let current_time = cx
            .get_model_copied(&controller.current_time, Invalidation::Paint)
            .unwrap_or(0.0);

        let seg = self.segment;
        let is_active = current_time >= seg.start_second && current_time < seg.end_second;
        let is_past = current_time >= seg.end_second;

        let on_seek = controller.on_seek.clone();
        let on_activate = self.on_activate;

        let base = if is_active {
            primary(&theme)
        } else if is_past {
            muted_fg(&theme)
        } else {
            alpha_mul(muted_fg(&theme), 0.6)
        };

        let fg_on_hover = theme.color_token("foreground");

        let test_id = self.test_id;
        let layout = self.layout;

        let style = self.text_style.unwrap_or_else(|| {
            typography::preset_text_style_with_overrides(
                &theme,
                typography::TypographyPreset::control_ui(typography::UiTextSize::Sm),
                Some(fret_core::FontWeight::NORMAL),
                None,
            )
        });
        let text = seg.text.clone();

        cx.pressable_with_id_props(move |cx, st, _id| {
            let mut pressable = PressableProps::default();
            pressable.enabled = true;
            pressable.focusable = true;
            pressable.a11y.role = Some(SemanticsRole::Button);
            pressable.a11y.label = Some(text.clone());
            pressable.a11y.test_id = test_id.clone();
            pressable.layout = decl_style::layout_style(&theme, layout);

            let fg = if st.hovered && on_seek.is_some() {
                fg_on_hover
            } else {
                base
            };

            if on_seek.is_some() || on_activate.is_some() {
                cx.pressable_on_activate({
                    let on_seek = on_seek.clone();
                    let on_activate = on_activate.clone();
                    let start = seg.start_second;
                    Arc::new(move |host, action_cx, reason| {
                        if let Some(cb) = on_seek.clone() {
                            cb(host, action_cx, start);
                        }
                        if let Some(cb) = on_activate.clone() {
                            cb(host, action_cx, reason);
                        }
                        host.notify(action_cx);
                        host.request_redraw(action_cx.window);
                    })
                });
            }

            let mut text_props = TextProps {
                layout: LayoutStyle::default(),
                text: text.clone(),
                style: Some(style.clone()),
                color: Some(fg),
                wrap: TextWrap::None,
                overflow: TextOverflow::Clip,
                align: fret_core::TextAlign::Start,
                ink_overflow: Default::default(),
            };
            text_props.layout.size.width = Length::Auto;
            let content = cx.text_props(text_props);
            (pressable, vec![content])
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use fret_app::App;
    use fret_core::{AppWindowId, Point, Px, Rect, Size};
    use fret_ui::element::{ElementKind, Length};

    fn bounds() -> Rect {
        Rect::new(
            Point::new(Px(0.0), Px(0.0)),
            Size::new(Px(480.0), Px(240.0)),
        )
    }

    fn single_segment() -> Arc<[TranscriptionSegmentData]> {
        Arc::from(vec![TranscriptionSegmentData::new(0.0, 1.0, "Hello")])
    }

    #[test]
    fn transcription_root_defaults_do_not_force_fill_width() {
        let window = AppWindowId::default();
        let mut app = App::new();

        let el =
            fret_ui::elements::with_element_cx(&mut app, window, bounds(), "transcription", |cx| {
                Transcription::from_arc(single_segment()).into_element(cx)
            });

        let ElementKind::Flex(props) = &el.kind else {
            panic!("expected Transcription root to render a flex container");
        };

        assert_ne!(props.layout.size.width, Length::Fill);
        assert_eq!(props.layout.size.min_width, Some(Length::Px(Px(0.0))));
    }

    #[test]
    fn transcription_segment_without_seek_still_renders_pressable_button() {
        let window = AppWindowId::default();
        let mut app = App::new();

        let el = fret_ui::elements::with_element_cx(
            &mut app,
            window,
            bounds(),
            "transcription-segment",
            |cx| Transcription::from_arc(single_segment()).into_element(cx),
        );

        let segment = el.children.first().expect("expected segment child");
        let ElementKind::Pressable(props) = &segment.kind else {
            panic!("expected TranscriptionSegment to render as a pressable button");
        };

        assert!(
            props.focusable,
            "expected upstream-aligned button semantics"
        );
        assert_eq!(props.a11y.role, Some(SemanticsRole::Button));
        assert_eq!(props.a11y.label.as_deref(), Some("Hello"));
    }

    #[test]
    fn transcription_segment_text_style_override_applies_to_text_child() {
        let window = AppWindowId::default();
        let mut app = App::new();
        let override_style = TextStyle {
            font: fret_core::FontId::default(),
            size: Px(18.0),
            line_height: Some(Px(28.0)),
            ..Default::default()
        };

        let el = fret_ui::elements::with_element_cx(
            &mut app,
            window,
            bounds(),
            "transcription-style",
            |cx| {
                Transcription::from_arc(single_segment()).into_element_with_children(cx, {
                    let override_style = override_style.clone();
                    move |cx, seg, index| {
                        TranscriptionSegment::new(seg, index)
                            .text_style(override_style.clone())
                            .into_element(cx)
                    }
                })
            },
        );

        let segment = el.children.first().expect("expected segment child");
        let ElementKind::Pressable(_) = &segment.kind else {
            panic!("expected segment root to remain a pressable");
        };
        let text = segment.children.first().expect("expected text child");
        let ElementKind::Text(props) = &text.kind else {
            panic!("expected segment content to render as text");
        };
        let style = props.style.as_ref().expect("expected text style");

        assert_eq!(style.size, Px(18.0));
        assert_eq!(style.line_height, Some(Px(28.0)));
    }

    #[test]
    fn transcription_segment_default_text_uses_shared_sm_typography_preset() {
        let window = AppWindowId::default();
        let mut app = App::new();

        let el = fret_ui::elements::with_element_cx(
            &mut app,
            window,
            bounds(),
            "transcription-default-style",
            |cx| Transcription::from_arc(single_segment()).into_element(cx),
        );

        let segment = el.children.first().expect("expected segment child");
        let text = segment.children.first().expect("expected text child");
        let ElementKind::Text(props) = &text.kind else {
            panic!("expected segment content to render as text");
        };

        let theme = Theme::global(&app).clone();
        assert_eq!(
            props.style,
            Some(typography::preset_text_style_with_overrides(
                &theme,
                typography::TypographyPreset::control_ui(typography::UiTextSize::Sm),
                Some(fret_core::FontWeight::NORMAL),
                None,
            ))
        );
    }
}
