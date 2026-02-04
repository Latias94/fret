use std::sync::Arc;

use fret_core::{Color, Px};
use fret_ui::element::AnyElement;
use fret_ui::element::ContainerProps;
use fret_ui::element::HoverRegionProps;
use fret_ui::element::InsetStyle;
use fret_ui::element::LayoutStyle;
use fret_ui::element::Length;
use fret_ui::element::Overflow;
use fret_ui::element::PositionStyle;
use fret_ui::element::ScrollAxis;
use fret_ui::element::ScrollIntrinsicMeasureMode;
use fret_ui::element::ScrollProps;
use fret_ui::element::ScrollbarAxis;
use fret_ui::element::ScrollbarProps;
use fret_ui::element::ScrollbarStyle;
use fret_ui::element::SemanticsProps;
use fret_ui::element::SizeStyle;
use fret_ui::element::StackProps;
use fret_ui::scroll::ScrollHandle;
use fret_ui::{ElementContext, Theme, UiHost};
use fret_ui_kit::LayoutRefinement;
use fret_ui_kit::declarative::style as decl_style;
use fret_ui_kit::primitives::scroll_area::DEFAULT_SCROLL_HIDE_DELAY_TICKS;
use fret_ui_kit::primitives::scroll_area::ScrollAreaType;

fn shadcn_scrollbar_thumb(theme: &Theme) -> Color {
    theme.color_required("border")
}

fn shadcn_scrollbar_thumb_hover(theme: &Theme) -> Color {
    theme.color_required("border")
}

fn shadcn_scrollbar_corner_bg(theme: &Theme) -> Color {
    theme.color_by_key("border").unwrap_or(Color::TRANSPARENT)
}

/// shadcn/ui `ScrollArea` primitives (v4).
///
/// Upstream (`new-york-v4`) composes:
/// - `ScrollArea.Root` (relative container)
/// - `ScrollArea.Viewport` (scrollable viewport)
/// - `ScrollArea.Scrollbar` + `ScrollArea.Thumb`
/// - `ScrollArea.Corner`
///
/// In Fret, scrollbars are explicit runtime primitives (`Scroll` + `Scrollbar`). This module
/// exposes a composable, Radix-shaped surface while keeping the existing compact builder API.
#[derive(Debug, Clone)]
pub struct ScrollAreaViewport {
    children: Vec<AnyElement>,
    axis: ScrollAxis,
    probe_unbounded: bool,
    viewport_test_id: Option<Arc<str>>,
    intrinsic_measure_mode: ScrollIntrinsicMeasureMode,
}

impl ScrollAreaViewport {
    pub fn new(children: impl IntoIterator<Item = AnyElement>) -> Self {
        let children = children.into_iter().collect();
        Self {
            children,
            axis: ScrollAxis::Y,
            probe_unbounded: true,
            viewport_test_id: None,
            intrinsic_measure_mode: ScrollIntrinsicMeasureMode::Content,
        }
    }

    pub fn axis(mut self, axis: ScrollAxis) -> Self {
        self.axis = axis;
        self
    }

    pub fn probe_unbounded(mut self, probe_unbounded: bool) -> Self {
        self.probe_unbounded = probe_unbounded;
        self
    }

    pub fn viewport_test_id(mut self, test_id: impl Into<Arc<str>>) -> Self {
        self.viewport_test_id = Some(test_id.into());
        self
    }

