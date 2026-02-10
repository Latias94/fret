//! Material 3 search view (docked, MVP).
//!
//! This component composes a `SearchBar`-like input surface with a dismissible popover panel that
//! can host arbitrary results/suggestions content.

use std::cell::Cell;
use std::rc::Rc;
use std::sync::Arc;

use fret_core::{Edges, Px, SemanticsRole};
use fret_icons::IconId;
use fret_runtime::Model;
use fret_ui::element::{AnyElement, ContainerProps, Length, Overflow, SemanticsDecoration};
use fret_ui::elements::ElementContext;
use fret_ui::{GlobalElementId, Theme, UiHost};
use fret_ui_kit::{OverlayController, OverlayPresence};

use crate::SearchBar;
use crate::foundation::elevation::shadow_for_elevation_with_color;
use crate::foundation::overlay_motion::drive_overlay_open_close_motion;
use crate::search_bar::SearchBarHeaderTokens;
use crate::tokens::{dropdown_menu as dropdown_menu_tokens, search_view as search_view_tokens};

#[derive(Debug, Clone)]
pub struct SearchView {
    open: Model<bool>,
    query: Model<String>,
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

    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
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

    pub fn into_element<H: UiHost>(
        self,
        cx: &mut ElementContext<'_, H>,
        content: impl FnOnce(&mut ElementContext<'_, H>) -> Vec<AnyElement>,
    ) -> AnyElement {
        cx.scope(|cx| {
            let input_id_out: Rc<Cell<Option<GlobalElementId>>> = Rc::new(Cell::new(None));
            let input_id_out_for_bar = input_id_out.clone();

            // Keep the input surface in the underlay so:
            // - focus stays on the text input while the overlay is open (Compose-like),
            // - the overlay can be dismissed without fighting focus-gained/blur heuristics.
            let mut bar = SearchBar::new(self.query.clone())
                .disabled(self.disabled)
                .placeholder_opt(self.placeholder.clone())
                .a11y_label_opt(self.a11y_label.clone())
                .test_id_opt(self.test_id.clone())
                .expanded_model(self.open.clone())
                .header_tokens(SearchBarHeaderTokens::SearchView)
                .input_id_out(input_id_out_for_bar);
            if let Some(icon) = self.leading_icon.as_ref() {
                bar = bar.leading_icon(icon.clone());
            }
            if let Some(icon) = self.trailing_icon.as_ref() {
                bar = bar.trailing_icon(icon.clone());
            }
            let bar = bar.into_element(cx);

            // Policy: open on focus gained (Compose-like), while keeping focus on the text input.
            if !self.disabled {
                if let Some(input_id) = input_id_out.get() {
                    let focused_input = cx.is_focused_element(input_id);

                    #[derive(Default)]
                    struct FrameState {
                        was_focused_input: bool,
                    }

                    let focus_gained = cx.with_state(FrameState::default, |st| {
                        let focus_gained = focused_input && !st.was_focused_input;
                        st.was_focused_input = focused_input;
                        focus_gained
                    });

                    if focus_gained {
                        let _ = cx.app.models_mut().update(&self.open, |v| *v = true);
                        cx.app.request_redraw(cx.window);
                    }
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
            let motion = drive_overlay_open_close_motion(cx, is_open, close_grace_frames);
            let overlay_presence = OverlayPresence {
                present: motion.present,
                interactive: is_open,
            };

            if !overlay_presence.present {
                return bar;
            }

            let Some(input_id) = input_id_out.get() else {
                return bar;
            };
            let Some(anchor) = fret_ui_kit::overlay::anchor_bounds_for_element(cx, input_id) else {
                return bar;
            };

            let outer = fret_ui_kit::overlay::outer_bounds_with_window_margin(
                cx.bounds,
                self.window_margin,
            );

            // Prefer a stable, scrollable max-height over intrinsic measurement for this MVP.
            let desired = fret_core::Size::new(anchor.size.width, self.max_height);

            let direction = fret_ui_kit::primitives::direction::use_direction_in_scope(cx, None);
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
            let (container_color, container_shape, shadow, divider_color) = {
                let theme = Theme::global(&*cx.app);
                let container_color = search_view_tokens::container_color(theme);
                let container_shape = search_view_tokens::docked_container_shape(theme);
                let elevation = search_view_tokens::container_elevation(theme);
                let shadow =
                    shadow_for_elevation_with_color(theme, elevation, None, container_shape);
                let divider_color = search_view_tokens::divider_color(theme);
                (container_color, container_shape, shadow, divider_color)
            };

            let overlay_test_id = self.overlay_test_id.clone();
            let overlay_panel = fret_ui_kit::primitives::popper_content::popper_wrapper_panel_at(
                cx,
                overlay_rect,
                Edges::all(Px(0.0)),
                Overflow::Visible,
                move |cx| {
                    let mut container = ContainerProps::default();
                    container.layout.size.width = Length::Fill;
                    container.layout.size.height = Length::Fill;
                    container.layout.overflow = Overflow::Clip;
                    container.background = Some(container_color);
                    container.corner_radii = container_shape;
                    container.shadow = shadow;

                    let panel = cx.container(container, move |cx| {
                        let divider = cx.container(
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

                        let body = cx.container(
                            ContainerProps {
                                padding: Edges::all(Px(8.0)),
                                ..Default::default()
                            },
                            content,
                        );

                        vec![divider, body]
                    });

                    let panel = if let Some(test_id) = overlay_test_id.as_ref() {
                        panel.attach_semantics(
                            SemanticsDecoration::default()
                                .role(SemanticsRole::Panel)
                                .test_id(test_id.clone()),
                        )
                    } else {
                        panel
                    };

                    vec![panel]
                },
            );

            let opacity = motion.alpha;
            let scale = motion.scale;
            let origin = fret_ui_kit::primitives::popper::popper_content_transform_origin(
                &layout, anchor, None,
            );
            let origin_inv = fret_core::Point::new(Px(-origin.x.0), Px(-origin.y.0));
            let transform = fret_core::Transform2D::translation(origin)
                * fret_core::Transform2D::scale_uniform(scale)
                * fret_core::Transform2D::translation(origin_inv);
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
            request.close_on_window_focus_lost = true;
            request.close_on_window_resize = true;
            request = request.add_dismissable_branch(input_id);

            OverlayController::request(cx, request);

            bar
        })
    }
}
