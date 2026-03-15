//! shadcn/ui `Form` facade (taxonomy + recipes).
//!
//! Upstream shadcn's `Form` is tightly integrated with `react-hook-form`. In Fret, we expose a
//! small, framework-agnostic surface focused on composition and consistent spacing.
//!
//! - `Form` maps to a vertical `FieldSet` container.
//! - `FormItem` maps to `Field` (label + control + description + message).
//! - `FormControl` approximates Radix `Slot.Root`: a single child passes through unchanged, while
//!   multi-child inputs keep a small compatibility fallback without `FieldContent`'s fill defaults.
//! - `FormMessage` maps to `FieldError` (destructive text).

use fret_ui::element::{AnyElement, ColumnProps, CrossAlign};
use fret_ui::{ElementContext, UiHost};

#[path = "form_field.rs"]
mod form_field;

pub use crate::field::Field as FormItem;
pub use crate::field::FieldDescription as FormDescription;
pub use crate::field::FieldError as FormMessage;
pub use crate::field::FieldLabel as FormLabel;
pub use crate::field::FieldSet as Form;
pub use crate::field::field_set as form;
pub use form_field::{FormErrorVisibility, FormField};

#[derive(Debug)]
pub struct FormControl {
    children: Vec<AnyElement>,
}

impl FormControl {
    pub fn new(children: impl IntoIterator<Item = AnyElement>) -> Self {
        Self {
            children: children.into_iter().collect(),
        }
    }

    #[track_caller]
    pub fn into_element<H: UiHost>(self, cx: &mut ElementContext<'_, H>) -> AnyElement {
        let mut children = self.children;
        if children.len() == 1 {
            return children.pop().expect("single-child form control");
        }

        cx.column(
            ColumnProps {
                align: CrossAlign::Start,
                ..Default::default()
            },
            move |_cx| children,
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use std::mem::discriminant;

    use fret_app::App;
    use fret_core::{AppWindowId, Point, Px, Rect, Size};
    use fret_runtime::Model;
    use fret_ui::element::{ElementKind, LayoutStyle, Length, SpacingLength};
    use fret_ui_kit::LayoutRefinement;

    use crate::shadcn_themes::{ShadcnBaseColor, ShadcnColorScheme, apply_shadcn_new_york};

    fn bounds() -> Rect {
        Rect::new(
            Point::new(Px(0.0), Px(0.0)),
            Size::new(Px(320.0), Px(180.0)),
        )
    }

    fn kind_layout(kind: &ElementKind) -> Option<&LayoutStyle> {
        match kind {
            ElementKind::Semantics(props) => Some(&props.layout),
            ElementKind::Container(props) => Some(&props.layout),
            ElementKind::Pressable(props) => Some(&props.layout),
            ElementKind::Stack(props) => Some(&props.layout),
            ElementKind::Column(props) => Some(&props.layout),
            ElementKind::Row(props) => Some(&props.layout),
            ElementKind::TextInput(props) => Some(&props.layout),
            ElementKind::TextArea(props) => Some(&props.layout),
            _ => None,
        }
    }

    #[test]
    fn form_control_is_slot_like_for_single_child() {
        let window = AppWindowId::default();
        let mut app = App::new();
        apply_shadcn_new_york(&mut app, ShadcnBaseColor::Neutral, ShadcnColorScheme::Light);

        let model: Model<String> = app.models_mut().insert(String::new());
        let control_only = fret_ui::elements::with_element_cx(
            &mut app,
            window,
            bounds(),
            "form-control-direct",
            |cx| {
                crate::input::Input::new(model.clone())
                    .refine_layout(LayoutRefinement::default().w_full().min_w_0())
                    .into_element(cx)
            },
        );

        let wrapped = fret_ui::elements::with_element_cx(
            &mut app,
            window,
            bounds(),
            "form-control-wrapped",
            |cx| {
                FormControl::new([crate::input::Input::new(model.clone())
                    .refine_layout(LayoutRefinement::default().w_full().min_w_0())
                    .into_element(cx)])
                .into_element(cx)
            },
        );

        assert_eq!(
            discriminant(&wrapped.kind),
            discriminant(&control_only.kind),
            "single-child FormControl should reuse the control root instead of introducing a layout wrapper",
        );
        assert_eq!(wrapped.children.len(), control_only.children.len());
        assert_eq!(
            kind_layout(&wrapped.kind)
                .expect("wrapped control layout")
                .size
                .width,
            kind_layout(&control_only.kind)
                .expect("direct control layout")
                .size
                .width,
        );
    }

    #[test]
    fn form_control_multi_child_fallback_drops_field_content_fill_defaults() {
        let window = AppWindowId::default();
        let mut app = App::new();

        let wrapped = fret_ui::elements::with_element_cx(
            &mut app,
            window,
            bounds(),
            "form-control-multi",
            |cx| FormControl::new([cx.text("alpha"), cx.text("beta")]).into_element(cx),
        );

        let ElementKind::Column(props) = &wrapped.kind else {
            panic!("expected multi-child FormControl fallback to use a zero-gap column");
        };

        assert_eq!(props.gap, SpacingLength::Px(Px(0.0)));
        assert_eq!(props.layout.size.width, Length::Auto);
        assert_eq!(props.layout.flex.grow, 0.0);
        assert_eq!(props.align, CrossAlign::Start);
    }
}