    pub fn intrinsic_measure_mode(mut self, mode: ScrollIntrinsicMeasureMode) -> Self {
        self.intrinsic_measure_mode = mode;
        self
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ScrollAreaScrollbarOrientation {
    Vertical,
    Horizontal,
}

impl Default for ScrollAreaScrollbarOrientation {
    fn default() -> Self {
        Self::Vertical
    }
}

/// shadcn/ui `ScrollBar` / Radix `ScrollAreaScrollbar` (v4).
#[derive(Debug, Clone)]
pub struct ScrollAreaScrollbar {
    orientation: ScrollAreaScrollbarOrientation,
    track_padding: Px,
    thumb_idle_alpha: f32,
}

impl Default for ScrollAreaScrollbar {
    fn default() -> Self {
        Self {
            orientation: ScrollAreaScrollbarOrientation::default(),
            track_padding: ScrollbarStyle::default().track_padding,
            // shadcn/ui v4 `ScrollAreaThumb` uses `bg-border` directly (no opacity modifier).
            // Keep the default thumb alpha at 1.0 for 1:1 web conformance.
            thumb_idle_alpha: 1.0,
        }
    }
}

impl ScrollAreaScrollbar {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn orientation(mut self, orientation: ScrollAreaScrollbarOrientation) -> Self {
        self.orientation = orientation;
        self
    }

    /// Mirrors the upstream wrapper's `p-px` track padding.
    pub fn track_padding(mut self, padding: Px) -> Self {
        self.track_padding = padding;
        self
    }

    pub fn thumb_idle_alpha(mut self, alpha: f32) -> Self {
        self.thumb_idle_alpha = alpha;
        self
    }
}

/// shadcn/ui `ScrollArea.Corner` (v4).
#[derive(Debug, Clone, Default)]
pub struct ScrollAreaCorner;

/// A composable, Radix/shadcn-shaped scroll-area surface (`Root` / `Viewport` / `Scrollbar` /
/// `Corner`).
#[derive(Clone)]
pub struct ScrollAreaRoot {
    viewport: ScrollAreaViewport,
    scrollbars: Vec<ScrollAreaScrollbar>,
    corner: bool,
    scrollbar_type: ScrollAreaType,
    scroll_hide_delay_ticks: u64,
    layout: LayoutRefinement,
    scroll_handle: Option<ScrollHandle>,
    show_scrollbar: bool,
}

impl std::fmt::Debug for ScrollAreaRoot {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ScrollAreaRoot")
            .field("scrollbars_len", &self.scrollbars.len())
            .field("corner", &self.corner)
            .field("scrollbar_type", &self.scrollbar_type)
            .field("scroll_hide_delay_ticks", &self.scroll_hide_delay_ticks)
            .field("layout", &self.layout)
            .field("show_scrollbar", &self.show_scrollbar)
            .finish()
    }
}

impl ScrollAreaRoot {
    pub fn new(viewport: ScrollAreaViewport) -> Self {
        Self {
            viewport,
            scrollbars: Vec::new(),
            corner: false,
            scrollbar_type: ScrollAreaType::default(),
            scroll_hide_delay_ticks: DEFAULT_SCROLL_HIDE_DELAY_TICKS,
            // Allow scroll areas to shrink inside flex containers (Tailwind's `min-w-0 min-h-0`).
            //
            // This avoids the classic "flex + scroll" failure mode where the scroll viewport
            // refuses to shrink below its content size (causing overflow or clipped-to-zero
            // behavior depending on parent constraints).
            layout: LayoutRefinement::default().min_w_0().min_h_0(),
            scroll_handle: None,
            show_scrollbar: true,
        }
    }

    pub fn show_scrollbar(mut self, show: bool) -> Self {
        self.show_scrollbar = show;
        self
    }

    /// Matches Radix ScrollArea `type` outcome.
    pub fn type_(mut self, scrollbar_type: ScrollAreaType) -> Self {
        self.scrollbar_type = scrollbar_type;
        self
    }

    /// Mirrors Radix `scrollHideDelay` (default 600ms).
    ///
    /// Fret currently expresses this value in frame-ish ticks (assuming ~60fps).
    pub fn scroll_hide_delay_ticks(mut self, ticks: u64) -> Self {
        self.scroll_hide_delay_ticks = ticks;
        self
    }

    pub fn refine_layout(mut self, layout: LayoutRefinement) -> Self {
        self.layout = self.layout.merge(layout);
        self
    }

    pub fn scroll_handle(mut self, handle: ScrollHandle) -> Self {
        self.scroll_handle = Some(handle);
        self
    }

    pub fn scrollbar(mut self, scrollbar: ScrollAreaScrollbar) -> Self {
        self.scrollbars.push(scrollbar);
        self
    }

    pub fn corner(mut self, corner: bool) -> Self {
        self.corner = corner;
        self
    }

