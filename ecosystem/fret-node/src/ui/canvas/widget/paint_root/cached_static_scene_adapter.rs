//! Paint-root cached static scene adapter contract.
//!
//! This module keeps cached group/node static replay route inputs behind a named seam. Concrete
//! retained host, service, scale, and scene sinks live next to the retained paint root binding.

use fret_core::{Scene, UiServices};
use fret_ui::UiHost;

pub(super) trait PaintRootCachedStaticSceneCx<H: UiHost> {
    fn paint_root_cached_static_host(&self) -> &H;
    fn paint_root_cached_static_services(&mut self) -> &mut dyn UiServices;
    fn paint_root_cached_static_scale_factor(&self) -> f32;
    fn paint_root_cached_static_scene(&mut self) -> &mut Scene;
}

pub(super) fn paint_root_cached_static_host<H>(cx: &impl PaintRootCachedStaticSceneCx<H>) -> &H
where
    H: UiHost,
{
    cx.paint_root_cached_static_host()
}

pub(super) fn paint_root_cached_static_services<H>(
    cx: &mut impl PaintRootCachedStaticSceneCx<H>,
) -> &mut dyn UiServices
where
    H: UiHost,
{
    cx.paint_root_cached_static_services()
}

pub(super) fn paint_root_cached_static_scale_factor<H>(
    cx: &impl PaintRootCachedStaticSceneCx<H>,
) -> f32
where
    H: UiHost,
{
    cx.paint_root_cached_static_scale_factor()
}

pub(super) fn paint_root_cached_static_scene<H>(
    cx: &mut impl PaintRootCachedStaticSceneCx<H>,
) -> &mut Scene
where
    H: UiHost,
{
    cx.paint_root_cached_static_scene()
}
