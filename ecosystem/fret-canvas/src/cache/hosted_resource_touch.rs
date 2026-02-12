use crate::text::TextCache;
use fret_core::SceneOp;

use super::path_cache::PathCache;
use super::svg_cache::SvgCache;

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct HostedResourceTouchCounts {
    pub text_blobs: u32,
    pub paths: u32,
    pub svgs: u32,
}

/// Aggregates "touch-on-replay" behavior across multiple retained caches.
///
/// Replay caches (`SceneOpCache` / `SceneOpTileCache`) store only recorded ops and do not manage
/// lifetimes of hosted resources referenced by those ops. Use this helper in cache-hit paths to
/// prevent cached `TextBlobId`/`PathId`/`SvgId` resources from being pruned while still referenced
/// by replayed ops.
pub trait TouchHostedResourcesInSceneOps {
    fn touch_hosted_resources_in_scene_ops(&mut self, ops: &[SceneOp])
    -> HostedResourceTouchCounts;
}

impl TouchHostedResourcesInSceneOps for (&mut TextCache, &mut PathCache) {
    fn touch_hosted_resources_in_scene_ops(
        &mut self,
        ops: &[SceneOp],
    ) -> HostedResourceTouchCounts {
        HostedResourceTouchCounts {
            text_blobs: self.0.touch_blobs_in_scene_ops(ops),
            paths: self.1.touch_paths_in_scene_ops(ops),
            svgs: 0,
        }
    }
}

impl TouchHostedResourcesInSceneOps for (&mut TextCache, &mut SvgCache) {
    fn touch_hosted_resources_in_scene_ops(
        &mut self,
        ops: &[SceneOp],
    ) -> HostedResourceTouchCounts {
        HostedResourceTouchCounts {
            text_blobs: self.0.touch_blobs_in_scene_ops(ops),
            paths: 0,
            svgs: self.1.touch_svgs_in_scene_ops(ops),
        }
    }
}

impl TouchHostedResourcesInSceneOps for (&mut PathCache, &mut SvgCache) {
    fn touch_hosted_resources_in_scene_ops(
        &mut self,
        ops: &[SceneOp],
    ) -> HostedResourceTouchCounts {
        HostedResourceTouchCounts {
            text_blobs: 0,
            paths: self.0.touch_paths_in_scene_ops(ops),
            svgs: self.1.touch_svgs_in_scene_ops(ops),
        }
    }
}

impl TouchHostedResourcesInSceneOps for (&mut TextCache, &mut PathCache, &mut SvgCache) {
    fn touch_hosted_resources_in_scene_ops(
        &mut self,
        ops: &[SceneOp],
    ) -> HostedResourceTouchCounts {
        HostedResourceTouchCounts {
            text_blobs: self.0.touch_blobs_in_scene_ops(ops),
            paths: self.1.touch_paths_in_scene_ops(ops),
            svgs: self.2.touch_svgs_in_scene_ops(ops),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cache::SvgBytes;
    use fret_core::{
        Color, DrawOrder, Edges, PathCommand, PathConstraints, PathId, PathMetrics, PathService,
        PathStyle, Point, Px, Rect, Size, StrokeStyle, SvgFit, SvgId, SvgService, TextBlobId,
        TextConstraints, TextInput, TextMetrics, TextOverflow, TextService, TextStyle, TextWrap,
    };

    #[derive(Default)]
    struct Services;

    impl TextService for Services {
        fn prepare(
            &mut self,
            _input: &TextInput,
            _constraints: TextConstraints,
        ) -> (TextBlobId, TextMetrics) {
            (
                TextBlobId::default(),
                TextMetrics {
                    size: Size::new(Px(0.0), Px(0.0)),
                    baseline: Px(0.0),
                },
            )
        }

        fn release(&mut self, _blob: TextBlobId) {}
    }

    impl PathService for Services {
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

    impl SvgService for Services {
        fn register_svg(&mut self, _bytes: &[u8]) -> SvgId {
            SvgId::default()
        }

        fn unregister_svg(&mut self, _svg: SvgId) -> bool {
            true
        }
    }

    impl fret_core::MaterialService for Services {
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

    #[test]
    fn touch_hosted_resources_in_scene_ops_aggregates_counts() {
        let mut services = Services::default();

        let mut text = TextCache::default();
        let mut paths = PathCache::default();
        let mut svgs = SvgCache::default();

        text.begin_frame();
        paths.begin_frame();
        svgs.begin_frame();

        let prepared = text.prepare(
            &mut services,
            "hello",
            &TextStyle::default(),
            TextConstraints {
                max_width: None,
                wrap: TextWrap::None,
                overflow: TextOverflow::Clip,
                scale_factor: 1.0,
            },
        );

        let (path, _metrics) = paths.prepare(
            &mut services,
            123,
            &[PathCommand::MoveTo(Point::new(Px(0.0), Px(0.0)))],
            PathStyle::Stroke(StrokeStyle::default()),
            PathConstraints { scale_factor: 1.0 },
        );

        let svg = svgs.prepare(&mut services, 9, SvgBytes::Static(b"<svg/>"));

        let ops = vec![
            SceneOp::Text {
                order: DrawOrder(0),
                origin: Point::new(Px(0.0), Px(0.0)),
                text: prepared.blob,
                color: Color::TRANSPARENT,
            },
            SceneOp::Path {
                order: DrawOrder(0),
                origin: Point::new(Px(0.0), Px(0.0)),
                path,
                color: Color::TRANSPARENT,
            },
            SceneOp::SvgImage {
                order: DrawOrder(0),
                rect: Rect::new(Point::new(Px(0.0), Px(0.0)), Size::new(Px(1.0), Px(1.0))),
                svg,
                fit: SvgFit::Contain,
                opacity: 1.0,
            },
            SceneOp::Quad {
                order: DrawOrder(0),
                rect: Rect::new(Point::new(Px(0.0), Px(0.0)), Size::new(Px(1.0), Px(1.0))),
                background: fret_core::Paint::Solid(Color::TRANSPARENT),
                border: Edges::all(Px(0.0)),
                border_paint: fret_core::Paint::Solid(Color::TRANSPARENT),
                corner_radii: fret_core::Corners::all(Px(0.0)),
            },
        ];

        let counts = (&mut text, &mut paths, &mut svgs).touch_hosted_resources_in_scene_ops(&ops);
        assert_eq!(
            counts,
            HostedResourceTouchCounts {
                text_blobs: 1,
                paths: 1,
                svgs: 1
            }
        );
    }
}