    pub fn into_element<H: UiHost>(self, cx: &mut ElementContext<'_, H>) -> AnyElement {
        let theme = Theme::global(&*cx.app).clone();

        let viewport = self.viewport;
        let scrollbars = self.scrollbars;
        let corner = self.corner;
        let scrollbar_type = self.scrollbar_type;
        let scroll_hide_delay_ticks = self.scroll_hide_delay_ticks;
        let layout = self.layout;
        let scroll_handle = self.scroll_handle;
        let show_scrollbar = self.show_scrollbar;

        cx.hover_region(HoverRegionProps::default(), move |cx, hovered| {
            let ScrollAreaViewport {
                children: viewport_children,
                axis: viewport_axis,
                probe_unbounded: viewport_probe_unbounded,
                viewport_test_id,
                intrinsic_measure_mode,
                ..
            } = viewport;

            let handle = scroll_handle
                .unwrap_or_else(|| cx.with_state(ScrollHandle::default, |h| h.clone()));

            let visible = show_scrollbar
                && fret_ui_kit::primitives::scroll_area::scrollbar_visibility(
                    cx,
                    scrollbar_type,
                    hovered,
                    handle.clone(),
                    scroll_hide_delay_ticks,
                )
                .visible;

            let max_offset = handle.max_offset();
            let wants_x = scrollbars
                .iter()
                .any(|s| s.orientation == ScrollAreaScrollbarOrientation::Horizontal);
            let wants_y = scrollbars
                .iter()
                .any(|s| s.orientation == ScrollAreaScrollbarOrientation::Vertical);

            let axis = match (wants_x, wants_y) {
                (true, true) => ScrollAxis::Both,
                (true, false) => ScrollAxis::X,
                (false, true) => ScrollAxis::Y,
                (false, false) => viewport_axis,
            };

            let overflow_x = wants_x && max_offset.x.0 > 0.01;
            let overflow_y = wants_y && max_offset.y.0 > 0.01;

            let show_scrollbar_x = overflow_x && visible;
            let show_scrollbar_y = overflow_y && visible;

            let mut layout = decl_style::layout_style(&theme, layout);
            if matches!(layout.size.width, Length::Auto) {
                layout.size.width = Length::Fill;
            }
            // Radix/shadcn ScrollArea roots typically behave like `size: 100%` containers. When the
            // author provides a `max-height` (cmdk-style lists), we keep `height: auto` so the root
            // can shrink-wrap the content up to the cap.
            if matches!(layout.size.height, Length::Auto) && layout.size.max_height.is_none() {
                layout.size.height = Length::Fill;
            }
            layout.size.min_width.get_or_insert(Px(0.0));
            layout.size.min_height.get_or_insert(Px(0.0));
            let shrinkwrap_height_via_max_h =
                matches!(layout.size.height, Length::Auto) && layout.size.max_height.is_some();
            vec![cx.stack_props(StackProps { layout }, move |cx| {
                let mut scroll_layout = LayoutStyle::default();
                scroll_layout.size.width = Length::Fill;
                // When the root is shrink-wrapped via `max-height` (cmdk-style
                // `max-h-[...] overflow-y-auto`), avoid `Fill` (percent sizing) on the viewport.
                // Percent heights under an auto-height containing block resolve to 0 in layout
                // engines like Taffy, which breaks hit-testing and hover-driven selection.
                scroll_layout.size.height = if shrinkwrap_height_via_max_h {
                    Length::Auto
                } else {
                    Length::Fill
                };
                scroll_layout.size.min_width = Some(Px(0.0));
                scroll_layout.size.min_height = Some(Px(0.0));
                scroll_layout.overflow = Overflow::Clip;

                let scroll = cx.scroll(
                    ScrollProps {
                        layout: scroll_layout,
                        axis,
                        scroll_handle: Some(handle.clone()),
                        windowed_paint: false,
                        probe_unbounded: viewport_probe_unbounded,
                        intrinsic_measure_mode,
                    },
                    move |_cx| viewport_children,
                );

                let scroll_id = scroll.id;
                let viewport = match viewport_test_id {
                    Some(test_id) => cx.semantics(
                        SemanticsProps {
                            role: fret_core::SemanticsRole::Group,
                            test_id: Some(test_id),
                            ..Default::default()
                        },
                        move |_cx| vec![scroll],
                    ),
                    None => scroll,
                };
                let mut children = vec![viewport];

                let thumb = shadcn_scrollbar_thumb(&theme);
                let thumb_hover = shadcn_scrollbar_thumb_hover(&theme);
                let scrollbar_width = theme.metric_required("metric.scrollbar.width");

                if wants_y {
                    if let Some(spec) = scrollbars
                        .iter()
                        .find(|s| s.orientation == ScrollAreaScrollbarOrientation::Vertical)
                    {
                        let gate_layout = if overflow_y {
                            LayoutStyle {
                                position: PositionStyle::Absolute,
                                inset: InsetStyle {
                                    top: Some(Px(0.0)),
                                    right: Some(Px(0.0)),
                                    bottom: Some(if overflow_x {
                                        scrollbar_width
                                    } else {
                                        Px(0.0)
                                    }),
                                    left: None,
                                },
                                size: SizeStyle {
                                    width: Length::Px(scrollbar_width),
                                    ..Default::default()
                                },
                                ..Default::default()
                            }
                        } else {
                            LayoutStyle {
                                position: PositionStyle::Absolute,
                                inset: InsetStyle {
                                    top: Some(Px(0.0)),
                                    right: Some(Px(0.0)),
                                    ..Default::default()
                                },
                                size: SizeStyle {
                                    width: Length::Px(Px(0.0)),
                                    height: Length::Px(Px(0.0)),
                                    ..Default::default()
                                },
                                ..Default::default()
                            }
                        };

                        let mut scrollbar_layout = LayoutStyle::default();
                        scrollbar_layout.size.width = Length::Fill;
                        scrollbar_layout.size.height = Length::Fill;

                        let scrollbar = cx.scrollbar(ScrollbarProps {
                            layout: scrollbar_layout,
                            axis: ScrollbarAxis::Vertical,
                            scroll_target: Some(scroll_id),
                            scroll_handle: handle.clone(),
                            style: ScrollbarStyle {
                                thumb,
                                thumb_hover,
                                thumb_idle_alpha: spec.thumb_idle_alpha,
                                track_padding: spec.track_padding,
                            },
                        });

                        children.push(cx.interactivity_gate_props(
                            fret_ui::element::InteractivityGateProps {
                                layout: gate_layout,
                                present: overflow_y,
                                interactive: show_scrollbar_y,
                            },
                            move |cx| {
                                vec![cx.opacity(if show_scrollbar_y { 1.0 } else { 0.0 }, |_cx| {
                                    vec![scrollbar]
                                })]
                            },
                        ));
                    }
                }

                if wants_x {
                    if let Some(spec) = scrollbars
                        .iter()
                        .find(|s| s.orientation == ScrollAreaScrollbarOrientation::Horizontal)
                    {
                        let gate_layout = if overflow_x {
                            LayoutStyle {
                                position: PositionStyle::Absolute,
                                inset: InsetStyle {
                                    top: None,
                                    right: Some(if overflow_y { scrollbar_width } else { Px(0.0) }),
                                    bottom: Some(Px(0.0)),
                                    left: Some(Px(0.0)),
                                },
                                size: SizeStyle {
                                    height: Length::Px(scrollbar_width),
                                    ..Default::default()
                                },
                                ..Default::default()
                            }
                        } else {
                            LayoutStyle {
                                position: PositionStyle::Absolute,
                                inset: InsetStyle {
                                    left: Some(Px(0.0)),
                                    bottom: Some(Px(0.0)),
                                    ..Default::default()
                                },
                                size: SizeStyle {
                                    width: Length::Px(Px(0.0)),
                                    height: Length::Px(Px(0.0)),
                                    ..Default::default()
                                },
                                ..Default::default()
                            }
                        };

                        let mut scrollbar_layout = LayoutStyle::default();
                        scrollbar_layout.size.width = Length::Fill;
                        scrollbar_layout.size.height = Length::Fill;

                        let scrollbar = cx.scrollbar(ScrollbarProps {
                            layout: scrollbar_layout,
                            axis: ScrollbarAxis::Horizontal,
                            scroll_target: Some(scroll_id),
                            scroll_handle: handle.clone(),
                            style: ScrollbarStyle {
                                thumb,
                                thumb_hover,
                                thumb_idle_alpha: spec.thumb_idle_alpha,
                                track_padding: spec.track_padding,
                            },
                        });

                        children.push(cx.interactivity_gate_props(
                            fret_ui::element::InteractivityGateProps {
                                layout: gate_layout,
                                present: overflow_x,
                                interactive: show_scrollbar_x,
                            },
                            move |cx| {
                                vec![cx.opacity(if show_scrollbar_x { 1.0 } else { 0.0 }, |_cx| {
                                    vec![scrollbar]
                                })]
                            },
                        ));
                    }
                }

                if corner && wants_x && wants_y {
                    let gate_layout = if overflow_x && overflow_y {
                        LayoutStyle {
                            position: PositionStyle::Absolute,
                            inset: InsetStyle {
                                right: Some(Px(0.0)),
                                bottom: Some(Px(0.0)),
                                ..Default::default()
                            },
                            size: SizeStyle {
                                width: Length::Px(scrollbar_width),
                                height: Length::Px(scrollbar_width),
                                ..Default::default()
                            },
                            ..Default::default()
                        }
                    } else {
                        LayoutStyle {
                            position: PositionStyle::Absolute,
                            inset: InsetStyle {
                                right: Some(Px(0.0)),
                                bottom: Some(Px(0.0)),
                                ..Default::default()
                            },
                            size: SizeStyle {
                                width: Length::Px(Px(0.0)),
                                height: Length::Px(Px(0.0)),
                                ..Default::default()
                            },
                            ..Default::default()
                        }
                    };

                    let corner = cx.container(
                        ContainerProps {
                            layout: {
                                let mut layout = LayoutStyle::default();
                                layout.size.width = Length::Fill;
                                layout.size.height = Length::Fill;
                                layout
                            },
                            background: Some(shadcn_scrollbar_corner_bg(&theme)),
                            ..Default::default()
                        },
                        |_cx| vec![],
                    );

                    children.push(cx.interactivity_gate_props(
                        fret_ui::element::InteractivityGateProps {
                            layout: gate_layout,
                            present: overflow_x && overflow_y,
                            interactive: false,
                        },
                        move |cx| {
                            vec![cx.opacity(
                                if show_scrollbar_x && show_scrollbar_y {
                                    1.0
                                } else {
                                    0.0
                                },
                                |_cx| vec![corner],
                            )]
                        },
                    ));
                }

                children
            })]
        })
    }
}

#[derive(Debug, Clone)]
pub struct ScrollArea {
    children: Vec<AnyElement>,
    axis: ScrollAxis,
    show_scrollbar: bool,
    scrollbar_type: ScrollAreaType,
    scroll_hide_delay_ticks: u64,
    layout: LayoutRefinement,
    scroll_handle: Option<ScrollHandle>,
    viewport_test_id: Option<Arc<str>>,
    viewport_intrinsic_measure_mode: Option<ScrollIntrinsicMeasureMode>,
}

impl ScrollArea {
    pub fn new(children: impl IntoIterator<Item = AnyElement>) -> Self {
        let children = children.into_iter().collect();
        Self {
            children,
            axis: ScrollAxis::Y,
            show_scrollbar: true,
            scrollbar_type: ScrollAreaType::default(),
            scroll_hide_delay_ticks: DEFAULT_SCROLL_HIDE_DELAY_TICKS,
            // Same rationale as `ScrollAreaRoot`: make the common case "safe by default" in
            // editor-like shells where scroll areas routinely live inside flex stacks.
            layout: LayoutRefinement::default().min_w_0().min_h_0(),
            scroll_handle: None,
            viewport_test_id: None,
            viewport_intrinsic_measure_mode: None,
        }
    }

