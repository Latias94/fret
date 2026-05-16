//! Inspector-style property grid (two-column rows).
//!
//! This is intentionally a thin composition layer on top of `PropertyRow`:
//! - the grid resolves shared policies (label width, gaps, density defaults),
//! - individual rows remain fully composable and can opt into reset/actions slots.

use std::sync::Arc;

use fret_core::{Axis, Edges, Px};
use fret_ui::element::{
    AnyElement, CrossAlign, FlexProps, LayoutStyle, Length, MainAlign, SizeStyle, SpacingLength,
};
use fret_ui::{ElementContext, ElementContextAccess, Theme, UiHost};

use crate::composites::property_row::{PropertyRow, property_row_label_text};
use crate::composites::property_row::{PropertyRowLayoutVariant, PropertyRowOptions};
use crate::primitives::EditorDensity;
use crate::primitives::inspector_layout::InspectorLayoutMetrics;

#[derive(Debug, Clone)]
pub struct PropertyGridOptions {
    pub layout: LayoutStyle,
    pub label_width: Option<Px>,
    pub column_gap: Option<Px>,
    pub row_gap: Option<Px>,
    pub test_id: Option<Arc<str>>,
}

impl Default for PropertyGridOptions {
    fn default() -> Self {
        Self {
            layout: LayoutStyle {
                size: SizeStyle {
                    width: Length::Fill,
                    height: Length::Auto,
                    ..Default::default()
                },
                ..Default::default()
            },
            label_width: None,
            column_gap: None,
            row_gap: None,
            test_id: None,
        }
    }
}

#[derive(Clone, Default)]
pub struct PropertyGrid {
    pub options: PropertyGridOptions,
}

impl PropertyGrid {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn options(mut self, options: PropertyGridOptions) -> Self {
        self.options = options;
        self
    }

