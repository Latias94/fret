//! Editor theme preset picker.
//!
//! This is an editor-policy control, not a runtime theme mechanism. It switches between
//! editor-owned preset patches and keeps Dear ImGui-style tuning in the ecosystem layer.

use std::sync::Arc;

use fret_core::{Axis, Color, Corners, Edges, Px, SemanticsRole};
use fret_runtime::Model;
use fret_ui::action::{ActivateReason, OnActivate};
use fret_ui::element::{
    AnyElement, ContainerProps, CrossAlign, FlexProps, LayoutStyle, Length, MainAlign,
    PressableA11y, PressableProps, RingPlacement, RingStyle, SemanticsProps, SizeStyle,
    SpacingLength,
};
use fret_ui::{ElementContext, Invalidation, Theme, UiHost};

use crate::primitives::colors::{
    editor_accent, editor_border, editor_focus_ring, editor_foreground, editor_muted_foreground,
    editor_subtle_bg,
};
use crate::primitives::input_group::derived_test_id;
use crate::primitives::readout::{
    editor_theme_preset_picker_header_text_props, editor_theme_preset_picker_row_label_text_props,
    editor_theme_preset_picker_row_status_text_props,
};
use crate::primitives::style::EditorStyle;
use crate::theme::{
    EDITOR_THEME_PRESETS_V1, EditorThemePresetV1, install_editor_theme_preset_v1,
    installed_editor_theme_preset_v1,
};

mod options;

pub use options::EditorThemePresetPickerOptions;

#[derive(Clone)]
pub struct EditorThemePresetPicker {
    model: Model<EditorThemePresetV1>,
    options: EditorThemePresetPickerOptions,
}

impl EditorThemePresetPicker {
    pub fn new(model: Model<EditorThemePresetV1>) -> Self {
        Self {
            model,
            options: EditorThemePresetPickerOptions::default(),
        }
    }

    pub fn options(mut self, options: EditorThemePresetPickerOptions) -> Self {
        self.options = options;
        self
    }

    #[track_caller]
    pub fn into_element<H: UiHost>(self, cx: &mut ElementContext<'_, H>) -> AnyElement {
        let selected = cx
            .get_model_copied(&self.model, Invalidation::Paint)
            .unwrap_or_default();

        if installed_editor_theme_preset_v1(&*cx.app) != Some(selected) {
            install_editor_theme_preset_v1(cx.app, selected);
        }

        let (density, row_height, border, ring, fg, muted_fg, subtle_bg, accent, text_px) = {
            let theme = Theme::global(&*cx.app);
            let style = EditorStyle::resolve(theme);
            let chrome = style.frame_chrome_small();
            (
                style.density,
                style.density.row_height,
                editor_border(theme),
                editor_focus_ring(theme),
                editor_foreground(theme),
                editor_muted_foreground(theme),
                editor_subtle_bg(theme),
                editor_accent(theme),
                chrome.text_px,
            )
        };

        let label = self
            .options
            .label
            .clone()
            .unwrap_or_else(|| Arc::from("Editor theme preset"));
        let item_prefix = self
            .options
            .item_test_id_prefix
            .clone()
            .or_else(|| derived_test_id(self.options.test_id.as_ref(), "item"));
        let options = self.options.clone();
        let model = self.model.clone();
        let total = EDITOR_THEME_PRESETS_V1.len();

        cx.semantics(
            SemanticsProps {
                layout: options.layout,
                role: SemanticsRole::ListBox,
                label: Some(label.clone()),
                test_id: options.test_id.clone(),
                ..Default::default()
            },
            move |cx| {
                let mut rows = Vec::with_capacity(total + 1);
                rows.push(header_text(cx, label.clone(), muted_fg, text_px));
                rows.extend(EDITOR_THEME_PRESETS_V1.iter().copied().enumerate().map(
                    |(index, preset)| {
                        preset_row(
                            cx,
                            model.clone(),
                            preset,
                            selected == preset,
                            index,
                            total,
                            item_prefix.clone(),
                            options.enabled,
                            options.focusable,
                            row_height,
                            density.padding_x,
                            border,
                            ring,
                            fg,
                            muted_fg,
                            subtle_bg,
                            accent,
                            text_px,
                        )
                    },
                ));

                vec![cx.container(
                    ContainerProps {
                        layout: LayoutStyle {
                            size: SizeStyle {
                                width: Length::Fill,
                                height: Length::Auto,
                                ..Default::default()
                            },
                            ..Default::default()
                        },
                        padding: Edges::all(Px(3.0)).into(),
                        background: Some(subtle_bg),
                        border: Edges::all(Px(1.0)),
                        border_color: Some(border),
                        corner_radii: Corners::all(Px(4.0)),
                        ..Default::default()
                    },
                    move |cx| {
                        vec![cx.flex(
                            FlexProps {
                                layout: LayoutStyle {
                                    size: SizeStyle {
                                        width: Length::Fill,
                                        height: Length::Auto,
                                        ..Default::default()
                                    },
                                    ..Default::default()
                                },
                                direction: Axis::Vertical,
                                gap: SpacingLength::Px(Px(3.0)),
                                justify: MainAlign::Start,
                                align: CrossAlign::Stretch,
                                ..Default::default()
                            },
                            move |_cx| rows,
                        )]
                    },
                )]
            },
        )
    }
}