    pub fn axis(mut self, axis: ScrollAxis) -> Self {
        self.axis = axis;
        self
    }

    pub fn show_scrollbar(mut self, show: bool) -> Self {
        self.show_scrollbar = show;
        self
    }

    /// Matches Radix ScrollArea `type` outcome (best-effort).
    pub fn type_(mut self, scrollbar_type: ScrollAreaType) -> Self {
        self.scrollbar_type = scrollbar_type;
        self
    }

    /// Mirrors Radix `scrollHideDelay` (default 600ms).
    ///
    /// Fret currently expresses this value in frame-ish ticks (assuming ~60fps).
    pub fn scroll_hide_delay_ticks(mut self, ticks: u64) -> Self {
        self.scroll_hide_delay_ticks = ticks;
        self
    }

    pub fn refine_layout(mut self, layout: LayoutRefinement) -> Self {
        self.layout = self.layout.merge(layout);
        self
    }

    pub fn scroll_handle(mut self, handle: ScrollHandle) -> Self {
        self.scroll_handle = Some(handle);
        self
    }

    pub fn viewport_test_id(mut self, test_id: impl Into<Arc<str>>) -> Self {
        self.viewport_test_id = Some(test_id.into());
        self
    }

    pub fn viewport_intrinsic_measure_mode(mut self, mode: ScrollIntrinsicMeasureMode) -> Self {
        self.viewport_intrinsic_measure_mode = Some(mode);
        self
    }

