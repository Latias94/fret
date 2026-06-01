//! Material 3 search view (docked, MVP).
//!
//! This component composes a `SearchBar`-like input surface with a dismissible popover panel that
//! can host arbitrary results/suggestions content.

use std::cell::Cell;
use std::rc::Rc;
use std::sync::Arc;

use fret_core::{Axis, Color, Corners, Edges, Px, SemanticsRole};
use fret_icons::IconId;
use fret_runtime::Model;
use fret_ui::element::{
    AnyElement, ContainerProps, CrossAlign, FlexProps, Length, MainAlign, Overflow, SemanticsProps,
};
use fret_ui::elements::ElementContext;
use fret_ui::{GlobalElementId, Theme, UiHost};
use fret_ui_kit::declarative::controllable_state;
use fret_ui_kit::overlay_controller;
use fret_ui_kit::primitives::focus_scope as focus_scope_prim;
use fret_ui_kit::{
    ColorRef, OverlayController, OverlayPresence, OverrideSlot, WidgetStateProperty, WidgetStates,
    merge_override_slot, resolve_override_slot_with,
};

use crate::foundation::context::material_layout_direction_in_scope;
use crate::foundation::elevation::shadow_for_elevation_with_color;
use crate::foundation::search_motion::{
    SearchMotionKind, drive_search_motion, search_full_screen_geometry_transform,
};
use crate::foundation::test_id::part_test_id;
use crate::search_bar::SearchBarHeaderTokens;
use crate::tokens::{dropdown_menu as dropdown_menu_tokens, search_view as search_view_tokens};
use crate::{SearchBar, SearchBarStyle};

fn search_view_color_override(
    theme: &Theme,
    slot: &OverrideSlot<ColorRef>,
    states: WidgetStates,
    fallback: impl FnOnce() -> Color,
) -> Color {
    resolve_override_slot_with(
        slot.as_ref(),
        states,
        |color| color.resolve(theme),
        fallback,
    )
}

fn search_view_metric_override(
    slot: &OverrideSlot<Px>,
    states: WidgetStates,
    fallback: impl FnOnce() -> Px,
) -> Px {
    resolve_override_slot_with(slot.as_ref(), states, |value| *value, fallback)
}

fn search_view_edges_override(
    slot: &OverrideSlot<Edges>,
    states: WidgetStates,
    fallback: impl FnOnce() -> Edges,
) -> Edges {
    resolve_override_slot_with(slot.as_ref(), states, |value| *value, fallback)
}

