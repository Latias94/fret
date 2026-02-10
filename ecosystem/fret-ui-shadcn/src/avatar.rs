use std::sync::Arc;

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
use fret_ui_kit::{ChromeRefinement, ColorRef, LayoutRefinement, MetricRef, Radius, Space, ui};

/// shadcn/ui `Avatar` root.
///
/// This is a fixed-size, overflow-clipped, fully-rounded container intended to host exactly one
/// `AvatarImage` and one `AvatarFallback` (order controls paint stacking).
#[derive(Debug, Clone)]
pub struct Avatar {
    children: Vec<AnyElement>,
    chrome: ChromeRefinement,
    layout: LayoutRefinement,
}

impl Avatar {
    pub fn new(children: impl IntoIterator<Item = AnyElement>) -> Self {
        let children = children.into_iter().collect();
        Self {
            children,
            chrome: ChromeRefinement::default(),
            layout: LayoutRefinement::default(),
        }
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

        let base_chrome = ChromeRefinement::default().rounded(Radius::Full);
        let base_layout = LayoutRefinement::default()
            .relative()
            .w_px(MetricRef::space(Space::N8))
            .h_px(MetricRef::space(Space::N8))
            .flex_shrink_0();

        let mut props = decl_style::container_props(
            &theme,
            base_chrome.merge(self.chrome),
            base_layout.merge(self.layout),
        );
        props.layout.overflow = Overflow::Clip;

        let children = self.children;
        cx.container(props, move |_cx| children)
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
                let theme = Theme::global(&*cx.app).clone();
                let layout = LayoutRefinement::default()
                    .absolute()
                    .inset(Space::N0)
                    .size_full()
                    .aspect_ratio(1.0)
                    .merge(self.layout);

                let wrapper = decl_style::container_props(&theme, self.chrome, layout);
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
                        image,
                        fit: ViewportFit::Cover,
                        opacity,
                        uv: None,
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
    delay_frames: Option<u64>,
    chrome: ChromeRefinement,
    layout: LayoutRefinement,
}

impl AvatarFallback {
    pub fn new(text: impl Into<Arc<str>>) -> Self {
        Self {
            text: text.into(),
            show_when_image_missing: None,
            delay_frames: None,
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
        self.delay_frames = Some(frames);
        self
    }

    /// Delay rendering fallback by a best-effort millisecond value (converted to frames).
    pub fn delay_ms(mut self, ms: u64) -> Self {
        let frames = ms.saturating_mul(60).saturating_add(999) / 1000;
        self.delay_frames = Some(frames);
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
        cx.scope(|cx| {
            let id = cx.root_id();
            let theme = Theme::global(&*cx.app).clone();

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

            let now_frame = cx.app.frame_id().0;
            let delay_ready =
                cx.with_state_for(id, radix_avatar::AvatarFallbackDelay::default, |gate| {
                    gate.drive(now_frame, self.delay_frames, want_render)
                });
            scheduling::set_continuous_frames(
                cx,
                want_render && self.delay_frames.is_some() && !delay_ready,
            );

            let present = if self.show_when_image_missing.is_some() {
                radix_avatar::fallback_visible(status, delay_ready)
            } else {
                delay_ready
            };

            let bg = theme.color_required("muted");
            let fg = theme.color_required("muted-foreground");

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
                .unwrap_or_else(|| theme.metric_required("font.size"));
            let line_height = theme
                .metric_by_key("component.avatar.fallback_line_height")
                .or_else(|| theme.metric_by_key("font.line_height"))
                .unwrap_or_else(|| theme.metric_required("font.line_height"));

            let label = ui::label(cx, self.text)
                .text_size_px(text_px)
                .line_height_px(line_height)
                .font_medium()
                .text_color(ColorRef::Color(fg))
                .nowrap()
                .into_element(cx);

            let flex_layout =
                decl_style::layout_style(&theme, LayoutRefinement::default().size_full());
            let flex = cx.flex(
                FlexProps {
                    layout: flex_layout,
                    direction: fret_core::Axis::Horizontal,
                    gap: Px(0.0),
                    padding: fret_core::Edges::all(Px(0.0)),
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

    use fret_app::App;
    use fret_core::{
        AppWindowId, PathCommand, PathConstraints, PathId, PathMetrics, PathService, PathStyle,
        Point, Px, Rect, SemanticsRole, Size as CoreSize, SvgId, SvgService, TextBlobId,
        TextConstraints, TextMetrics, TextService,
    };
    use fret_runtime::{Effect, FrameId};
    use fret_ui::tree::UiTree;

    #[derive(Default)]
    struct FakeServices;

    impl fret_core::MaterialService for FakeServices {
        fn register_material(
            &mut self,
            _desc: fret_core::MaterialDescriptor,
        ) -> Result<fret_core::MaterialId, fret_core::MaterialRegistrationError> {
            Err(fret_core::MaterialRegistrationError::Unsupported)
        }

        fn unregister_material(&mut self, _id: fret_core::MaterialId) -> bool {
            true
        }
    }

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

    fn render_frame(
        ui: &mut UiTree<App>,
        app: &mut App,
        services: &mut dyn fret_core::UiServices,
        window: AppWindowId,
        bounds: Rect,
        image: Model<Option<ImageId>>,
        delay_frames: u64,
    ) {
        let root =
            fret_ui::declarative::render_root(ui, app, services, window, bounds, "test", |cx| {
                let image_el = AvatarImage::model(image.clone()).into_element(cx);
                let fallback_el = AvatarFallback::new("JD")
                    .when_image_missing_model(image.clone())
                    .delay_frames(delay_frames)
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
    fn avatar_fallback_delay_gates_text_until_elapsed() {
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

        for frame in 1..=3 {
            app.set_frame_id(FrameId(frame));
            render_frame(
                &mut ui,
                &mut app,
                &mut services,
                window,
                bounds,
                image.clone(),
                2,
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

        let image = app.models_mut().insert(None::<ImageId>);

        let mut services = FakeServices::default();
        let bounds = Rect::new(
            Point::new(Px(0.0), Px(0.0)),
            fret_core::Size::new(Px(200.0), Px(120.0)),
        );

        // Let the delay elapse so fallback becomes visible.
        for frame in 1..=3 {
            app.set_frame_id(FrameId(frame));
            render_frame(
                &mut ui,
                &mut app,
                &mut services,
                window,
                bounds,
                image.clone(),
                2,
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
            2,
        );
        ui.request_semantics_snapshot();
        ui.layout_all(&mut app, &mut services, bounds, 1.0);

        ui.request_semantics_snapshot();
        let snap = ui.semantics_snapshot().expect("semantics snapshot");
        assert!(!snapshot_contains_label(&snap, "JD"));
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