    pub fn into_element<H: UiHost>(self, cx: &mut ElementContext<'_, H>) -> AnyElement {
        let mut viewport = ScrollAreaViewport::new(self.children).axis(self.axis);
        if let Some(test_id) = self.viewport_test_id {
            viewport = viewport.viewport_test_id(test_id);
        }
        if let Some(mode) = self.viewport_intrinsic_measure_mode {
            viewport = viewport.intrinsic_measure_mode(mode);
        }

        let mut root = ScrollAreaRoot::new(viewport)
            .show_scrollbar(self.show_scrollbar)
            .type_(self.scrollbar_type)
            .scroll_hide_delay_ticks(self.scroll_hide_delay_ticks)
            .refine_layout(self.layout)
            .corner(matches!(self.axis, ScrollAxis::Both));

        if self.axis.scroll_y() {
            root = root.scrollbar(
                ScrollAreaScrollbar::new().orientation(ScrollAreaScrollbarOrientation::Vertical),
            );
        }

        if self.axis.scroll_x() {
            root = root.scrollbar(
                ScrollAreaScrollbar::new().orientation(ScrollAreaScrollbarOrientation::Horizontal),
            );
        }

        if let Some(handle) = self.scroll_handle {
            root = root.scroll_handle(handle);
        }

        root.into_element(cx)
    }
}

pub fn scroll_area<H: UiHost, I>(
    cx: &mut ElementContext<'_, H>,
    f: impl FnOnce(&mut ElementContext<'_, H>) -> I,
) -> AnyElement
where
    I: IntoIterator<Item = AnyElement>,
{
    ScrollArea::new(f(cx)).into_element(cx)
}

#[cfg(test)]
mod tests {
    use super::*;
    use fret_app::App;
    use fret_core::{
        AppWindowId, Modifiers, MouseButtons, Point, Px, Rect, Size, SvgId, SvgService,
    };
    use fret_core::{PathCommand, PathConstraints, PathId, PathMetrics, PathService, PathStyle};
    use fret_core::{TextBlobId, TextConstraints, TextMetrics, TextService};
    use fret_runtime::TickId;
    use fret_ui::element::{ColumnProps, ContainerProps, LayoutStyle, Length};
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
                    size: Size::new(Px(10.0), Px(10.0)),
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

    fn bounds() -> Rect {
        Rect::new(
            Point::new(Px(0.0), Px(0.0)),
            Size::new(Px(400.0), Px(240.0)),
        )
    }

    fn render_with<C, I>(
        ui: &mut UiTree<App>,
        app: &mut App,
        services: &mut dyn fret_core::UiServices,
        window: AppWindowId,
        ty: ScrollAreaType,
        content: C,
    ) -> fret_core::NodeId
    where
        C: FnOnce(&mut ElementContext<'_, App>) -> I,
        I: IntoIterator<Item = AnyElement>,
    {
        render_with_axis(ui, app, services, window, ScrollAxis::Y, ty, content)
    }

    fn render_with_axis<C, I>(
        ui: &mut UiTree<App>,
        app: &mut App,
        services: &mut dyn fret_core::UiServices,
        window: AppWindowId,
        axis: ScrollAxis,
        ty: ScrollAreaType,
        content: C,
    ) -> fret_core::NodeId
    where
        C: FnOnce(&mut ElementContext<'_, App>) -> I,
        I: IntoIterator<Item = AnyElement>,
    {
        let root =
            fret_ui::declarative::render_root(ui, app, services, window, bounds(), "sa", |cx| {
                vec![
                    ScrollArea::new(content(cx))
                        .axis(axis)
                        .type_(ty)
                        .into_element(cx),
                ]
            });
        ui.set_root(root);
        ui.layout_all(app, services, bounds(), 1.0);
        root
    }

    fn render(
        ui: &mut UiTree<App>,
        app: &mut App,
        services: &mut dyn fret_core::UiServices,
        window: AppWindowId,
        ty: ScrollAreaType,
    ) -> fret_core::NodeId {
        render_with(ui, app, services, window, ty, |cx| {
            vec![cx.column(ColumnProps::default(), |cx| {
                (0..50).map(|_| cx.text("Row")).collect::<Vec<_>>()
            })]
        })
    }

    fn point_on_vertical_scrollbar(bounds: Rect) -> Point {
        Point::new(
            Px(bounds.origin.x.0 + bounds.size.width.0 - 1.0),
            Px(bounds.origin.y.0 + 10.0),
        )
    }

    fn point_on_horizontal_scrollbar(bounds: Rect) -> Point {
        Point::new(
            Px(bounds.origin.x.0 + 10.0),
            Px(bounds.origin.y.0 + bounds.size.height.0 - 1.0),
        )
    }

    fn point_in_content(bounds: Rect) -> Point {
        Point::new(Px(bounds.origin.x.0 + 10.0), Px(bounds.origin.y.0 + 10.0))
    }

    fn assert_hit_matches(ui: &UiTree<App>, p: Point, expected: fret_core::NodeId, msg: &str) {
        let hit = ui.debug_hit_test(p).hit;
        assert_eq!(
            hit,
            Some(expected),
            "{msg} (hit={hit:?} expected={expected:?} p={p:?})"
        );
    }

    fn assert_hit_differs(ui: &UiTree<App>, p: Point, baseline: fret_core::NodeId, msg: &str) {
        let hit = ui.debug_hit_test(p).hit;
        assert_ne!(
            hit,
            Some(baseline),
            "{msg} (hit={hit:?} baseline={baseline:?} p={p:?})"
        );
        let hit = hit.expect(msg);
        let hit_bounds = ui.debug_node_bounds(hit).expect("hit bounds");
        assert!(
            hit_bounds.contains(p),
            "{msg} (hit={hit:?} hit_bounds={hit_bounds:?} p={p:?})"
        );
    }

    #[test]
    fn scroll_area_hover_type_shows_scrollbar_only_when_hovered() {
        let window = AppWindowId::default();
        let mut app = App::new();
        let mut ui: UiTree<App> = UiTree::new();
        ui.set_window(window);

        let mut services = FakeServices::default();

        let root = render(
            &mut ui,
            &mut app,
            &mut services,
            window,
            ScrollAreaType::Hover,
        );
        // Root -> HoverRegion -> Stack -> Scroll (+ structurally stable scrollbar chrome).
        let hover_region = ui.children(root)[0];
        let stack = ui.children(hover_region)[0];
        let stack_bounds = ui.debug_node_bounds(stack).expect("stack bounds");
        let baseline = ui
            .debug_hit_test(point_in_content(stack_bounds))
            .hit
            .expect("baseline hit");
        assert_hit_matches(
            &ui,
            point_on_vertical_scrollbar(stack_bounds),
            baseline,
            "expected scrollbar region to hit the same target before hover",
        );

        ui.dispatch_event(
            &mut app,
            &mut services,
            &fret_core::Event::Pointer(fret_core::PointerEvent::Move {
                pointer_id: fret_core::PointerId(0),
                position: Point::new(Px(10.0), Px(10.0)),
                buttons: MouseButtons::default(),
                modifiers: Modifiers::default(),
                pointer_type: fret_core::PointerType::Mouse,
            }),
        );

        let root = render(
            &mut ui,
            &mut app,
            &mut services,
            window,
            ScrollAreaType::Hover,
        );
        let hover_region = ui.children(root)[0];
        let stack = ui.children(hover_region)[0];
        let stack_bounds = ui.debug_node_bounds(stack).expect("stack bounds");
        let baseline = ui
            .debug_hit_test(point_in_content(stack_bounds))
            .hit
            .expect("baseline hit");
        assert_hit_differs(
            &ui,
            point_on_vertical_scrollbar(stack_bounds),
            baseline,
            "expected scrollbar hit target on hover",
        );
    }

    #[test]
    fn scroll_area_auto_type_shows_scrollbar_when_overflowing() {
        let window = AppWindowId::default();
        let mut app = App::new();
        let mut ui: UiTree<App> = UiTree::new();
        ui.set_window(window);

        let mut services = FakeServices::default();

        // First render establishes the scroll handle's viewport/content sizes during layout.
        let _ = render(
            &mut ui,
            &mut app,
            &mut services,
            window,
            ScrollAreaType::Auto,
        );
        let root = render(
            &mut ui,
            &mut app,
            &mut services,
            window,
            ScrollAreaType::Auto,
        );

        // Auto type: scrollbar should be interactive when overflowing (without requiring hover).
        let hover_region = ui.children(root)[0];
        let stack = ui.children(hover_region)[0];
        let stack_bounds = ui.debug_node_bounds(stack).expect("stack bounds");
        let baseline = ui
            .debug_hit_test(point_in_content(stack_bounds))
            .hit
            .expect("baseline hit");
        assert_hit_differs(
            &ui,
            point_on_vertical_scrollbar(stack_bounds),
            baseline,
            "expected auto scrollbar hit target for overflow",
        );
    }

    #[test]
    fn scroll_area_scroll_type_shows_while_scrolling_then_hides() {
        let window = AppWindowId::default();
        let mut app = App::new();
        let mut ui: UiTree<App> = UiTree::new();
        ui.set_window(window);

        let mut services = FakeServices::default();
        let handle = ScrollHandle::default();

        let root = fret_ui::declarative::render_root(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds(),
            "sa-scroll",
            |cx| {
                vec![
                    ScrollArea::new(vec![cx.column(ColumnProps::default(), |cx| {
                        (0..50).map(|_| cx.text("Row")).collect::<Vec<_>>()
                    })])
                    .type_(ScrollAreaType::Scroll)
                    .scroll_hide_delay_ticks(4)
                    .scroll_handle(handle.clone())
                    .into_element(cx),
                ]
            },
        );
        ui.set_root(root);
        ui.layout_all(&mut app, &mut services, bounds(), 1.0);

        let hover_region = ui.children(root)[0];
        let stack = ui.children(hover_region)[0];
        let stack_bounds = ui.debug_node_bounds(stack).expect("stack bounds");
        let baseline = ui
            .debug_hit_test(point_in_content(stack_bounds))
            .hit
            .expect("baseline hit");
        assert_hit_matches(
            &ui,
            point_on_vertical_scrollbar(stack_bounds),
            baseline,
            "expected scrollbar region to hit the same target before any scrolling",
        );

        // Simulate a scroll delta by mutating the shared handle between frames.
        handle.set_offset(Point::new(Px(0.0), Px(10.0)));
        app.set_tick_id(TickId(1));

        let root = fret_ui::declarative::render_root(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds(),
            "sa-scroll",
            |cx| {
                vec![
                    ScrollArea::new(vec![cx.column(ColumnProps::default(), |cx| {
                        (0..50).map(|_| cx.text("Row")).collect::<Vec<_>>()
                    })])
                    .type_(ScrollAreaType::Scroll)
                    .scroll_hide_delay_ticks(4)
                    .scroll_handle(handle.clone())
                    .into_element(cx),
                ]
            },
        );
        ui.set_root(root);
        ui.layout_all(&mut app, &mut services, bounds(), 1.0);

        let hover_region = ui.children(root)[0];
        let stack = ui.children(hover_region)[0];
        let stack_bounds = ui.debug_node_bounds(stack).expect("stack bounds");
        let baseline = ui
            .debug_hit_test(point_in_content(stack_bounds))
            .hit
            .expect("baseline hit");
        assert_hit_differs(
            &ui,
            point_on_vertical_scrollbar(stack_bounds),
            baseline,
            "expected scrollbar hit target while scrolling",
        );

        // Keep rendering without scroll input; after debounce + hide delay it should disappear.
        for n in 0..12 {
            app.set_tick_id(TickId(2 + n));
            let root = fret_ui::declarative::render_root(
                &mut ui,
                &mut app,
                &mut services,
                window,
                bounds(),
                "sa-scroll",
                |cx| {
                    vec![
                        ScrollArea::new(vec![cx.column(ColumnProps::default(), |cx| {
                            (0..50).map(|_| cx.text("Row")).collect::<Vec<_>>()
                        })])
                        .type_(ScrollAreaType::Scroll)
                        .scroll_hide_delay_ticks(4)
                        .scroll_handle(handle.clone())
                        .into_element(cx),
                    ]
                },
            );
            ui.set_root(root);
            ui.layout_all(&mut app, &mut services, bounds(), 1.0);
        }

        let root = fret_ui::declarative::render_root(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds(),
            "sa-scroll",
            |cx| {
                vec![
                    ScrollArea::new(vec![cx.column(ColumnProps::default(), |cx| {
                        (0..50).map(|_| cx.text("Row")).collect::<Vec<_>>()
                    })])
                    .type_(ScrollAreaType::Scroll)
                    .scroll_hide_delay_ticks(4)
                    .scroll_handle(handle.clone())
                    .into_element(cx),
                ]
            },
        );
        ui.set_root(root);
        ui.layout_all(&mut app, &mut services, bounds(), 1.0);

        let hover_region = ui.children(root)[0];
        let stack = ui.children(hover_region)[0];
        let stack_bounds = ui.debug_node_bounds(stack).expect("stack bounds");
        let baseline = ui
            .debug_hit_test(point_in_content(stack_bounds))
            .hit
            .expect("baseline hit");
        assert_hit_matches(
            &ui,
            point_on_vertical_scrollbar(stack_bounds),
            baseline,
            "expected scrollbar region to hit the same target after scroll ends",
        );
    }

    #[test]
    fn scroll_area_auto_type_mounts_horizontal_scrollbar_when_overflowing_x() {
        let window = AppWindowId::default();
        let mut app = App::new();
        let mut ui: UiTree<App> = UiTree::new();
        ui.set_window(window);

        let mut services = FakeServices::default();

        let wide = |cx: &mut ElementContext<'_, App>| {
            let mut layout = LayoutStyle::default();
            layout.size.width = Length::Px(Px(800.0));
            layout.size.height = Length::Px(Px(10.0));
            vec![cx.container(
                ContainerProps {
                    layout,
                    ..Default::default()
                },
                |_cx| vec![],
            )]
        };

        let _ = render_with_axis(
            &mut ui,
            &mut app,
            &mut services,
            window,
            ScrollAxis::X,
            ScrollAreaType::Auto,
            wide,
        );
        let root = render_with_axis(
            &mut ui,
            &mut app,
            &mut services,
            window,
            ScrollAxis::X,
            ScrollAreaType::Auto,
            wide,
        );

        // Auto type: horizontal scrollbar should be interactive when overflowing in X.
        let hover_region = ui.children(root)[0];
        let stack = ui.children(hover_region)[0];
        let stack_bounds = ui.debug_node_bounds(stack).expect("stack bounds");
        let baseline = ui
            .debug_hit_test(point_in_content(stack_bounds))
            .hit
            .expect("baseline hit");
        assert_hit_differs(
            &ui,
            point_on_horizontal_scrollbar(stack_bounds),
            baseline,
            "expected horizontal scrollbar hit target",
        );
    }

    #[test]
    fn scroll_area_auto_type_mounts_two_scrollbars_and_corner_for_both_overflow() {
        let window = AppWindowId::default();
        let mut app = App::new();
        let mut ui: UiTree<App> = UiTree::new();
        ui.set_window(window);

        let mut services = FakeServices::default();

        let large = |cx: &mut ElementContext<'_, App>| {
            let mut layout = LayoutStyle::default();
            layout.size.width = Length::Px(Px(800.0));
            layout.size.height = Length::Px(Px(800.0));
            vec![cx.container(
                ContainerProps {
                    layout,
                    ..Default::default()
                },
                |_cx| vec![],
            )]
        };

        let _ = render_with_axis(
            &mut ui,
            &mut app,
            &mut services,
            window,
            ScrollAxis::Both,
            ScrollAreaType::Auto,
            large,
        );
        let root = render_with_axis(
            &mut ui,
            &mut app,
            &mut services,
            window,
            ScrollAxis::Both,
            ScrollAreaType::Auto,
            large,
        );

        // Auto type: both scrollbars should be interactive when overflowing in both axes.
        let hover_region = ui.children(root)[0];
        let stack = ui.children(hover_region)[0];
        let stack_bounds = ui.debug_node_bounds(stack).expect("stack bounds");
        let baseline = ui
            .debug_hit_test(point_in_content(stack_bounds))
            .hit
            .expect("baseline hit");
        assert_hit_differs(
            &ui,
            point_on_vertical_scrollbar(stack_bounds),
            baseline,
            "expected vertical scrollbar hit target for overflow-both",
        );
        assert_hit_differs(
            &ui,
            point_on_horizontal_scrollbar(stack_bounds),
            baseline,
            "expected horizontal scrollbar hit target for overflow-both",
        );
    }

    #[test]
    fn scroll_area_explicit_handle_reports_overflow_across_renders() {
        let window = AppWindowId::default();
        let mut app = App::new();
        let mut ui: UiTree<App> = UiTree::new();
        ui.set_window(window);

        let mut services = FakeServices::default();
        let handle = ScrollHandle::default();

        let root = fret_ui::declarative::render_root(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds(),
            "sa-explicit-handle",
            |cx| {
                vec![
                    ScrollArea::new(vec![cx.column(ColumnProps::default(), |cx| {
                        (0..50).map(|_| cx.text("Row")).collect::<Vec<_>>()
                    })])
                    .type_(ScrollAreaType::Auto)
                    .scroll_handle(handle.clone())
                    .into_element(cx),
                ]
            },
        );
        ui.set_root(root);
        ui.layout_all(&mut app, &mut services, bounds(), 1.0);

        let hover_region = ui.children(root)[0];
        let stack = ui.children(hover_region)[0];
        let scroll = ui.children(stack)[0];
        let root_bounds = ui.debug_node_bounds(root).expect("root bounds");
        let hover_bounds = ui.debug_node_bounds(hover_region).expect("hover bounds");
        let stack_bounds = ui.debug_node_bounds(stack).expect("stack bounds");
        let scroll_bounds = ui.debug_node_bounds(scroll).expect("scroll bounds");

        assert!(
            handle.max_offset().y.0 > 0.01,
            "expected explicit scroll handle to observe overflow after layout (viewport={:?} content={:?} max_offset={:?} root={:?} hover={:?} stack={:?} scroll={:?})",
            handle.viewport_size(),
            handle.content_size(),
            handle.max_offset(),
            root_bounds,
            hover_bounds,
            stack_bounds,
            scroll_bounds,
        );

        let root = fret_ui::declarative::render_root(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds(),
            "sa-explicit-handle",
            |cx| {
                vec![
                    ScrollArea::new(vec![cx.column(ColumnProps::default(), |cx| {
                        (0..50).map(|_| cx.text("Row")).collect::<Vec<_>>()
                    })])
                    .type_(ScrollAreaType::Auto)
                    .scroll_handle(handle.clone())
                    .into_element(cx),
                ]
            },
        );
        ui.set_root(root);
        ui.layout_all(&mut app, &mut services, bounds(), 1.0);

        let hover_region = ui.children(root)[0];
        let stack = ui.children(hover_region)[0];
        assert_eq!(
            ui.children(stack).len(),
            2,
            "expected auto scrollbar to mount for overflow when using an explicit handle"
        );
    }
}