fn search_view_corners_override(
    slot: &OverrideSlot<Corners>,
    states: WidgetStates,
    fallback: impl FnOnce() -> Corners,
) -> Corners {
    resolve_override_slot_with(slot.as_ref(), states, |value| *value, fallback)
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum SearchViewPresentation {
    #[default]
    Docked,
    FullScreen,
}

#[derive(Debug, Clone, Default)]
pub struct SearchViewStyle {
    pub header_style: SearchBarStyle,
    pub container_background: OverrideSlot<ColorRef>,
    pub container_elevation: OverrideSlot<Px>,
    pub docked_container_corner_radii: OverrideSlot<Corners>,
    pub divider_color: OverrideSlot<ColorRef>,
    pub full_screen_header_container_height: OverrideSlot<Px>,
    pub body_padding: OverrideSlot<Edges>,
}

impl SearchViewStyle {
    pub fn header_style(mut self, style: SearchBarStyle) -> Self {
        self.header_style = self.header_style.merged(style);
        self
    }

    pub fn container_background(mut self, color: WidgetStateProperty<Option<ColorRef>>) -> Self {
        self.container_background = Some(color);
        self
    }

    pub fn container_elevation(mut self, elevation: WidgetStateProperty<Option<Px>>) -> Self {
        self.container_elevation = Some(elevation);
        self
    }

    pub fn docked_container_corner_radii(
        mut self,
        corners: WidgetStateProperty<Option<Corners>>,
    ) -> Self {
        self.docked_container_corner_radii = Some(corners);
        self
    }

    pub fn divider_color(mut self, color: WidgetStateProperty<Option<ColorRef>>) -> Self {
        self.divider_color = Some(color);
        self
    }

    pub fn full_screen_header_container_height(
        mut self,
        height: WidgetStateProperty<Option<Px>>,
    ) -> Self {
        self.full_screen_header_container_height = Some(height);
        self
    }

    pub fn body_padding(mut self, padding: WidgetStateProperty<Option<Edges>>) -> Self {
        self.body_padding = Some(padding);
        self
    }

    pub fn merged(self, other: Self) -> Self {
        Self {
            header_style: self.header_style.merged(other.header_style),
            container_background: merge_override_slot(
                self.container_background,
                other.container_background,
            ),
            container_elevation: merge_override_slot(
                self.container_elevation,
                other.container_elevation,
            ),
            docked_container_corner_radii: merge_override_slot(
                self.docked_container_corner_radii,
                other.docked_container_corner_radii,
            ),
            divider_color: merge_override_slot(self.divider_color, other.divider_color),
            full_screen_header_container_height: merge_override_slot(
                self.full_screen_header_container_height,
                other.full_screen_header_container_height,
            ),
            body_padding: merge_override_slot(self.body_padding, other.body_padding),
        }
    }
}

#[derive(Debug, Clone)]
pub struct SearchView {
    open: Model<bool>,
    query: Model<String>,
    style: SearchViewStyle,
    presentation: SearchViewPresentation,
    disabled: bool,
    placeholder: Option<Arc<str>>,
    a11y_label: Option<Arc<str>>,
    test_id: Option<Arc<str>>,
    overlay_test_id: Option<Arc<str>>,
    leading_icon: Option<IconId>,
    trailing_icon: Option<IconId>,
    window_margin: Px,
    max_height: Px,
}

impl SearchView {
    pub fn new(open: Model<bool>, query: Model<String>) -> Self {
        Self {
            open,
            query,
            style: SearchViewStyle::default(),
            presentation: SearchViewPresentation::default(),
            disabled: false,
            placeholder: None,
            a11y_label: None,
            test_id: None,
            overlay_test_id: None,
            leading_icon: None,
            trailing_icon: None,
            window_margin: Px(12.0),
            max_height: Px(360.0),
        }
    }

    /// Creates a search view with controlled/uncontrolled open and query models.
    ///
    /// When `open` or `query` is `None`, the corresponding model is stored at the root call site
    /// and initialized from the provided default.
    pub fn new_controllable<H: UiHost>(
        cx: &mut ElementContext<'_, H>,
        open: Option<Model<bool>>,
        default_open: bool,
        query: Option<Model<String>>,
        default_query: impl Into<String>,
    ) -> Self {
        let open = controllable_state::use_controllable_model(cx, open, || default_open).model();
        let default_query = default_query.into();
        let query = controllable_state::use_controllable_model(cx, query, || default_query).model();
        Self::new(open, query)
    }

    /// Default teaching-surface constructor for a search view that owns its open and query models.
    pub fn uncontrolled<H: UiHost>(cx: &mut ElementContext<'_, H>) -> Self {
        Self::new_controllable(cx, None, false, None, String::new())
    }

    /// Returns the resolved open model, including the internally owned model for uncontrolled use.
    pub fn open_model(&self) -> Model<bool> {
        self.open.clone()
    }

    /// Returns the resolved query model, including the internally owned model for uncontrolled use.
    pub fn query_model(&self) -> Model<String> {
        self.query.clone()
    }

    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }

    pub fn style(mut self, style: SearchViewStyle) -> Self {
        self.style = self.style.merged(style);
        self
    }

    pub fn presentation(mut self, presentation: SearchViewPresentation) -> Self {
        self.presentation = presentation;
        self
    }

    pub fn full_screen(mut self) -> Self {
        self.presentation = SearchViewPresentation::FullScreen;
        self
    }

    pub fn docked(mut self) -> Self {
        self.presentation = SearchViewPresentation::Docked;
        self
    }

    pub fn placeholder(mut self, placeholder: impl Into<Arc<str>>) -> Self {
        self.placeholder = Some(placeholder.into());
        self
    }

    pub fn a11y_label(mut self, label: impl Into<Arc<str>>) -> Self {
        self.a11y_label = Some(label.into());
        self
    }

    pub fn test_id(mut self, id: impl Into<Arc<str>>) -> Self {
        self.test_id = Some(id.into());
        self
    }

    pub fn overlay_test_id(mut self, id: impl Into<Arc<str>>) -> Self {
        self.overlay_test_id = Some(id.into());
        self
    }

    pub fn leading_icon(mut self, icon: IconId) -> Self {
        self.leading_icon = Some(icon);
        self
    }

    pub fn trailing_icon(mut self, icon: IconId) -> Self {
        self.trailing_icon = Some(icon);
        self
    }

    pub fn window_margin(mut self, margin: Px) -> Self {
        self.window_margin = margin;
        self
    }

    pub fn max_height(mut self, max_height: Px) -> Self {
        self.max_height = max_height;
        self
    }

    #[track_caller]
    pub fn into_element<H: UiHost>(
        self,
        cx: &mut ElementContext<'_, H>,
        content: impl FnOnce(&mut ElementContext<'_, H>) -> Vec<AnyElement>,
    ) -> AnyElement {
        cx.scope(|cx| {
            let root_test_id = self.test_id.clone();
            let input_id_out: Rc<Cell<Option<GlobalElementId>>> = Rc::new(Cell::new(None));
            let input_id_out_for_bar = input_id_out.clone();
            let controlled_element_id: Rc<Cell<Option<GlobalElementId>>> = cx.slot_state(
                || Rc::new(Cell::new(None::<GlobalElementId>)),
                |id| id.clone(),
            );

            // Keep the input surface in the underlay so:
            // - focus stays on the text input while the overlay is open (Compose-like),
            // - the overlay can be dismissed without fighting focus-gained/blur heuristics.
            let mut bar = SearchBar::new(self.query.clone())
                .style(self.style.header_style.clone())
                .disabled(self.disabled)
                .placeholder_opt(self.placeholder.clone())
                .a11y_label_opt(self.a11y_label.clone())
                .test_id_opt(root_test_id.clone())
                .expanded_model(self.open.clone())
                .header_tokens(SearchBarHeaderTokens::SearchView)
                .input_id_out(input_id_out_for_bar)
                .controls_element_id(controlled_element_id.clone());
            if let Some(icon) = self.leading_icon.as_ref() {
                bar = bar.leading_icon(icon.clone());
            }
            if let Some(icon) = self.trailing_icon.as_ref() {
                bar = bar.trailing_icon(icon.clone());
            }
            let bar = bar.into_element(cx);

            // Policy: open on focus gained (Compose-like), while keeping focus on the text input.
            if !self.disabled
                && let Some(input_id) = input_id_out.get()
            {
                let focused_input = cx.is_focused_element(input_id);

                #[derive(Default)]
                struct FrameState {
                    was_focused_input: bool,
                }

                let focus_gained = cx.slot_state(FrameState::default, |st| {
                    let focus_gained = focused_input && !st.was_focused_input;
                    st.was_focused_input = focused_input;
                    focus_gained
                });

                if focus_gained {
                    let _ = cx.app.models_mut().update(&self.open, |v| *v = true);
                    cx.app.request_redraw(cx.window);
                }
            }

            let is_open = cx
                .get_model_copied(&self.open, fret_ui::Invalidation::Layout)
                .unwrap_or(false);

            let close_grace_frames = {
                let theme = Theme::global(&*cx.app);
                Some(crate::motion::ms_to_frames(
                    dropdown_menu_tokens::close_duration_ms(theme),
                ))
            };
            let motion_kind = match self.presentation {
                SearchViewPresentation::Docked => SearchMotionKind::Docked,
                SearchViewPresentation::FullScreen => SearchMotionKind::FullScreen,
            };
            let motion = drive_search_motion(cx, is_open, motion_kind, close_grace_frames);
            let overlay_presence = OverlayPresence {
                present: motion.present,
                interactive: is_open,
            };

            if !overlay_presence.present {
                controlled_element_id.set(None);
                return bar;
            }

            let Some(input_id) = input_id_out.get() else {
                return bar;
            };

            if self.presentation == SearchViewPresentation::FullScreen {
                let viewport =
                    fret_ui_kit::overlay::outer_bounds_with_window_margin_for_environment(
                        cx,
                        fret_ui::Invalidation::Layout,
                        Px(0.0),
                    );
                let collapsed = fret_ui_kit::overlay::anchor_bounds_for_element(cx, input_id)
                    .unwrap_or(viewport);
                let full_screen_transform =
                    search_full_screen_geometry_transform(motion.progress, viewport, collapsed);
                let overlay_test_id = self
                    .overlay_test_id
                    .clone()
                    .or_else(|| root_test_id.as_ref().map(|id| part_test_id(id, "overlay")));
                let header_slot_test_id = root_test_id
                    .as_ref()
                    .map(|id| part_test_id(id, "overlay.header-slot"));
                let header_test_id = root_test_id
                    .as_ref()
                    .map(|id| part_test_id(id, "overlay.header"));
                let divider_test_id = root_test_id
                    .as_ref()
                    .map(|id| part_test_id(id, "overlay.divider"));
                let body_test_id = root_test_id
                    .as_ref()
                    .map(|id| part_test_id(id, "overlay.body"));
                let header_input_id_out: Rc<Cell<Option<GlobalElementId>>> =
                    Rc::new(Cell::new(None));
                let header_input_id_out_for_bar = header_input_id_out.clone();

                let mut header = SearchBar::new(self.query.clone())
                    .style(self.style.header_style.clone())
                    .disabled(self.disabled)
                    .placeholder_opt(self.placeholder.clone())
                    .a11y_label_opt(self.a11y_label.clone())
                    .test_id_opt(header_test_id)
                    .expanded_model(self.open.clone())
                    .header_tokens(SearchBarHeaderTokens::SearchView)
                    .input_id_out(header_input_id_out_for_bar)
                    .controls_element_id(controlled_element_id.clone());
                if let Some(icon) = self.leading_icon.as_ref() {
                    header = header.leading_icon(icon.clone());
                }
                if let Some(icon) = self.trailing_icon.as_ref() {
                    header = header.trailing_icon(icon.clone());
                }
                let header = header.into_element(cx);
                let initial_focus = header_input_id_out.get();
                let labelled_by_element = initial_focus.map(|id| id.0);

                let (container_color, divider_color, header_slot_height, body_padding) = {
                    let theme = Theme::global(&*cx.app);
                    let states = WidgetStates::empty();
                    (
                        search_view_color_override(
                            theme,
                            &self.style.container_background,
                            states,
                            || search_view_tokens::container_color(theme),
                        ),
                        search_view_color_override(theme, &self.style.divider_color, states, || {
                            search_view_tokens::divider_color(theme)
                        }),
                        search_view_metric_override(
                            &self.style.full_screen_header_container_height,
                            states,
                            || search_view_tokens::full_screen_header_container_height(theme),
                        ),
                        search_view_edges_override(&self.style.body_padding, states, || {
                            Edges::all(Px(8.0))
                        }),
                    )
                };

                let controlled_element_id_out = controlled_element_id.clone();
                let overlay_root = cx.named("full_screen_overlay", move |cx| {
                    let mut panel_container = ContainerProps::default();
                    panel_container.layout.size.width = Length::Fill;
                    panel_container.layout.size.height = Length::Fill;
                    panel_container.layout.overflow = Overflow::Clip;
                    panel_container.background = Some(container_color);

                    let mut column = FlexProps::default();
                    column.direction = Axis::Vertical;
                    column.justify = MainAlign::Start;
                    column.align = CrossAlign::Stretch;
                    column.wrap = false;
                    column.gap = Px(0.0).into();
                    column.layout.size.width = Length::Fill;
                    column.layout.size.height = Length::Fill;

                    let panel = cx.container(panel_container, move |cx| {
                        let mut header_slot = ContainerProps::default();
                        header_slot.layout.size.width = Length::Fill;
                        header_slot.layout.size.height = Length::Px(header_slot_height);
                        header_slot.layout.size.min_height = Some(Length::Px(header_slot_height));
                        header_slot.layout.overflow = Overflow::Visible;
                        header_slot.padding = Edges {
                            left: Px(0.0),
                            right: Px(0.0),
                            top: Px(8.0),
                            bottom: Px(8.0),
                        }
                        .into();
                        let mut header_slot = cx.container(header_slot, move |_cx| vec![header]);
                        if let Some(test_id) = header_slot_test_id.clone() {
                            header_slot = header_slot.test_id(test_id);
                        }

                        let mut divider = cx.container(
                            ContainerProps {
                                layout: fret_ui::element::LayoutStyle {
                                    size: fret_ui::element::SizeStyle {
                                        width: Length::Fill,
                                        height: Length::Px(Px(1.0)),
                                        ..Default::default()
                                    },
                                    ..Default::default()
                                },
                                background: Some(divider_color),
                                ..Default::default()
                            },
                            |_cx| Vec::<AnyElement>::new(),
                        );
                        if let Some(test_id) = divider_test_id.clone() {
                            divider = divider.test_id(test_id);
                        }

                        let mut body = cx.container(
                            ContainerProps {
                                layout: {
                                    let mut layout = fret_ui::element::LayoutStyle::default();
                                    layout.size.width = Length::Fill;
                                    layout.size.height = Length::Fill;
                                    layout.flex.grow = 1.0;
                                    layout
                                },
                                padding: body_padding.into(),
                                ..Default::default()
                            },
                            content,
                        );
                        if let Some(test_id) = body_test_id.clone() {
                            body = body.test_id(test_id);
                        }

                        vec![cx.flex(column, move |_cx| vec![header_slot, divider, body])]
                    });

                    let sem = SemanticsProps {
                        role: SemanticsRole::Dialog,
                        test_id: overlay_test_id.clone(),
                        labelled_by_element,
                        ..Default::default()
                    };
                    let panel = cx.semantics_with_id(sem, move |_cx, panel_id| {
                        controlled_element_id_out.set(Some(panel_id));
                        vec![panel]
                    });

                    let trapped = focus_scope_prim::focus_trap(cx, move |_cx| vec![panel]);

                    fret_ui_kit::declarative::overlay_motion::wrap_opacity_and_render_transform_gated(
                        cx,
                        motion.content_alpha,
                        full_screen_transform,
                        overlay_presence.interactive,
                        vec![trapped],
                    )
                });

                let overlay_id = cx.root_id();
                let mut request = overlay_controller::OverlayRequest::modal(
                    overlay_id,
                    Some(input_id),
                    self.open.clone(),
                    overlay_presence,
                    vec![overlay_root],
                );
                request.root_name = Some(format!("material3.search_view.full_screen.{}", input_id.0));
                request.close_on_window_focus_lost = true;
                request.close_on_window_resize = true;
                request.initial_focus = initial_focus;
                request.on_close_auto_focus = Some(Arc::new(|_host, _cx, request| {
                    request.prevent_default();
                }));

                OverlayController::request(cx, request);

                return bar;
            }

            let Some(anchor) = fret_ui_kit::overlay::anchor_bounds_for_element(cx, input_id) else {
                return bar;
            };

            let outer = fret_ui_kit::overlay::outer_bounds_with_window_margin_for_environment(
                cx,
                fret_ui::Invalidation::Layout,
                self.window_margin,
            );

            // Prefer a stable, scrollable max-height over intrinsic measurement for this MVP.
            let animated_height = (self.max_height.0 * motion.progress).max(1.0);
            let desired = fret_core::Size::new(anchor.size.width, Px(animated_height));

            let direction = material_layout_direction_in_scope(cx);
            let placement = fret_ui_kit::primitives::popper::PopperContentPlacement::new(
                direction,
                fret_ui_kit::primitives::popper::Side::Bottom,
                fret_ui_kit::primitives::popper::Align::Start,
                Px(0.0),
            )
            .with_collision_padding({
                let theme = Theme::global(&*cx.app);
                dropdown_menu_tokens::collision_padding(theme)
            });

            let layout = fret_ui_kit::primitives::popper::popper_content_layout_sized(
                outer, anchor, desired, placement,
            );

            let overlay_rect = layout.rect;
            let (container_color, container_shape, shadow, divider_color, body_padding) = {
                let theme = Theme::global(&*cx.app);
                let states = WidgetStates::empty();
                let container_color = search_view_color_override(
                    theme,
                    &self.style.container_background,
                    states,
                    || search_view_tokens::container_color(theme),
                );
                let container_shape = search_view_corners_override(
                    &self.style.docked_container_corner_radii,
                    states,
                    || search_view_tokens::docked_container_shape(theme),
                );
                let elevation =
                    search_view_metric_override(&self.style.container_elevation, states, || {
                        search_view_tokens::container_elevation(theme)
                    });
                let shadow =
                    shadow_for_elevation_with_color(theme, elevation, None, container_shape);
                let divider_color =
                    search_view_color_override(theme, &self.style.divider_color, states, || {
                        search_view_tokens::divider_color(theme)
                    });
                let body_padding = search_view_edges_override(&self.style.body_padding, states, || {
                    Edges::all(Px(8.0))
                });
                (
                    container_color,
                    container_shape,
                    shadow,
                    divider_color,
                    body_padding,
                )
            };

            let overlay_test_id = self
                .overlay_test_id
                .clone()
                .or_else(|| root_test_id.as_ref().map(|id| part_test_id(id, "overlay")));
            let divider_test_id = root_test_id
                .as_ref()
                .map(|id| part_test_id(id, "overlay.divider"));
            let body_test_id = root_test_id
                .as_ref()
                .map(|id| part_test_id(id, "overlay.body"));
            let labelled_by_element = Some(input_id.0);
            let controlled_element_id_out = controlled_element_id.clone();
            let overlay_panel = fret_ui_kit::primitives::popper_content::popper_wrapper_panel_at(
                cx,
                overlay_rect,
                Edges::all(Px(0.0)),
                Overflow::Visible,
                move |cx| {
                    cx.provide(direction, |cx| {
                        let mut container = ContainerProps::default();
                        container.layout.size.width = Length::Fill;
                        container.layout.size.height = Length::Fill;
                        container.layout.overflow = Overflow::Clip;
                        container.background = Some(container_color);
                        container.corner_radii = container_shape;
                        container.shadow = shadow;

                        let panel = cx.container(container, move |cx| {
                            let mut divider = cx.container(
                                ContainerProps {
                                    layout: fret_ui::element::LayoutStyle {
                                        size: fret_ui::element::SizeStyle {
                                            width: Length::Fill,
                                            height: Length::Px(Px(1.0)),
                                            ..Default::default()
                                        },
                                        ..Default::default()
                                    },
                                    background: Some(divider_color),
                                    ..Default::default()
                                },
                                |_cx| Vec::<AnyElement>::new(),
                            );
                            if let Some(test_id) = divider_test_id.clone() {
                                divider = divider.test_id(test_id);
                            }

                            let mut body = cx.container(
                                ContainerProps {
                                    padding: body_padding.into(),
                                    ..Default::default()
                                },
                                content,
                            );
                            if let Some(test_id) = body_test_id.clone() {
                                body = body.test_id(test_id);
                            }

                            vec![divider, body]
                        });

                        let sem = SemanticsProps {
                            role: SemanticsRole::Panel,
                            test_id: overlay_test_id.clone(),
                            labelled_by_element,
                            ..Default::default()
                        };
                        let panel = cx.semantics_with_id(sem, move |_cx, panel_id| {
                            controlled_element_id_out.set(Some(panel_id));
                            vec![panel]
                        });

                        vec![panel]
                    })
                },
            );

            let opacity = motion.content_alpha;
            let transform = fret_core::Transform2D::IDENTITY;
            let overlay_root =
                fret_ui_kit::declarative::overlay_motion::wrap_opacity_and_render_transform_gated(
                    cx,
                    opacity,
                    transform,
                    overlay_presence.interactive,
                    vec![overlay_panel],
                );

            let mut request = fret_ui_kit::overlay_controller::OverlayRequest::dismissible_popover(
                input_id,
                input_id,
                self.open.clone(),
                overlay_presence,
                vec![overlay_root],
            );
            request.root_name = Some(format!("material3.search_view.{}", input_id.0));
            request.initial_focus = Some(input_id);
            request.close_on_window_focus_lost = true;
            request.close_on_window_resize = true;
            request = request.add_dismissable_branch(input_id);

            OverlayController::request(cx, request);

            bar
        })
    }
}

#[cfg(test)]
mod tests {
    use fret_app::App;
    use fret_core::{AppWindowId, Point, Rect, Size};
    use fret_ui::elements::with_element_cx;
    use fret_ui_kit::declarative::model_watch::ModelWatchExt as _;

    use super::*;

    fn bounds() -> Rect {
        Rect::new(
            Point::new(Px(0.0), Px(0.0)),
            Size::new(Px(200.0), Px(120.0)),
        )
    }

    #[test]
    fn search_view_new_controllable_uses_controlled_models_when_provided() {
        let window = AppWindowId::default();
        let mut app = App::new();
        let controlled_open = app.models_mut().insert(true);
        let controlled_query = app.models_mut().insert(String::from("alpha"));

        with_element_cx(
            &mut app,
            window,
            bounds(),
            "material3-search-view-controlled",
            |cx| {
                let view = SearchView::new_controllable(
                    cx,
                    Some(controlled_open.clone()),
                    false,
                    Some(controlled_query.clone()),
                    "",
                );
                assert_eq!(view.open_model(), controlled_open);
                assert_eq!(view.query_model(), controlled_query);
            },
        );
    }

    #[test]
    fn search_view_new_controllable_applies_default_open_and_query() {
        let window = AppWindowId::default();
        let mut app = App::new();

        with_element_cx(
            &mut app,
            window,
            bounds(),
            "material3-search-view-defaults",
            |cx| {
                let view =
                    SearchView::new_controllable(cx, None, true, None, String::from("hello"));
                let open = cx
                    .watch_model(&view.open_model())
                    .layout()
                    .copied()
                    .unwrap_or(false);
                let query = cx
                    .watch_model(&view.query_model())
                    .layout()
                    .cloned()
                    .unwrap_or_default();
                assert!(open);
                assert_eq!(query, "hello");
            },
        );
    }

    #[test]
    fn search_view_uncontrolled_multiple_instances_do_not_share_open_or_query_models() {
        let window = AppWindowId::default();
        let mut app = App::new();

        with_element_cx(
            &mut app,
            window,
            bounds(),
            "material3-search-view-uncontrolled-scope",
            |cx| {
                let a = SearchView::uncontrolled(cx);
                let b = SearchView::uncontrolled(cx);
                assert_ne!(a.open_model(), b.open_model());
                assert_ne!(a.query_model(), b.query_model());
            },
        );
    }
}
