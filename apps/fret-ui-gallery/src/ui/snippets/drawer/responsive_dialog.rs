pub const SOURCE: &str = include_str!("responsive_dialog.rs");

// region: example
use fret_ui_kit::ui;
use fret_ui_shadcn::{self as shadcn, prelude::*};

#[derive(Default, Clone)]
struct Models {
    desktop_open: Option<Model<bool>>,
    mobile_open: Option<Model<bool>>,
}

fn models<H: UiHost>(cx: &mut ElementContext<'_, H>) -> Models {
    cx.with_state(Models::default, |st| st.clone())
}

fn ensure_open<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    model: Option<Model<bool>>,
    set: impl FnOnce(&mut Models, Model<bool>),
) -> Model<bool> {
    match model {
        Some(model) => model,
        None => {
            let inserted = cx.app.models_mut().insert(false);
            cx.with_state(Models::default, |st| set(st, inserted.clone()));
            inserted
        }
    }
}

pub fn render<H: UiHost>(cx: &mut ElementContext<'_, H>) -> AnyElement {
    let st = models(cx);
    let desktop_open = ensure_open(cx, st.desktop_open, |st, model| {
        st.desktop_open = Some(model)
    });
    let mobile_open = ensure_open(cx, st.mobile_open, |st, model| st.mobile_open = Some(model));

    let desktop_open_trigger = desktop_open.clone();
    let mobile_open_trigger = mobile_open.clone();

    let desktop_dialog = shadcn::Dialog::new(desktop_open).into_element(
        cx,
        move |cx| {
            shadcn::Button::new("Desktop Dialog")
                .variant(shadcn::ButtonVariant::Outline)
                .toggle_model(desktop_open_trigger.clone())
                .test_id("ui-gallery-drawer-responsive-desktop-trigger")
                .into_element(cx)
        },
        move |cx| {
            shadcn::DialogContent::new([
                shadcn::DialogHeader::new([
                    shadcn::DialogTitle::new("Responsive Dialog").into_element(cx),
                    shadcn::DialogDescription::new(
                        "Desktop branch uses Dialog in the responsive pattern.",
                    )
                    .into_element(cx),
                ])
                .into_element(cx),
                shadcn::DialogFooter::new([shadcn::DialogClose::from_scope().build(
                    cx,
                    shadcn::Button::new("Close").variant(shadcn::ButtonVariant::Outline),
                )])
                .into_element(cx),
            ])
            .into_element(cx)
            .test_id("ui-gallery-drawer-responsive-desktop-content")
        },
    );

    let mobile_drawer = shadcn::Drawer::new(mobile_open).into_element(
        cx,
        move |cx| {
            shadcn::Button::new("Mobile Drawer")
                .variant(shadcn::ButtonVariant::Outline)
                .toggle_model(mobile_open_trigger.clone())
                .test_id("ui-gallery-drawer-responsive-mobile-trigger")
                .into_element(cx)
        },
        move |cx| {
            shadcn::DrawerContent::new([
                shadcn::DrawerHeader::new([
                    shadcn::DrawerTitle::new("Responsive Drawer").into_element(cx),
                    shadcn::DrawerDescription::new(
                        "Mobile branch uses Drawer in the responsive pattern.",
                    )
                    .into_element(cx),
                ])
                .into_element(cx),
                shadcn::DrawerFooter::new([shadcn::DrawerClose::from_scope().build(
                    cx,
                    shadcn::Button::new("Close").variant(shadcn::ButtonVariant::Outline),
                )])
                .into_element(cx),
            ])
            .into_element(cx)
            .test_id("ui-gallery-drawer-responsive-mobile-content")
        },
    );

    ui::h_flex(move |_cx| [desktop_dialog, mobile_drawer])
        .gap(Space::N2)
        .wrap()
        .w_full()
        .items_center()
        .into_element(cx)
}
// endregion: example