    #[track_caller]
    pub fn into_element<H: UiHost>(
        self,
        cx: &mut ElementContext<'_, H>,
        rows: impl FnOnce(&mut ElementContext<'_, H>, PropertyGridRowCx) -> Vec<AnyElement>,
    ) -> AnyElement {
        let (density, row_opts, row_gap) = {
            let theme = Theme::global(&*cx.app);
            let metrics = InspectorLayoutMetrics::resolve(theme);
            let density = metrics.density;
            let column_gap = self.options.column_gap.unwrap_or(metrics.column_gap);
            let row_gap = self.options.row_gap.unwrap_or(metrics.row_gap);

            let row_opts = PropertyRowOptions {
                label_width: self.options.label_width,
                gap: Some(column_gap),
                trailing_gap: Some(metrics.trailing_gap),
                value_max_width: Some(metrics.value_max_width),
                status_slot_width: Some(metrics.status_slot_width),
                reset_slot_width: Some(metrics.reset_slot_width),
                variant: PropertyRowLayoutVariant::Auto,
                ..Default::default()
            };

            (density, row_opts, row_gap)
        };

        let row_cx = PropertyGridRowCx {
            density,
            row_options: row_opts,
        };

        let mut root = cx.flex(
            FlexProps {
                layout: self.options.layout,
                direction: Axis::Vertical,
                gap: SpacingLength::Px(row_gap),
                padding: Edges::all(Px(0.0)).into(),
                justify: MainAlign::Start,
                align: CrossAlign::Stretch,
                wrap: false,
            },
            move |cx| rows(cx, row_cx),
        );

        if let Some(test_id) = self.options.test_id.as_ref() {
            root = root.test_id(test_id.clone());
        }
        root
    }

    #[track_caller]
    pub fn into_element_in<'a, H: UiHost + 'a, Cx>(
        self,
        cx: &mut Cx,
        rows: impl FnOnce(&mut ElementContext<'_, H>, PropertyGridRowCx) -> Vec<AnyElement>,
    ) -> AnyElement
    where
        Cx: ElementContextAccess<'a, H>,
    {
        self.into_element(cx.elements(), rows)
    }
}

#[derive(Clone)]
pub struct PropertyGridRowCx {
    density: EditorDensity,
    row_options: PropertyRowOptions,
}

impl PropertyGridRowCx {
    pub fn density(&self) -> EditorDensity {
        self.density
    }

    pub(crate) fn row_options(&self) -> PropertyRowOptions {
        self.row_options.clone()
    }

    pub fn label_text<H: UiHost>(
        &self,
        cx: &mut ElementContext<'_, H>,
        text: impl Into<Arc<str>>,
    ) -> AnyElement {
        property_row_label_text(cx, text)
    }

    pub fn row<H: UiHost>(
        &self,
        cx: &mut ElementContext<'_, H>,
        label: impl FnOnce(&mut ElementContext<'_, H>) -> AnyElement,
        value: impl FnOnce(&mut ElementContext<'_, H>) -> AnyElement,
    ) -> AnyElement {
        self.row_with(cx, PropertyRow::new(), label, value, |_cx| None)
    }

    pub fn row_with<H: UiHost>(
        &self,
        cx: &mut ElementContext<'_, H>,
        row: PropertyRow,
        label: impl FnOnce(&mut ElementContext<'_, H>) -> AnyElement,
        value: impl FnOnce(&mut ElementContext<'_, H>) -> AnyElement,
        actions: impl FnOnce(&mut ElementContext<'_, H>) -> Option<AnyElement>,
    ) -> AnyElement {
        row.options(self.row_options())
            .into_element(cx, label, value, actions)
    }
}

#[cfg(test)]
mod tests {
    use std::sync::{Arc, Mutex};

    use fret_app::App;
    use fret_core::{AppWindowId, Color, Point, Px, Rect, Size, TextStyle};
    use fret_ui::elements::GlobalElementId;
    use fret_ui::{UiTree, declarative};

    use super::{PropertyGrid, PropertyGridOptions};
    use crate::composites::property_row::{
        PropertyRow, PropertyRowLayoutVariant, PropertyRowOptions,
    };
    use crate::primitives::readout::{
        editor_inline_error_text_props, editor_validation_message_text_props,
    };
    use crate::test_support::WrappingTextServices;

    fn bounds() -> Rect {
        Rect::new(
            Point::new(Px(0.0), Px(0.0)),
            Size::new(Px(180.0), Px(160.0)),
        )
    }

    fn current_bounds(
        app: &mut App,
        window: AppWindowId,
        id: GlobalElementId,
        label: &str,
    ) -> Rect {
        fret_ui::elements::current_bounds_for_element(app, window, id)
            .unwrap_or_else(|| panic!("{label} bounds"))
    }

    fn lock_id(id: &Arc<Mutex<Option<GlobalElementId>>>, label: &str) -> GlobalElementId {
        id.lock().unwrap().unwrap_or_else(|| panic!("{label} id"))
    }

    #[test]
    fn property_grid_keeps_rows_separated_when_value_text_wraps_under_narrow_layout() {
        let mut app = App::new();
        let mut ui: UiTree<App> = UiTree::new();
        let window = AppWindowId::default();
        ui.set_window(window);

        let mut services = WrappingTextServices;
        let grid_id = Arc::new(Mutex::new(None::<GlobalElementId>));
        let first_row_id = Arc::new(Mutex::new(None::<GlobalElementId>));
        let wrapping_row_id = Arc::new(Mutex::new(None::<GlobalElementId>));
        let validation_text_id = Arc::new(Mutex::new(None::<GlobalElementId>));
        let trailing_row_id = Arc::new(Mutex::new(None::<GlobalElementId>));

        let grid_id_for_render = Arc::clone(&grid_id);
        let first_row_id_for_render = Arc::clone(&first_row_id);
        let wrapping_row_id_for_render = Arc::clone(&wrapping_row_id);
        let validation_text_id_for_render = Arc::clone(&validation_text_id);
        let trailing_row_id_for_render = Arc::clone(&trailing_row_id);
        let root = declarative::render_root(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds(),
            "property-grid-wrapping-layout",
            move |cx| {
                let grid = PropertyGrid::new()
                    .options(PropertyGridOptions {
                        label_width: Some(Px(104.0)),
                        column_gap: Some(Px(8.0)),
                        row_gap: Some(Px(4.0)),
                        test_id: Some(Arc::from("inspector.grid")),
                        ..Default::default()
                    })
                    .into_element(cx, |cx, rows| {
                        let first = rows.row_with(
                            cx,
                            PropertyRow::new().options(PropertyRowOptions {
                                variant: PropertyRowLayoutVariant::Row,
                                test_id: Some(Arc::from("inspector.grid.exposure")),
                                ..Default::default()
                            }),
                            |cx| rows.label_text(cx, "Exposure"),
                            |cx| {
                                cx.text_props(editor_inline_error_text_props(
                                    Arc::from("0.50"),
                                    Color::from_srgb_hex_rgb(0xCC_CC_CC),
                                    Px(20.0),
                                ))
                            },
                            |_cx| None,
                        );
                        *first_row_id_for_render.lock().unwrap() = Some(first.id);

                        let wrapping = rows.row_with(
                            cx,
                            PropertyRow::new().options(PropertyRowOptions {
                                variant: PropertyRowLayoutVariant::Row,
                                test_id: Some(Arc::from("inspector.grid.validation")),
                                ..Default::default()
                            }),
                            |cx| rows.label_text(cx, "Validation"),
                            |cx| {
                                let text = cx.text_props(editor_validation_message_text_props(
                                    Arc::from(
                                        "Value must stay between 0.0 and 1.0 for this render target.",
                                    ),
                                    Color::from_srgb_hex_rgb(0xCC_44_44),
                                    TextStyle::default(),
                                ));
                                *validation_text_id_for_render.lock().unwrap() = Some(text.id);
                                text
                            },
                            |_cx| None,
                        );
                        *wrapping_row_id_for_render.lock().unwrap() = Some(wrapping.id);

                        let trailing = rows.row_with(
                            cx,
                            PropertyRow::new().options(PropertyRowOptions {
                                variant: PropertyRowLayoutVariant::Row,
                                test_id: Some(Arc::from("inspector.grid.roughness")),
                                ..Default::default()
                            }),
                            |cx| rows.label_text(cx, "Roughness"),
                            |cx| {
                                cx.text_props(editor_inline_error_text_props(
                                    Arc::from("0.25"),
                                    Color::from_srgb_hex_rgb(0xCC_CC_CC),
                                    Px(20.0),
                                ))
                            },
                            |_cx| None,
                        );
                        *trailing_row_id_for_render.lock().unwrap() = Some(trailing.id);

                        vec![first, wrapping, trailing]
                    });
                *grid_id_for_render.lock().unwrap() = Some(grid.id);
                vec![grid]
            },
        );
        ui.set_root(root);
        ui.layout_all(&mut app, &mut services, bounds(), 1.0);

        let grid_bounds = current_bounds(&mut app, window, lock_id(&grid_id, "grid"), "grid");
        let first_bounds = current_bounds(
            &mut app,
            window,
            lock_id(&first_row_id, "first row"),
            "first row",
        );
        let wrapping_bounds = current_bounds(
            &mut app,
            window,
            lock_id(&wrapping_row_id, "wrapping row"),
            "wrapping row",
        );
        let validation_bounds = current_bounds(
            &mut app,
            window,
            lock_id(&validation_text_id, "validation text"),
            "validation text",
        );
        let trailing_bounds = current_bounds(
            &mut app,
            window,
            lock_id(&trailing_row_id, "trailing row"),
            "trailing row",
        );

        assert!(
            validation_bounds.size.height.0 > 28.0,
            "validation text should wrap to multiple measured lines under narrow grid layout: {validation_bounds:?}"
        );
        assert!(
            wrapping_bounds.size.height.0 + 0.5 >= validation_bounds.size.height.0,
            "wrapping property row should grow to contain validation text: row={wrapping_bounds:?} text={validation_bounds:?}"
        );
        assert!(
            first_bounds.origin.y.0 + first_bounds.size.height.0
                <= wrapping_bounds.origin.y.0 + 0.5,
            "first row should not overlap wrapping row: first={first_bounds:?} wrapping={wrapping_bounds:?}"
        );
        assert!(
            wrapping_bounds.origin.y.0 + wrapping_bounds.size.height.0
                <= trailing_bounds.origin.y.0 + 0.5,
            "wrapping row should push the following row down: wrapping={wrapping_bounds:?} trailing={trailing_bounds:?}"
        );
        assert!(
            trailing_bounds.origin.y.0 + trailing_bounds.size.height.0
                <= grid_bounds.origin.y.0 + grid_bounds.size.height.0 + 0.5,
            "property grid should contain rows after a wrapping value row: grid={grid_bounds:?} trailing={trailing_bounds:?}"
        );
    }
}