fn header_text<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    label: Arc<str>,
    color: Color,
    text_px: Px,
) -> AnyElement {
    cx.text_props(editor_theme_preset_picker_header_text_props(
        label, color, text_px,
    ))
}

#[allow(clippy::too_many_arguments)]
fn preset_row<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    model: Model<EditorThemePresetV1>,
    preset: EditorThemePresetV1,
    selected: bool,
    index: usize,
    total: usize,
    item_prefix: Option<Arc<str>>,
    enabled: bool,
    focusable: bool,
    row_height: Px,
    padding_x: Px,
    border: Color,
    ring: Color,
    fg: Color,
    muted_fg: Color,
    subtle_bg: Color,
    accent: Color,
    text_px: Px,
) -> AnyElement {
    let item_test_id = item_prefix
        .as_ref()
        .map(|prefix| Arc::<str>::from(format!("{prefix}.{}", preset.key())));
    let label = Arc::<str>::from(preset.label());
    let model_for_activate = model.clone();

    let mut row = cx.pressable(
        PressableProps {
            layout: LayoutStyle {
                size: SizeStyle {
                    width: Length::Fill,
                    height: Length::Px(row_height),
                    ..Default::default()
                },
                ..Default::default()
            },
            enabled,
            focusable,
            focus_ring: Some(RingStyle {
                placement: RingPlacement::Outset,
                width: Px(1.0),
                offset: Px(1.0),
                color: ring,
                offset_color: None,
                corner_radii: Corners::all(Px(3.0)),
            }),
            a11y: PressableA11y {
                role: Some(SemanticsRole::ListBoxOption),
                label: Some(label.clone()),
                test_id: item_test_id.clone(),
                selected,
                pos_in_set: Some((index as u32) + 1),
                set_size: Some(total as u32),
                ..Default::default()
            },
            ..Default::default()
        },
        move |cx, state| {
            let on_activate: OnActivate =
                Arc::new(move |host, action_cx, _reason: ActivateReason| {
                    let _ = host
                        .models_mut()
                        .update(&model_for_activate, |value| *value = preset);
                    host.request_redraw(action_cx.window);
                });
            cx.pressable_add_on_activate(on_activate);

            let active_bg = mix_color(subtle_bg, accent, 0.42);
            let hover_bg = mix_color(subtle_bg, accent, 0.18);
            let pressed_bg = mix_color(subtle_bg, accent, 0.32);
            let background = if selected {
                active_bg
            } else if state.pressed {
                pressed_bg
            } else if state.hovered || state.hovered_raw {
                hover_bg
            } else {
                subtle_bg
            };
            let text_color = if enabled {
                fg
            } else {
                mix_color(muted_fg, subtle_bg, 0.35)
            };
            let border_color = if selected { accent } else { border };
            let check_text = if selected { "On" } else { "" };

            vec![cx.container(
                ContainerProps {
                    layout: LayoutStyle {
                        size: SizeStyle {
                            width: Length::Fill,
                            height: Length::Fill,
                            ..Default::default()
                        },
                        ..Default::default()
                    },
                    padding: Edges::symmetric(padding_x, Px(0.0)).into(),
                    background: Some(background),
                    border: Edges::all(Px(1.0)),
                    border_color: Some(border_color),
                    corner_radii: Corners::all(Px(3.0)),
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
                            gap: SpacingLength::Px(Px(8.0)),
                            justify: MainAlign::Start,
                            align: CrossAlign::Center,
                            ..Default::default()
                        },
                        move |cx| {
                            vec![
                                cx.text_props(editor_theme_preset_picker_row_label_text_props(
                                    label.clone(),
                                    text_color,
                                    row_height,
                                    text_px,
                                )),
                                cx.text_props(editor_theme_preset_picker_row_status_text_props(
                                    Arc::from(check_text),
                                    muted_fg,
                                    row_height,
                                    text_px,
                                )),
                            ]
                        },
                    )]
                },
            )]
        },
    );

    if let Some(test_id) = item_test_id {
        row = row.test_id(test_id);
    }

    row
}

