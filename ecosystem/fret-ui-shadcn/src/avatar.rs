use std::sync::Arc;
use std::time::Duration;

use fret_core::{ImageId, Px, ViewportFit};
use fret_runtime::Model;
use fret_ui::element::{
    AnyElement, ContainerProps, CrossAlign, ElementKind, FlexProps, ImageProps,
    InteractivityGateProps, LayoutStyle, Length, MainAlign, Overflow, SizeStyle,
};
use fret_ui::{ElementContext, Theme, UiHost};
use fret_ui_kit::declarative::model_watch::ModelWatchExt as _;
use fret_ui_kit::declarative::scheduling;
use fret_ui_kit::declarative::style as decl_style;
use fret_ui_kit::primitives::avatar as radix_avatar;
use fret_ui_kit::{
    ChromeRefinement, ColorFallback, ColorRef, LayoutRefinement, MetricRef, Radius, Space, ui,
};

use crate::test_id::attach_test_id;

const AVATAR_BADGE_MARKER_PREFIX: &str = "fret-ui-shadcn.avatar-badge";

fn matches_marker(test_id: &str, prefix: &str) -> bool {
    test_id == prefix
        || (test_id.starts_with(prefix)
            && test_id
                .as_bytes()
                .get(prefix.len())
                .is_some_and(|b| *b == b':'))
}

fn is_avatar_badge_marker(element: &AnyElement) -> bool {
    element
        .semantics_decoration
        .as_ref()
        .and_then(|d| d.test_id.as_deref())
        .is_some_and(|id| matches_marker(id, AVATAR_BADGE_MARKER_PREFIX))
        || match &element.kind {
            ElementKind::Semantics(props) => props
                .test_id
                .as_deref()
                .is_some_and(|id| matches_marker(id, AVATAR_BADGE_MARKER_PREFIX)),
            _ => false,
        }
}

#[derive(Debug, Clone, Copy, Default)]
struct AvatarSizeProviderState {
    current: AvatarSize,
}

fn inherited_avatar_size<H: UiHost>(cx: &ElementContext<'_, H>) -> Option<AvatarSize> {
    cx.provided::<AvatarSizeProviderState>()
        .map(|st| st.current)
}

#[track_caller]
fn with_avatar_size_provider<H: UiHost, R>(
    cx: &mut ElementContext<'_, H>,
    size: AvatarSize,
    f: impl FnOnce(&mut ElementContext<'_, H>) -> R,
) -> R {
    cx.provide(AvatarSizeProviderState { current: size }, f)
}

/// shadcn/ui avatar size variants (`size="sm" | "default" | "lg"`).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum AvatarSize {
    #[default]
    Default,
    Sm,
    Lg,
}

