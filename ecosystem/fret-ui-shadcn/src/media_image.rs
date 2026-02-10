use fret_core::{ImageId, ViewportFit};
use fret_runtime::Model;
use fret_ui::element::{AnyElement, ImageProps, LayoutStyle, Length, Overflow, SizeStyle};
use fret_ui::{ElementContext, Theme, UiHost};
use fret_ui_kit::declarative::global_watch::GlobalWatchExt as _;
use fret_ui_kit::declarative::model_watch::ModelWatchExt as _;
use fret_ui_kit::declarative::style as decl_style;
use fret_ui_kit::primitives::aspect_ratio::AspectRatio;
use fret_ui_kit::{ChromeRefinement, ImageMetadataStore, LayoutRefinement, Space};

use crate::Skeleton;

/// A small shadcn-style image recipe for cards/media rows.
///
/// This stays policy-owned (ecosystem layer) and only depends on the stable mechanism contract:
/// `SceneOp::Image { fit: ViewportFit, .. }` (ADR 1170).
#[derive(Debug, Clone)]
pub struct MediaImage {
    source: MediaImageSource,
    fit: ViewportFit,
    opacity: f32,
    loading: bool,
    intrinsic_aspect_ratio_from_metadata: bool,
    loading_slot: Option<AnyElement>,
    fallback_slot: Option<AnyElement>,
    chrome: ChromeRefinement,
    layout: LayoutRefinement,
}

#[derive(Debug, Clone)]
enum MediaImageSource {
    Ready(ImageId),
    Optional(Option<ImageId>),
    Model(Model<Option<ImageId>>),
}

impl MediaImageSource {
    fn resolve<H: UiHost>(&self, cx: &mut ElementContext<'_, H>) -> Option<ImageId> {
        match self {
            Self::Ready(image) => Some(*image),
            Self::Optional(image) => *image,
            Self::Model(model) => cx.watch_model(model).copied().flatten(),
        }
    }
}

impl MediaImage {
    pub fn new(image: ImageId) -> Self {
        Self {
            source: MediaImageSource::Ready(image),
            fit: ViewportFit::Cover,
            opacity: 1.0,
            loading: false,
            intrinsic_aspect_ratio_from_metadata: false,
            loading_slot: None,
            fallback_slot: None,
            chrome: ChromeRefinement::default(),
            layout: LayoutRefinement::default(),
        }
    }

    pub fn maybe(image: Option<ImageId>) -> Self {
        Self {
            source: MediaImageSource::Optional(image),
            ..Self::new(ImageId::default())
        }
    }

    pub fn model(image: Model<Option<ImageId>>) -> Self {
        Self {
            source: MediaImageSource::Model(image),
            ..Self::new(ImageId::default())
        }
    }

    pub fn fit(mut self, fit: ViewportFit) -> Self {
        self.fit = fit;
        self
    }

    pub fn opacity(mut self, opacity: f32) -> Self {
        self.opacity = opacity;
        self
    }

    /// When `true` and no image is present, renders the loading slot (or a default `Skeleton`).
    ///
    /// This is intentionally separate from `Option<ImageId>` because `None` may mean either:
    /// - loading in progress, or
    /// - image missing (fallback).
    pub fn loading(mut self, loading: bool) -> Self {
        self.loading = loading;
        self
    }

    /// When enabled and metadata is available, wraps the image in an `AspectRatio` container
    /// derived from the app-owned `ImageMetadataStore` global.
    ///
    /// This is intentionally opt-in: intrinsic metadata must not implicitly affect layout
    /// (ADR 0126).
    pub fn intrinsic_aspect_ratio_from_metadata(mut self, enabled: bool) -> Self {
        self.intrinsic_aspect_ratio_from_metadata = enabled;
        self
    }

    pub fn loading_slot(mut self, slot: AnyElement) -> Self {
        self.loading_slot = Some(slot);
        self
    }

