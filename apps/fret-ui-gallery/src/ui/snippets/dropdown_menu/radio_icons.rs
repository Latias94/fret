pub const SOURCE: &str = include_str!("radio_icons.rs");

// region: example
use fret_core::Px;
use fret_ui_shadcn::{self as shadcn, prelude::*};
use std::sync::Arc;

#[derive(Default, Clone)]
struct Models {
    payment_method: Option<Model<Option<Arc<str>>>>,
}

pub fn render<H: UiHost>(cx: &mut ElementContext<'_, H>) -> AnyElement {
    let state = cx.with_state(Models::default, |st| st.clone());

    let payment_method = match state.payment_method {
        Some(model) => model,
        None => {
            let model = cx.app.models_mut().insert(Some(Arc::<str>::from("card")));
            cx.with_state(Models::default, |st| {
                st.payment_method = Some(model.clone())
            });
            model
        }
    };

    shadcn::DropdownMenu::new_controllable(cx, None, false).build_parts(
        cx,
        shadcn::DropdownMenuTrigger::build(
            shadcn::Button::new("Payment Method")
                .variant(shadcn::ButtonVariant::Outline)
                .test_id("ui-gallery-dropdown-menu-radio-icons-trigger"),
        ),
        shadcn::DropdownMenuContent::new()
            .align(shadcn::DropdownMenuAlign::Start)
            .side_offset(Px(4.0))
            // shadcn/ui docs: `DropdownMenuContent className="min-w-56"`.
            .min_width(Px(224.0)),
        |_cx| {
            [shadcn::DropdownMenuGroup::new([
                shadcn::DropdownMenuLabel::new("Select Payment Method").into(),
                shadcn::DropdownMenuRadioGroup::new(payment_method.clone())
                    .item(
                        shadcn::DropdownMenuRadioItemSpec::new("card", "Credit Card")
                            .leading_icon(IconId::new_static("lucide.credit-card"))
                            .test_id("ui-gallery-dropdown-menu-radio-icons-card"),
                    )
                    .item(
                        shadcn::DropdownMenuRadioItemSpec::new("paypal", "PayPal")
                            .leading_icon(IconId::new_static("lucide.wallet"))
                            .test_id("ui-gallery-dropdown-menu-radio-icons-paypal"),
                    )
                    .item(
                        shadcn::DropdownMenuRadioItemSpec::new("bank", "Bank Transfer")
                            .leading_icon(IconId::new_static("lucide.building-2"))
                            .test_id("ui-gallery-dropdown-menu-radio-icons-bank"),
                    )
                    .into(),
            ])
            .into()]
        },
    )
}
// endregion: example