/// Build an avatar and its size-dependent parts inside an avatar size provider.
///
/// This avoids footguns where callers construct `AvatarBadge` / `AvatarGroupCount` elements outside
/// the `Avatar` subtree and accidentally miss inherited size defaults.
pub fn avatar_sized<H: UiHost, I>(
    cx: &mut ElementContext<'_, H>,
    size: AvatarSize,
    f: impl FnOnce(&mut ElementContext<'_, H>) -> I,
) -> AnyElement
where
    I: IntoIterator<Item = AnyElement>,
{
    let children = with_avatar_size_provider(cx, size, |cx| f(cx).into_iter().collect::<Vec<_>>());
    Avatar::new(children).size(size).into_element(cx)
}

/// shadcn/ui `Avatar` root.
///
/// This is a fixed-size, fully-rounded container intended to host exactly one `AvatarImage` and
/// one `AvatarFallback` (order controls paint stacking).
///
/// Note: Upstream shadcn/ui sets `overflow-hidden` on the avatar root. When an [`AvatarBadge`] is
/// present, Fret wraps the clipped core in an overflow-visible wrapper so the badge can render
/// outside the circle without being clipped.
#[derive(Debug)]
pub struct Avatar {
    children: Vec<AnyElement>,
    size: AvatarSize,
    chrome: ChromeRefinement,
    layout: LayoutRefinement,
}

impl Avatar {
    pub fn new(children: impl IntoIterator<Item = AnyElement>) -> Self {
        let children = children.into_iter().collect();
        Self {
            children,
            size: AvatarSize::default(),
            chrome: ChromeRefinement::default(),
            layout: LayoutRefinement::default(),
        }
    }

    pub fn children(mut self, children: impl IntoIterator<Item = AnyElement>) -> Self {
        self.children = children.into_iter().collect();
        self
    }

    pub fn size(mut self, size: AvatarSize) -> Self {
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

    #[track_caller]
    pub fn into_element<H: UiHost>(self, cx: &mut ElementContext<'_, H>) -> AnyElement {
        let size_variant = self.size;
        with_avatar_size_provider(cx, size_variant, |cx| {
            let theme = Theme::global(&*cx.app).snapshot();

            let size = match size_variant {
                AvatarSize::Sm => Space::N6,
                AvatarSize::Default => Space::N8,
                AvatarSize::Lg => Space::N10,
            };

            let base_chrome = ChromeRefinement::default().rounded(Radius::Full);
            let base_layout = LayoutRefinement::default()
                .relative()
                .w_px(MetricRef::space(size))
                .h_px(MetricRef::space(size))
                .flex_shrink_0();

            let mut badge: Option<AnyElement> = None;
            let mut core_children: Vec<AnyElement> = Vec::with_capacity(self.children.len());
            for child in self.children {
                if badge.is_none() && is_avatar_badge_marker(&child) {
                    badge = Some(child);
                } else {
                    core_children.push(child);
                }
            }

            let (core_layout, wrapper_layout) = match badge.as_ref() {
                None => (base_layout.merge(self.layout), None),
                Some(_) => (
                    LayoutRefinement::default()
                        .absolute()
                        .inset(Space::N0)
                        .size_full(),
                    Some(base_layout.merge(self.layout)),
                ),
            };

            let mut core_props =
                decl_style::container_props(&theme, base_chrome.merge(self.chrome), core_layout);
            core_props.layout.overflow = Overflow::Clip;

            let core = cx.container(core_props, move |_cx| core_children);

            match (wrapper_layout, badge) {
                (None, None) => core,
                (Some(wrapper_layout), Some(badge)) => {
                    let wrapper_props = decl_style::container_props(
                        &theme,
                        ChromeRefinement::default(),
                        wrapper_layout,
                    );
                    cx.container(wrapper_props, move |_cx| vec![core, badge])
                }
                _ => core,
            }
        })
    }
}

/// shadcn/ui `AvatarBadge`.
///
/// Positioned at the bottom-right of the current `Avatar` size scope.
#[derive(Debug, Default)]
pub struct AvatarBadge {
    children: Vec<AnyElement>,
    size: Option<AvatarSize>,
    chrome: ChromeRefinement,
    layout: LayoutRefinement,
}

impl AvatarBadge {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn children(mut self, children: impl IntoIterator<Item = AnyElement>) -> Self {
        self.children = children.into_iter().collect();
        self
    }

    /// Explicitly set the avatar size for this badge.
    ///
    /// Most compositions rely on `Avatar` installing a size provider; however, callers sometimes
    /// build `AvatarBadge` before inserting it into an `Avatar` subtree. In those cases, inherited
    /// size is unavailable, so callers can pass an explicit size to match upstream shadcn behavior.
    pub fn size(mut self, size: AvatarSize) -> Self {
        self.size = Some(size);
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

    #[track_caller]
    pub fn into_element<H: UiHost>(self, cx: &mut ElementContext<'_, H>) -> AnyElement {
        let size = self
            .size
            .or_else(|| inherited_avatar_size(cx))
            .unwrap_or_default();
        let (dot_px, icon_px, hide_icon) = match size {
            AvatarSize::Sm => (Px(8.0), Px(0.0), true),
            AvatarSize::Default => (Px(10.0), Px(8.0), false),
            AvatarSize::Lg => (Px(12.0), Px(8.0), false),
        };

        let chrome = ChromeRefinement::default()
            .rounded(Radius::Full)
            .border_width(Px(2.0))
            .border_color(ColorRef::Token {
                key: "background",
                fallback: ColorFallback::ThemeSurfaceBackground,
            })
            .bg(ColorRef::Token {
                key: "primary",
                fallback: ColorFallback::ThemeAccent,
            })
            .merge(self.chrome);

        let layout = LayoutRefinement::default()
            .absolute()
            .right_px(Px(0.0))
            .bottom_px(Px(0.0))
            .w_px(MetricRef::Px(dot_px))
            .h_px(MetricRef::Px(dot_px))
            .merge(self.layout);

        let props = {
            let theme = Theme::global(&*cx.app);
            let mut props = decl_style::container_props(theme, chrome, layout);
            props.layout.overflow = Overflow::Visible;
            props
        };

        let children = if hide_icon { Vec::new() } else { self.children };

        let el = cx.container(props, move |cx| {
            vec![cx.flex(
                FlexProps {
                    layout: LayoutStyle::default(),
                    direction: fret_core::Axis::Horizontal,
                    gap: Px(0.0).into(),
                    padding: fret_core::Edges::all(Px(0.0)).into(),
                    justify: MainAlign::Center,
                    align: CrossAlign::Center,
                    wrap: false,
                },
                move |cx| {
                    let mut children = Some(children);
                    if children.as_ref().is_some_and(|c| c.is_empty()) {
                        Vec::new()
                    } else {
                        let children = children.take().unwrap_or_default();
                        vec![cx.container(
                            {
                                let theme = Theme::global(&*cx.app);
                                decl_style::container_props(
                                    theme,
                                    ChromeRefinement::default(),
                                    LayoutRefinement::default()
                                        .w_px(MetricRef::Px(icon_px))
                                        .h_px(MetricRef::Px(icon_px)),
                                )
                            },
                            move |_cx| children,
                        )]
                    }
                },
            )]
        });

        let marker: Arc<str> = Arc::from(format!("{}:{}", AVATAR_BADGE_MARKER_PREFIX, el.id.0));
        attach_test_id(el, marker)
    }
}

/// shadcn/ui `AvatarGroup`.
///
/// Overlap wrapper (`-space-x-2` in Tailwind) + installs a size scope so `AvatarGroupCount` can
/// match the intended avatar size.
#[derive(Debug)]
pub struct AvatarGroup {
    children: Vec<AnyElement>,
    size: Option<AvatarSize>,
    chrome: ChromeRefinement,
    layout: LayoutRefinement,
}

impl AvatarGroup {
    pub fn new(children: impl IntoIterator<Item = AnyElement>) -> Self {
        Self {
            children: children.into_iter().collect(),
            size: None,
            chrome: ChromeRefinement::default(),
            layout: LayoutRefinement::default(),
        }
    }

    pub fn size(mut self, size: AvatarSize) -> Self {
        self.size = Some(size);
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

    #[track_caller]
    pub fn into_element<H: UiHost>(self, cx: &mut ElementContext<'_, H>) -> AnyElement {
        let run = |cx: &mut ElementContext<'_, H>| {
            let props = {
                let theme = Theme::global(&*cx.app);
                decl_style::container_props(theme, self.chrome, self.layout)
            };

            let mut out = Vec::new();
            for (idx, child) in self.children.into_iter().enumerate() {
                if idx == 0 {
                    out.push(child);
                } else {
                    out.push(cx.container(
                        {
                            let theme = Theme::global(&*cx.app);
                            decl_style::container_props(
                                theme,
                                ChromeRefinement::default(),
                                LayoutRefinement::default().ml_neg(Space::N2),
                            )
                        },
                        move |_cx| vec![child],
                    ));
                }
            }

            cx.container(props, move |cx| {
                vec![cx.flex(
                    FlexProps {
                        layout: LayoutStyle::default(),
                        direction: fret_core::Axis::Horizontal,
                        gap: Px(0.0).into(),
                        padding: fret_core::Edges::all(Px(0.0)).into(),
                        justify: MainAlign::Start,
                        align: CrossAlign::Center,
                        wrap: false,
                    },
                    move |_cx| out,
                )]
            })
        };

        if let Some(size) = self.size {
            with_avatar_size_provider(cx, size, run)
        } else {
            run(cx)
        }
    }
}

/// shadcn/ui `AvatarGroupCount`.
#[derive(Debug, Default)]
pub struct AvatarGroupCount {
    children: Vec<AnyElement>,
    size: Option<AvatarSize>,
    chrome: ChromeRefinement,
    layout: LayoutRefinement,
}

impl AvatarGroupCount {
    pub fn new(children: impl IntoIterator<Item = AnyElement>) -> Self {
        Self {
            children: children.into_iter().collect(),
            size: None,
            chrome: ChromeRefinement::default(),
            layout: LayoutRefinement::default(),
        }
    }

    /// Explicitly set the avatar size for this count bubble.
    ///
    /// See `AvatarBadge::size(...)` for why some call sites need this.
    pub fn size(mut self, size: AvatarSize) -> Self {
        self.size = Some(size);
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

    #[track_caller]
    pub fn into_element<H: UiHost>(self, cx: &mut ElementContext<'_, H>) -> AnyElement {
        let size = self
            .size
            .or_else(|| inherited_avatar_size(cx))
            .unwrap_or_default();
        let (box_space, text_px) = match size {
            AvatarSize::Sm => (Space::N6, Px(12.0)),
            AvatarSize::Default => (Space::N8, Px(14.0)),
            AvatarSize::Lg => (Space::N10, Px(14.0)),
        };

        let chrome = ChromeRefinement::default()
            .rounded(Radius::Full)
            .border_width(Px(2.0))
            .border_color(ColorRef::Token {
                key: "background",
                fallback: ColorFallback::ThemeSurfaceBackground,
            })
            .bg(ColorRef::Token {
                key: "muted",
                fallback: ColorFallback::ThemePanelBackground,
            })
            .text_color(ColorRef::Token {
                key: "muted-foreground",
                fallback: ColorFallback::ThemeTextMuted,
            })
            .merge(self.chrome);

        let layout = LayoutRefinement::default()
            .w_px(MetricRef::space(box_space))
            .h_px(MetricRef::space(box_space))
            .flex_shrink_0()
            .merge(self.layout);

        let props = {
            let theme = Theme::global(&*cx.app);
            decl_style::container_props(theme, chrome, layout)
        };

        let children = self.children;
        cx.container(props, move |cx| {
            let mut children = Some(children);
            vec![cx.flex(
                FlexProps {
                    layout: LayoutStyle::default(),
                    direction: fret_core::Axis::Horizontal,
                    gap: Px(0.0).into(),
                    padding: fret_core::Edges::all(Px(0.0)).into(),
                    justify: MainAlign::Center,
                    align: CrossAlign::Center,
                    wrap: false,
                },
                move |cx| {
                    if children.as_ref().is_some_and(|c| c.is_empty()) {
                        vec![
                            ui::text("+3")
                                .text_size_px(text_px)
                                .font_medium()
                                .nowrap()
                                .into_element(cx),
                        ]
                    } else {
                        children.take().unwrap_or_default()
                    }
                },
            )]
        })
    }
}

/// shadcn/ui `AvatarImage`.
#[derive(Debug, Clone)]
pub struct AvatarImage {
    source: AvatarImageSource,
    opacity: f32,
    chrome: ChromeRefinement,
    layout: LayoutRefinement,
}

#[derive(Debug, Clone)]
enum AvatarImageSource {
    Ready(ImageId),
    Optional(Option<ImageId>),
    Model(Model<Option<ImageId>>),
}

impl AvatarImageSource {
    fn resolve<H: UiHost>(&self, cx: &mut ElementContext<'_, H>) -> Option<ImageId> {
        match self {
            Self::Ready(image) => Some(*image),
            Self::Optional(image) => *image,
            Self::Model(model) => cx.watch_model(model).copied().flatten(),
        }
    }
}

impl AvatarImage {
    pub fn new(image: ImageId) -> Self {
        Self {
            source: AvatarImageSource::Ready(image),
            opacity: 1.0,
            chrome: ChromeRefinement::default(),
            layout: LayoutRefinement::default(),
        }
    }

    pub fn maybe(image: Option<ImageId>) -> Self {
        Self {
            source: AvatarImageSource::Optional(image),
            opacity: 1.0,
            chrome: ChromeRefinement::default(),
            layout: LayoutRefinement::default(),
        }
    }

    pub fn model(image: Model<Option<ImageId>>) -> Self {
        Self {
            source: AvatarImageSource::Model(image),
            opacity: 1.0,
            chrome: ChromeRefinement::default(),
            layout: LayoutRefinement::default(),
        }
    }

    pub fn opacity(mut self, opacity: f32) -> Self {
        self.opacity = opacity;
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

    #[track_caller]
    pub fn into_element<H: UiHost>(self, cx: &mut ElementContext<'_, H>) -> AnyElement {
        cx.scope(|cx| {
            let id = cx.root_id();
            let image = self.source.resolve(cx);

            let present = image.is_some();
            let mut gate_layout = LayoutStyle::default();
            gate_layout.size = SizeStyle {
                width: Length::Fill,
                height: Length::Fill,
                ..Default::default()
            };
            let children = if let Some(image) = image {
                let theme = Theme::global(&*cx.app).snapshot();
                let layout = LayoutRefinement::default()
                    .absolute()
                    .inset(Space::N0)
                    .size_full()
                    .aspect_ratio(1.0)
                    .merge(self.layout);

                let mut wrapper = decl_style::container_props(
                    &theme,
                    ChromeRefinement::default()
                        .rounded(Radius::Full)
                        .merge(self.chrome),
                    layout,
                );
                wrapper.layout.overflow = Overflow::Clip;
                let opacity = self.opacity.clamp(0.0, 1.0);
                let mut image_layout = LayoutStyle::default();
                image_layout.size = SizeStyle {
                    width: Length::Fill,
                    height: Length::Fill,
                    ..Default::default()
                };

                vec![cx.container(wrapper, move |cx| {
                    vec![cx.image_props(ImageProps {
                        layout: image_layout,
                        fit: ViewportFit::Cover,
                        opacity,
                        uv: None,
                        ..ImageProps::new(image)
                    })]
                })]
            } else {
                Vec::new()
            };

            AnyElement::new(
                id,
                ElementKind::InteractivityGate(InteractivityGateProps {
                    layout: gate_layout,
                    present,
                    interactive: false,
                    ..Default::default()
                }),
                children,
            )
        })
    }
}

/// shadcn/ui `AvatarFallback`.
#[derive(Debug, Clone)]
pub struct AvatarFallback {
    text: Arc<str>,
    show_when_image_missing: Option<AvatarImageSource>,
    delay: Option<Duration>,
    chrome: ChromeRefinement,
    layout: LayoutRefinement,
}

impl AvatarFallback {
    pub fn new(text: impl Into<Arc<str>>) -> Self {
        Self {
            text: text.into(),
            show_when_image_missing: None,
            delay: None,
            chrome: ChromeRefinement::default(),
            layout: LayoutRefinement::default(),
        }
    }

    /// Gate fallback rendering to the "image is not loaded" outcome, using an optional image id.
    pub fn when_image_missing(mut self, image: Option<ImageId>) -> Self {
        self.show_when_image_missing = Some(AvatarImageSource::Optional(image));
        self
    }

    /// Gate fallback rendering to the "image is not loaded" outcome, using an image model.
    pub fn when_image_missing_model(mut self, image: Model<Option<ImageId>>) -> Self {
        self.show_when_image_missing = Some(AvatarImageSource::Model(image));
        self
    }

    /// Delay rendering fallback by a number of frames (60fps-ish ticks).
    pub fn delay_frames(mut self, frames: u64) -> Self {
        const NS_PER_TICK_60HZ: u64 = 1_000_000_000 / 60;
        let nanos = (frames as u128)
            .saturating_mul(NS_PER_TICK_60HZ as u128)
            .min(u64::MAX as u128) as u64;
        self.delay = Some(Duration::from_nanos(nanos));
        self
    }

    /// Delay rendering fallback by a millisecond duration (Radix `delayMs` outcome).
    pub fn delay_ms(mut self, ms: u64) -> Self {
        self.delay = Some(Duration::from_millis(ms));
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

    #[track_caller]
    pub fn into_element<H: UiHost>(self, cx: &mut ElementContext<'_, H>) -> AnyElement {
        cx.scope(|cx| {
            let id = cx.root_id();
            let theme = Theme::global(&*cx.app).snapshot();

            let image_loaded = self
                .show_when_image_missing
                .as_ref()
                .and_then(|src| src.resolve(cx))
                .is_some();
            let status = if image_loaded {
                radix_avatar::AvatarImageLoadingStatus::Loaded
            } else {
                radix_avatar::AvatarImageLoadingStatus::Loading
            };

            let want_render = match &self.show_when_image_missing {
                Some(_) => !image_loaded,
                None => true,
            };

            let frame_id = cx.frame_id.0;
            let dt = fret_ui_kit::declarative::motion::effective_frame_delta_for_cx(cx);
            let delay = self.delay;
            let delay_ready =
                cx.with_state_for(id, radix_avatar::AvatarFallbackDelay::default, |gate| {
                    gate.drive(frame_id, dt, delay, want_render)
                });
            scheduling::set_continuous_frames(
                cx,
                want_render && self.delay.is_some() && !delay_ready,
            );

            let present = if self.show_when_image_missing.is_some() {
                radix_avatar::fallback_visible(status, delay_ready)
            } else {
                delay_ready
            };

            let bg = theme.color_token("muted");

            let base_chrome = ChromeRefinement::default()
                .rounded(Radius::Full)
                .bg(ColorRef::Color(bg));

            let base_layout = LayoutRefinement::default()
                .absolute()
                .inset(Space::N0)
                .size_full()
                .aspect_ratio(1.0);

            let props = decl_style::container_props(
                &theme,
                base_chrome.merge(self.chrome),
                base_layout.merge(self.layout),
            );

            let text_px = theme
                .metric_by_key("component.avatar.fallback_text_px")
                .or_else(|| theme.metric_by_key("font.size"))
                .unwrap_or_else(|| theme.metric_token("font.size"));
            let line_height = theme
                .metric_by_key("component.avatar.fallback_line_height")
                .or_else(|| theme.metric_by_key("font.line_height"))
                .unwrap_or_else(|| theme.metric_token("font.line_height"));

            let label = ui::label(self.text)
                .text_size_px(text_px)
                .line_height_px(line_height)
                .nowrap()
                .into_element(cx);

            let flex_layout =
                decl_style::layout_style(&theme, LayoutRefinement::default().size_full());
            let flex = cx.flex(
                FlexProps {
                    layout: flex_layout,
                    direction: fret_core::Axis::Horizontal,
                    gap: Px(0.0).into(),
                    padding: fret_core::Edges::all(Px(0.0)).into(),
                    justify: MainAlign::Center,
                    align: CrossAlign::Center,
                    wrap: false,
                },
                move |_cx| vec![label],
            );

            let child = cx.container(ContainerProps { ..props }, move |_cx| vec![flex]);

            AnyElement::new(
                id,
                ElementKind::InteractivityGate(InteractivityGateProps {
                    layout: LayoutStyle {
                        size: SizeStyle {
                            width: Length::Fill,
                            height: Length::Fill,
                            ..Default::default()
                        },
                        ..Default::default()
                    },
                    present,
                    interactive: false,
                    ..Default::default()
                }),
                vec![child],
            )
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use std::time::Duration;

    use fret_app::App;
    use fret_core::{
        AppWindowId, PathCommand, PathConstraints, PathId, PathMetrics, PathService, PathStyle,
        Point, Px, Rect, SemanticsRole, Size as CoreSize, SvgId, SvgService, TextBlobId,
        TextConstraints, TextMetrics, TextService, WindowFrameClockService,
    };
    use fret_runtime::{Effect, FrameId};
    use fret_ui::element::ElementKind;
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
                    size: CoreSize::new(Px(10.0), Px(10.0)),
                    baseline: Px(8.0),
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

    fn render_frame(
        ui: &mut UiTree<App>,
        app: &mut App,
        services: &mut dyn fret_core::UiServices,
        window: AppWindowId,
        bounds: Rect,
        image: Model<Option<ImageId>>,
        delay_ms: u64,
    ) {
        let root =
            fret_ui::declarative::render_root(ui, app, services, window, bounds, "test", |cx| {
                let image_el = AvatarImage::model(image.clone()).into_element(cx);
                let fallback_el = AvatarFallback::new("JD")
                    .when_image_missing_model(image.clone())
                    .delay_ms(delay_ms)
                    .into_element(cx);
                vec![Avatar::new(vec![image_el, fallback_el]).into_element(cx)]
            });
        ui.set_root(root);
    }

    fn snapshot_contains_label(snap: &fret_core::SemanticsSnapshot, text: &str) -> bool {
        snap.nodes.iter().any(|n| {
            n.role == SemanticsRole::Text
                && (n.label.as_deref() == Some(text) || n.value.as_deref() == Some(text))
        })
    }

    #[test]
    fn avatar_badge_can_inherit_or_override_size() {
        let window = AppWindowId::default();
        let mut app = App::new();
        let bounds = Rect::new(
            Point::new(Px(0.0), Px(0.0)),
            fret_core::Size::new(Px(200.0), Px(120.0)),
        );

        fret_ui::elements::with_element_cx(&mut app, window, bounds, "test", |cx| {
            fn badge_box_px(el: &AnyElement) -> Px {
                let ElementKind::Container(ContainerProps { layout, .. }) = &el.kind else {
                    panic!("expected AvatarBadge to be a container element");
                };
                match layout.size.width {
                    Length::Px(px) => px,
                    other => panic!("expected AvatarBadge width to be px, got {other:?}"),
                }
            }

            let default_el = AvatarBadge::new().into_element(cx);
            assert_eq!(badge_box_px(&default_el), Px(10.0));

            let inherited_sm_el = with_avatar_size_provider(cx, AvatarSize::Sm, |cx| {
                AvatarBadge::new().into_element(cx)
            });
            assert_eq!(badge_box_px(&inherited_sm_el), Px(8.0));

            let explicit_sm_el = AvatarBadge::new().size(AvatarSize::Sm).into_element(cx);
            assert_eq!(badge_box_px(&explicit_sm_el), Px(8.0));
        });
    }

    #[test]
    fn avatar_group_count_can_inherit_or_override_size() {
        let window = AppWindowId::default();
        let mut app = App::new();
        let bounds = Rect::new(
            Point::new(Px(0.0), Px(0.0)),
            fret_core::Size::new(Px(200.0), Px(120.0)),
        );

        fret_ui::elements::with_element_cx(&mut app, window, bounds, "test", |cx| {
            fn count_box_px(el: &AnyElement) -> Px {
                let ElementKind::Container(ContainerProps { layout, .. }) = &el.kind else {
                    panic!("expected AvatarGroupCount to be a container element");
                };
                match layout.size.width {
                    Length::Px(px) => px,
                    other => panic!("expected AvatarGroupCount width to be px, got {other:?}"),
                }
            }

            let inherited_sm_el = with_avatar_size_provider(cx, AvatarSize::Sm, |cx| {
                AvatarGroupCount::new(Vec::<AnyElement>::new()).into_element(cx)
            });
            let explicit_sm_el = AvatarGroupCount::new(Vec::<AnyElement>::new())
                .size(AvatarSize::Sm)
                .into_element(cx);
            assert_eq!(
                count_box_px(&inherited_sm_el),
                count_box_px(&explicit_sm_el)
            );
        });
    }

    #[test]
    fn avatar_fallback_delay_gates_text_until_elapsed() {
        let window = AppWindowId::default();
        let mut app = App::new();
        let mut ui: UiTree<App> = UiTree::new();
        ui.set_window(window);

        app.with_global_mut(WindowFrameClockService::default, |svc, _app| {
            svc.set_fixed_delta(window, Some(Duration::from_millis(16)));
        });

        let image = app.models_mut().insert(None::<ImageId>);

        let mut services = FakeServices::default();
        let bounds = Rect::new(
            Point::new(Px(0.0), Px(0.0)),
            fret_core::Size::new(Px(200.0), Px(120.0)),
        );

        let delay_ms = 32;
        for frame in 1..=3 {
            app.set_frame_id(FrameId(frame));
            render_frame(
                &mut ui,
                &mut app,
                &mut services,
                window,
                bounds,
                image.clone(),
                delay_ms,
            );
            ui.request_semantics_snapshot();
            ui.layout_all(&mut app, &mut services, bounds, 1.0);
            let snap = ui.semantics_snapshot().expect("semantics snapshot");
            let visible = snapshot_contains_label(&snap, "JD");
            assert_eq!(visible, frame >= 3);
        }
    }

    #[test]
    fn avatar_fallback_hides_when_image_becomes_available() {
        let window = AppWindowId::default();
        let mut app = App::new();
        let mut ui: UiTree<App> = UiTree::new();
        ui.set_window(window);

        app.with_global_mut(WindowFrameClockService::default, |svc, _app| {
            svc.set_fixed_delta(window, Some(Duration::from_millis(16)));
        });

        let image = app.models_mut().insert(None::<ImageId>);

        let mut services = FakeServices::default();
        let bounds = Rect::new(
            Point::new(Px(0.0), Px(0.0)),
            fret_core::Size::new(Px(200.0), Px(120.0)),
        );

        // Let the delay elapse so fallback becomes visible.
        let delay_ms = 32;
        for frame in 1..=3 {
            app.set_frame_id(FrameId(frame));
            render_frame(
                &mut ui,
                &mut app,
                &mut services,
                window,
                bounds,
                image.clone(),
                delay_ms,
            );
            ui.request_semantics_snapshot();
            ui.layout_all(&mut app, &mut services, bounds, 1.0);
        }

        ui.request_semantics_snapshot();
        let snap = ui.semantics_snapshot().expect("semantics snapshot");
        assert!(snapshot_contains_label(&snap, "JD"));

        // Image becomes available -> fallback hides immediately (Radix `status === loaded`).
        let _ = app
            .models_mut()
            .update(&image, |v| *v = Some(ImageId::default()));
        app.set_frame_id(FrameId(4));
        render_frame(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            image.clone(),
            delay_ms,
        );
        ui.request_semantics_snapshot();
        ui.layout_all(&mut app, &mut services, bounds, 1.0);

        ui.request_semantics_snapshot();
        let snap = ui.semantics_snapshot().expect("semantics snapshot");
        assert!(!snapshot_contains_label(&snap, "JD"));
    }

    #[test]
    fn avatar_root_clips_overflow_like_shadcn() {
        let window = AppWindowId::default();
        let mut app = App::new();
        let bounds = Rect::new(
            Point::new(Px(0.0), Px(0.0)),
            fret_core::Size::new(Px(200.0), Px(120.0)),
        );

        let el = fret_ui::elements::with_element_cx(&mut app, window, bounds, "test", |cx| {
            Avatar::new([AvatarFallback::new("JD").into_element(cx)]).into_element(cx)
        });

        let ElementKind::Container(props) = &el.kind else {
            panic!("expected Avatar root to be a Container, got {:?}", el.kind);
        };
        assert_eq!(props.layout.overflow, Overflow::Clip);
    }

    #[test]
    fn avatar_with_badge_uses_wrapper_to_avoid_clipping_badge() {
        let window = AppWindowId::default();
        let mut app = App::new();
        let bounds = Rect::new(
            Point::new(Px(0.0), Px(0.0)),
            fret_core::Size::new(Px(200.0), Px(120.0)),
        );

        let el = fret_ui::elements::with_element_cx(&mut app, window, bounds, "test", |cx| {
            let badge = AvatarBadge::new().into_element(cx);
            Avatar::new([AvatarFallback::new("JD").into_element(cx), badge]).into_element(cx)
        });

        let ElementKind::Container(wrapper) = &el.kind else {
            panic!(
                "expected Avatar+Badge root to be a wrapper Container, got {:?}",
                el.kind
            );
        };
        assert_eq!(
            wrapper.layout.overflow,
            Overflow::Visible,
            "expected wrapper overflow to remain visible"
        );

        let core = el.children.first().expect("wrapper core child");
        let ElementKind::Container(core_props) = &core.kind else {
            panic!(
                "expected wrapper first child (core) to be a Container, got {:?}",
                core.kind
            );
        };
        assert_eq!(
            core_props.layout.overflow,
            Overflow::Clip,
            "expected avatar core to clip overflow"
        );
    }

    #[test]
    fn avatar_image_uses_fill_layout_when_image_is_ready() {
        let window = AppWindowId::default();
        let mut app = App::new();
        let mut ui: UiTree<App> = UiTree::new();
        ui.set_window(window);

        let image = app.models_mut().insert(Some(ImageId::default()));
        let mut services = FakeServices::default();
        let bounds = Rect::new(
            Point::new(Px(0.0), Px(0.0)),
            fret_core::Size::new(Px(200.0), Px(120.0)),
        );

        app.set_frame_id(FrameId(1));
        render_frame(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            image.clone(),
            0,
        );
        ui.layout_all(&mut app, &mut services, bounds, 1.0);

        let root = ui.base_root().expect("root");
        let avatar = ui
            .children(root)
            .first()
            .copied()
            .expect("expected avatar element");
        let avatar_bounds = ui.debug_node_bounds(avatar).expect("avatar bounds");
        assert!(
            avatar_bounds.size.width.0 > 1.0 && avatar_bounds.size.height.0 > 1.0,
            "expected avatar to have non-zero bounds, got {avatar_bounds:?}"
        );

        let image_gate = ui
            .children(avatar)
            .first()
            .copied()
            .expect("expected image gate element");
        let gate_bounds = ui.debug_node_bounds(image_gate).expect("image gate bounds");
        assert!(
            gate_bounds.size.width.0 > 1.0 && gate_bounds.size.height.0 > 1.0,
            "expected image gate to have non-zero bounds, got {gate_bounds:?}"
        );

        let image_wrapper = ui
            .children(image_gate)
            .first()
            .copied()
            .expect("expected image wrapper element");
        let wrapper_bounds = ui
            .debug_node_bounds(image_wrapper)
            .expect("image wrapper bounds");
        assert!(
            wrapper_bounds.size.width.0 > 1.0 && wrapper_bounds.size.height.0 > 1.0,
            "expected image wrapper to have non-zero bounds, got {wrapper_bounds:?}"
        );

        let image_node = ui
            .children(image_wrapper)
            .first()
            .copied()
            .expect("expected image node");
        let image_bounds = ui.debug_node_bounds(image_node).expect("image bounds");
        assert!(
            image_bounds.size.width.0 > 1.0 && image_bounds.size.height.0 > 1.0,
            "expected image to have non-zero bounds, got {image_bounds:?}"
        );
    }

    #[test]
    fn avatar_image_emits_cover_fit_scene_op() {
        let window = AppWindowId::default();
        let mut app = App::new();
        let mut ui: UiTree<App> = UiTree::new();
        ui.set_window(window);

        let image = app.models_mut().insert(Some(ImageId::default()));
        let mut services = FakeServices::default();
        let bounds = Rect::new(
            Point::new(Px(0.0), Px(0.0)),
            fret_core::Size::new(Px(200.0), Px(120.0)),
        );

        app.set_frame_id(FrameId(1));
        render_frame(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            image.clone(),
            0,
        );
        ui.layout_all(&mut app, &mut services, bounds, 1.0);

        let mut scene = fret_core::Scene::default();
        ui.paint_all(&mut app, &mut services, bounds, &mut scene, 1.0);

        assert!(
            scene
                .ops()
                .iter()
                .any(|op| matches!(op, fret_core::SceneOp::Image { fit, .. } if *fit == ViewportFit::Cover)),
            "expected AvatarImage to emit SceneOp::Image with ViewportFit::Cover"
        );
    }

    #[test]
    fn avatar_fallback_delay_requests_animation_frames_while_pending() {
        let window = AppWindowId::default();
        let mut app = App::new();
        let mut ui: UiTree<App> = UiTree::new();
        ui.set_window(window);

        let image = app.models_mut().insert(None::<ImageId>);

        let mut services = FakeServices::default();
        let bounds = Rect::new(
            Point::new(Px(0.0), Px(0.0)),
            fret_core::Size::new(Px(200.0), Px(120.0)),
        );

        app.set_frame_id(FrameId(1));
        render_frame(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            image.clone(),
            10,
        );

        let effects = app.flush_effects();
        assert!(
            effects
                .iter()
                .any(|e| matches!(e, Effect::RequestAnimationFrame(w) if *w == window)),
            "expected RequestAnimationFrame while avatar fallback delay is pending"
        );

        let _ = app
            .models_mut()
            .update(&image, |v| *v = Some(ImageId::default()));
        app.set_frame_id(FrameId(2));
        render_frame(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            image.clone(),
            10,
        );

        let effects = app.flush_effects();
        assert!(
            !effects
                .iter()
                .any(|e| matches!(e, Effect::RequestAnimationFrame(w) if *w == window)),
            "did not expect RequestAnimationFrame after avatar image becomes available"
        );
    }
}
