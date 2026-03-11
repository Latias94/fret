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
use fret_ui::{ElementContext, Theme, UiHost};

use crate::composites::property_row::PropertyRow;
use crate::composites::property_row::{
    PropertyRowLayoutVariant, PropertyRowOptions, PropertyRowReset,
};
use crate::primitives::{EditorDensity, EditorTokenKeys};

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
            let density = EditorDensity::resolve(theme);
            let column_gap = self
                .options
                .column_gap
                .or_else(|| theme.metric_by_key(EditorTokenKeys::PROPERTY_COLUMN_GAP))
                .unwrap_or(Px(8.0));
            let row_gap = self
                .options
                .row_gap
                .or_else(|| theme.metric_by_key(EditorTokenKeys::PROPERTY_ROW_GAP))
                .unwrap_or(Px(4.0));

            let row_opts = PropertyRowOptions {
                label_width: self.options.label_width,
                gap: Some(column_gap),
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
}

#[derive(Clone)]
pub struct PropertyGridRowCx {
    pub density: EditorDensity,
    pub row_options: PropertyRowOptions,
}

impl PropertyGridRowCx {
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
        row.options(self.row_options.clone())
            .into_element(cx, label, value, actions)
    }
}

#[derive(Clone, Default)]
pub struct PropertyGridRow {
    pub options: Option<PropertyRowOptions>,
    pub reset: Option<PropertyRowReset>,
}

impl PropertyGridRow {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn options(mut self, options: Option<PropertyRowOptions>) -> Self {
        self.options = options;
        self
    }

    pub fn reset(mut self, reset: Option<PropertyRowReset>) -> Self {
        self.reset = reset;
        self
    }

    #[track_caller]
    pub fn into_element<H: UiHost>(
        self,
        cx: &mut ElementContext<'_, H>,
        row_cx: &PropertyGridRowCx,
        label: impl FnOnce(&mut ElementContext<'_, H>) -> AnyElement,
        value: impl FnOnce(&mut ElementContext<'_, H>) -> AnyElement,
        actions: impl FnOnce(&mut ElementContext<'_, H>) -> Option<AnyElement>,
    ) -> AnyElement {
        let options = self.options.unwrap_or_else(|| row_cx.row_options.clone());
        let row = PropertyRow::new().options(options).reset(self.reset);
        row.into_element(cx, label, value, actions)
    }
}