    pub fn fallback_slot(mut self, slot: AnyElement) -> Self {
        self.fallback_slot = Some(slot);
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
        let image = self.source.resolve(cx);

        let chrome = ChromeRefinement::default().merge(self.chrome);
        let layout = self.layout;

        let intrinsic_ratio =
            if self.intrinsic_aspect_ratio_from_metadata && layout.aspect_ratio.is_none() {
                image
                    .and_then(|image| {
                        cx.watch_global::<ImageMetadataStore>()
                            .layout()
                            .map(|store| store.aspect_ratio(image))
                            .flatten()
                    })
                    .filter(|ratio| ratio.is_finite() && *ratio > 0.0)
            } else {
                None
            };

        let content_host = decl_style::container_props(
            &theme,
            ChromeRefinement::default(),
            LayoutRefinement::default()
                .absolute()
                .inset(Space::N0)
                .size_full(),
        );

        let fit = self.fit;
        let opacity = self.opacity.clamp(0.0, 1.0);
        let loading = self.loading;
        let loading_slot = self.loading_slot;
        let fallback_slot = self.fallback_slot;

        if let Some(ratio) = intrinsic_ratio {
            let inner = {
                let base_layout = LayoutRefinement::default().relative().size_full();
                let inner =
                    decl_style::container_props(&theme, ChromeRefinement::default(), base_layout);
                cx.container(inner, move |cx| {
                    vec![cx.container(content_host, move |cx| {
                        if let Some(image) = image {
                            let mut image_layout = LayoutStyle::default();
                            image_layout.size = SizeStyle {
                                width: Length::Fill,
                                height: Length::Fill,
                                ..Default::default()
                            };
                            vec![cx.image_props(ImageProps {
                                layout: image_layout,
                                image,
                                fit,
                                opacity,
                                uv: None,
                            })]
                        } else if loading {
                            if let Some(slot) = loading_slot {
                                vec![slot]
                            } else {
                                vec![
                                    Skeleton::new()
                                        .refine_layout(
                                            LayoutRefinement::default()
                                                .absolute()
                                                .inset(Space::N0)
                                                .size_full(),
                                        )
                                        .into_element(cx),
                                ]
                            }
                        } else if let Some(slot) = fallback_slot {
                            vec![slot]
                        } else {
                            Vec::new()
                        }
                    })]
                })
            };

            AspectRatio::new(ratio, inner)
                .refine_style(chrome)
                .refine_layout(layout)
                .into_element(cx)
        } else {
            let base_layout = LayoutRefinement::default().relative().size_full();
            let mut wrapper =
                decl_style::container_props(&theme, chrome, base_layout.merge(layout));
            wrapper.layout.overflow = Overflow::Clip;

            cx.container(wrapper, move |cx| {
                vec![cx.container(content_host, move |cx| {
                    if let Some(image) = image {
                        let mut image_layout = LayoutStyle::default();
                        image_layout.size = SizeStyle {
                            width: Length::Fill,
                            height: Length::Fill,
                            ..Default::default()
                        };
                        vec![cx.image_props(ImageProps {
                            layout: image_layout,
                            image,
                            fit,
                            opacity,
                            uv: None,
                        })]
                    } else if loading {
                        if let Some(slot) = loading_slot {
                            vec![slot]
                        } else {
                            vec![
                                Skeleton::new()
                                    .refine_layout(
                                        LayoutRefinement::default()
                                            .absolute()
                                            .inset(Space::N0)
                                            .size_full(),
                                    )
                                    .into_element(cx),
                            ]
                        }
                    } else if let Some(slot) = fallback_slot {
                        vec![slot]
                    } else {
                        Vec::new()
                    }
                })]
            })
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use fret_app::App;
    use fret_core::{
        AppWindowId, PathCommand, PathConstraints, PathId, PathMetrics, PathService, PathStyle,
        Point, Px, Rect, Scene, SceneOp, TextBlobId, TextConstraints, TextMetrics, TextService,
    };
    use fret_ui::element::ElementKind;
    use fret_ui::tree::UiTree;
    use fret_ui_kit::with_image_metadata_store_mut;

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
                    size: fret_core::Size::new(Px(10.0), Px(10.0)),
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

    impl fret_core::SvgService for FakeServices {
        fn register_svg(&mut self, _bytes: &[u8]) -> fret_core::SvgId {
            fret_core::SvgId::default()
        }

        fn unregister_svg(&mut self, _svg: fret_core::SvgId) -> bool {
            true
        }
    }

    #[test]
    fn media_image_emits_fit_scene_op_when_image_present() {
        let window = AppWindowId::default();
        let mut app = App::new();
        let mut ui: UiTree<App> = UiTree::new();
        ui.set_window(window);

        let bounds = Rect::new(
            Point::new(Px(0.0), Px(0.0)),
            fret_core::Size::new(Px(200.0), Px(120.0)),
        );

        let img = ImageId::default();
        let mut services = FakeServices::default();
        let root = fret_ui::declarative::render_root(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            "test",
            |cx| {
                vec![
                    MediaImage::new(img)
                        .fit(ViewportFit::Contain)
                        .into_element(cx),
                ]
            },
        );
        ui.set_root(root);
        ui.layout_all(&mut app, &mut services, bounds, 1.0);

        let mut scene = Scene::default();
        ui.paint_all(&mut app, &mut services, bounds, &mut scene, 1.0);

        assert!(
            scene.ops().iter().any(|op| matches!(op, SceneOp::Image { image, fit, .. } if *image == img && *fit == ViewportFit::Contain)),
            "expected SceneOp::Image with fit=Contain"
        );
    }

    #[test]
    fn media_image_can_use_intrinsic_aspect_ratio_from_metadata() {
        let window = AppWindowId::default();
        let mut app = App::new();

        let bounds = Rect::new(
            Point::new(Px(0.0), Px(0.0)),
            fret_core::Size::new(Px(200.0), Px(120.0)),
        );

        let img = ImageId::default();
        with_image_metadata_store_mut(&mut app, |store| {
            store.set_intrinsic_size_px(img, (1920, 1080));
        });

        fret_ui::elements::with_element_cx(&mut app, window, bounds, "test", |cx| {
            let el = MediaImage::new(img)
                .intrinsic_aspect_ratio_from_metadata(true)
                .into_element(cx);
            let ElementKind::Container(props) = &el.kind else {
                panic!("expected a container element");
            };
            assert_eq!(props.layout.aspect_ratio, Some(1920.0 / 1080.0));
        });
    }

    #[test]
    fn media_image_does_not_invent_aspect_ratio_when_metadata_missing() {
        let window = AppWindowId::default();
        let mut app = App::new();

        let bounds = Rect::new(
            Point::new(Px(0.0), Px(0.0)),
            fret_core::Size::new(Px(200.0), Px(120.0)),
        );

        let img = ImageId::default();
        fret_ui::elements::with_element_cx(&mut app, window, bounds, "test", |cx| {
            let el = MediaImage::new(img)
                .intrinsic_aspect_ratio_from_metadata(true)
                .into_element(cx);
            let ElementKind::Container(props) = &el.kind else {
                panic!("expected a container element");
            };
            assert_eq!(props.layout.aspect_ratio, None);
        });
    }

    #[test]
    fn media_image_does_not_emit_image_when_missing_and_not_loading() {
        let window = AppWindowId::default();
        let mut app = App::new();
        let mut ui: UiTree<App> = UiTree::new();
        ui.set_window(window);

        let bounds = Rect::new(
            Point::new(Px(0.0), Px(0.0)),
            fret_core::Size::new(Px(200.0), Px(120.0)),
        );

        let mut services = FakeServices::default();
        let root = fret_ui::declarative::render_root(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            "test",
            |cx| {
                vec![
                    MediaImage::maybe(None)
                        .fallback_slot(cx.text("fallback"))
                        .into_element(cx),
                ]
            },
        );
        ui.set_root(root);
        ui.layout_all(&mut app, &mut services, bounds, 1.0);

        let mut scene = Scene::default();
        ui.paint_all(&mut app, &mut services, bounds, &mut scene, 1.0);

        assert!(
            !scene
                .ops()
                .iter()
                .any(|op| matches!(op, SceneOp::Image { .. })),
            "expected no SceneOp::Image when source is missing"
        );
    }
}