fn mix_color(a: Color, b: Color, t: f32) -> Color {
    let t = t.clamp(0.0, 1.0);
    Color {
        r: a.r + (b.r - a.r) * t,
        g: a.g + (b.g - a.g) * t,
        b: a.b + (b.b - a.b) * t,
        a: a.a + (b.a - a.a) * t,
    }
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use fret_app::App;
    use fret_core::{
        AppWindowId, Event, Modifiers, MouseButton, Point, PointerEvent, PointerId, PointerType,
        Px, Rect, SemanticsRole, SemanticsSnapshot, Size,
    };
    use fret_runtime::Model;
    use fret_ui::{Theme, UiTree, declarative};

    use super::{EditorThemePresetPicker, EditorThemePresetPickerOptions};
    use crate::primitives::EditorTokenKeys;
    use crate::test_support::WrappingTextServices;
    use crate::theme::{
        EditorThemePresetV1, installed_editor_theme_preset_v1,
        reapply_installed_editor_theme_preset_v1,
    };

    #[test]
    fn editor_theme_preset_picker_stamps_listbox_options_and_selected_state() {
        let window = AppWindowId::default();
        let bounds = test_bounds();
        let mut app = App::new();
        let mut services = WrappingTextServices;
        let mut ui: UiTree<App> = UiTree::new();
        ui.set_window(window);
        let model = app.models_mut().insert(EditorThemePresetV1::ImguiLikeDense);

        render_picker_frame(&mut ui, &mut app, &mut services, window, bounds, &model);
        ui.request_semantics_snapshot();
        ui.layout_all(&mut app, &mut services, bounds, 1.0);
        let snapshot = ui.semantics_snapshot().expect("semantics snapshot");

        let listbox = node_by_test_id(snapshot, "tests.theme_preset");
        assert_eq!(listbox.role, SemanticsRole::ListBox);
        assert_eq!(listbox.label.as_deref(), Some("Editor theme preset"));

        let default = node_by_test_id(snapshot, "tests.theme_preset.item.default");
        assert_eq!(default.role, SemanticsRole::ListBoxOption);
        assert_eq!(default.label.as_deref(), Some("Default"));
        assert_eq!(default.pos_in_set, Some(1));
        assert_eq!(default.set_size, Some(2));
        assert!(!default.flags.selected);

        let dense = node_by_test_id(snapshot, "tests.theme_preset.item.imgui_like_dense");
        assert_eq!(dense.role, SemanticsRole::ListBoxOption);
        assert_eq!(dense.label.as_deref(), Some("ImGui-like dense"));
        assert_eq!(dense.pos_in_set, Some(2));
        assert_eq!(dense.set_size, Some(2));
        assert!(dense.flags.selected);
    }

    #[test]
    fn editor_theme_preset_picker_click_updates_model_and_replays_reversible_preset() {
        let window = AppWindowId::default();
        let bounds = test_bounds();
        let mut app = App::new();
        let mut services = WrappingTextServices;
        let mut ui: UiTree<App> = UiTree::new();
        ui.set_window(window);
        let model = app.models_mut().insert(EditorThemePresetV1::Default);

        render_picker_frame(&mut ui, &mut app, &mut services, window, bounds, &model);
        pump_semantics(&mut ui, &mut app, &mut services, bounds);
        assert_eq!(
            installed_editor_theme_preset_v1(&app),
            Some(EditorThemePresetV1::Default)
        );

        click_test_id(
            &mut ui,
            &mut app,
            &mut services,
            "tests.theme_preset.item.imgui_like_dense",
        );
        assert_eq!(
            app.models().get_copied(&model),
            Some(EditorThemePresetV1::ImguiLikeDense)
        );

        render_picker_frame(&mut ui, &mut app, &mut services, window, bounds, &model);
        pump_semantics(&mut ui, &mut app, &mut services, bounds);
        assert_eq!(
            installed_editor_theme_preset_v1(&app),
            Some(EditorThemePresetV1::ImguiLikeDense)
        );
        assert_eq!(
            Theme::global(&app).metric_by_key(EditorTokenKeys::DENSITY_ROW_HEIGHT),
            Some(Px(22.0))
        );
        assert_eq!(
            Theme::global(&app).metric_by_key(EditorTokenKeys::NUMERIC_SCRUB_DRAG_THRESHOLD),
            Some(Px(2.0))
        );

        click_test_id(
            &mut ui,
            &mut app,
            &mut services,
            "tests.theme_preset.item.default",
        );
        assert_eq!(
            app.models().get_copied(&model),
            Some(EditorThemePresetV1::Default)
        );

        render_picker_frame(&mut ui, &mut app, &mut services, window, bounds, &model);
        pump_semantics(&mut ui, &mut app, &mut services, bounds);
        assert_eq!(
            reapply_installed_editor_theme_preset_v1(&mut app),
            Some(EditorThemePresetV1::Default)
        );
        assert_eq!(
            Theme::global(&app).metric_by_key(EditorTokenKeys::DENSITY_ROW_HEIGHT),
            Some(Px(24.0))
        );
        assert_eq!(
            Theme::global(&app).metric_by_key(EditorTokenKeys::NUMERIC_SCRUB_SPEED),
            Some(Px(0.02))
        );
        assert_eq!(
            Theme::global(&app).metric_by_key(EditorTokenKeys::NUMERIC_SCRUB_DRAG_THRESHOLD),
            Some(Px(4.0))
        );
    }

    fn test_bounds() -> Rect {
        Rect::new(
            Point::new(Px(0.0), Px(0.0)),
            Size::new(Px(260.0), Px(120.0)),
        )
    }

    fn render_picker_frame(
        ui: &mut UiTree<App>,
        app: &mut App,
        services: &mut WrappingTextServices,
        window: AppWindowId,
        bounds: Rect,
        model: &Model<EditorThemePresetV1>,
    ) {
        let model = model.clone();
        let root = declarative::render_root(
            ui,
            app,
            services,
            window,
            bounds,
            "editor-theme-preset-picker-test",
            move |cx| {
                vec![
                    EditorThemePresetPicker::new(model.clone())
                        .options(EditorThemePresetPickerOptions {
                            test_id: Some(Arc::from("tests.theme_preset")),
                            ..Default::default()
                        })
                        .into_element(cx),
                ]
            },
        );
        ui.set_root(root);
    }

    fn pump_semantics(
        ui: &mut UiTree<App>,
        app: &mut App,
        services: &mut WrappingTextServices,
        bounds: Rect,
    ) {
        ui.request_semantics_snapshot();
        ui.layout_all(app, services, bounds, 1.0);
        let mut scene = fret_core::Scene::default();
        ui.paint_all(app, services, bounds, &mut scene, 1.0);
    }

    fn click_test_id(
        ui: &mut UiTree<App>,
        app: &mut App,
        services: &mut WrappingTextServices,
        test_id: &str,
    ) {
        let snapshot = ui.semantics_snapshot().expect("semantics snapshot");
        let node = node_by_test_id(snapshot, test_id);
        let bounds = node.bounds;
        let position = Point::new(
            Px(bounds.origin.x.0 + bounds.size.width.0 * 0.5),
            Px(bounds.origin.y.0 + bounds.size.height.0 * 0.5),
        );

        ui.dispatch_event(
            app,
            services,
            &Event::Pointer(PointerEvent::Down {
                position,
                button: MouseButton::Left,
                modifiers: Modifiers::default(),
                click_count: 1,
                pointer_id: PointerId(0),
                pointer_type: PointerType::Mouse,
            }),
        );
        ui.dispatch_event(
            app,
            services,
            &Event::Pointer(PointerEvent::Up {
                position,
                button: MouseButton::Left,
                modifiers: Modifiers::default(),
                click_count: 1,
                is_click: true,
                pointer_id: PointerId(0),
                pointer_type: PointerType::Mouse,
            }),
        );
    }

    fn node_by_test_id<'a>(
        snapshot: &'a SemanticsSnapshot,
        test_id: &str,
    ) -> &'a fret_core::SemanticsNode {
        snapshot
            .nodes
            .iter()
            .find(|node| node.test_id.as_deref() == Some(test_id))
            .unwrap_or_else(|| panic!("expected semantics node `{test_id}`"))
    }
}
