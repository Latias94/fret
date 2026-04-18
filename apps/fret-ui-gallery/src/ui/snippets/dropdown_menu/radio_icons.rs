pub const SOURCE: &str = include_str!("radio_icons.rs");

// region: example
use fret::{AppComponentCx, UiChild};
use fret_core::Px;
use fret_ui_kit::IntoUiElement;
use fret_ui_kit::declarative::ModelWatchExt as _;
use fret_ui_shadcn::{facade as shadcn, prelude::*};
use std::sync::Arc;

#[derive(Default, Clone)]
struct CheckoutSettings {
    payment_method: Option<Arc<str>>,
}

pub fn render(cx: &mut AppComponentCx<'_>) -> impl UiChild + use<> {
    let checkout = cx.local_model(|| CheckoutSettings {
        payment_method: Some(Arc::<str>::from("card")),
    });
    let checkout_now = cx
        .watch_model(&checkout)
        .layout()
        .cloned()
        .unwrap_or_default();

    super::preview_frame_with(cx, |cx| {
        shadcn::DropdownMenu::uncontrolled(cx)
            .compose()
            .trigger(
                shadcn::Button::new("Payment Method")
                    .variant(shadcn::ButtonVariant::Outline)
                    .test_id("ui-gallery-dropdown-menu-radio-icons-trigger"),
            )
            .content(
                shadcn::DropdownMenuContent::new()
                    .align(shadcn::DropdownMenuAlign::Start)
                    .side_offset(Px(4.0))
                    // shadcn/ui docs: `DropdownMenuContent className="min-w-56"`.
                    .min_width(Px(224.0)),
            )
            .entries([shadcn::DropdownMenuGroup::new([
                shadcn::DropdownMenuLabel::new("Select Payment Method").into(),
                shadcn::DropdownMenuRadioGroup::from_value(checkout_now.payment_method.clone())
                    .on_value_change({
                        let checkout = checkout.clone();
                        move |host, _action_cx, value| {
                            let _ = host
                                .models_mut()
                                .update(&checkout, |state| state.payment_method = Some(value));
                        }
                    })
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
            .into()])
    })
    .into_element(cx)
}
// endregion: example
