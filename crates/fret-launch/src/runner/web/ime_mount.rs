use web_sys::wasm_bindgen::JsCast as _;

pub(super) fn ensure_canvas_ime_mount(
    canvas: &web_sys::HtmlCanvasElement,
) -> Option<web_sys::HtmlElement> {
    let canvas_el: web_sys::HtmlElement = canvas.clone().unchecked_into();

    if let Some(parent) = canvas_el.parent_element() {
        // Preferred: a dedicated overlay layer inside the wrapper.
        if parent.get_attribute("data-fret-ime-wrapper").as_deref() == Some("1") {
            if let Ok(Some(overlay)) = parent.query_selector("[data-fret-ime-overlay='1']") {
                if let Ok(overlay) = overlay.dyn_into::<web_sys::HtmlElement>() {
                    return Some(overlay);
                }
            }

            let document = canvas_el.owner_document()?;
            let el = document.create_element("div").ok()?;
            let overlay: web_sys::HtmlElement = el.dyn_into().ok()?;
            let _ = overlay.set_attribute("data-fret-ime-overlay", "1");
            let _ = overlay.set_attribute("data-fret-ime-mount", "1");

            let style = overlay.style();
            let _ = style.set_property("position", "absolute");
            let _ = style.set_property("left", "0");
            let _ = style.set_property("top", "0");
            let _ = style.set_property("width", "100%");
            let _ = style.set_property("height", "100%");
            let _ = style.set_property("pointer-events", "none");
            let _ = style.set_property("overflow", "hidden");

            let _ = parent.append_child(&overlay);
            return Some(overlay);
        }

        // Back-compat: older mount strategy uses the direct parent as the mount.
        if parent.get_attribute("data-fret-ime-mount").as_deref() == Some("1") {
            if let Ok(parent) = parent.dyn_into::<web_sys::HtmlElement>() {
                return Some(parent);
            }
        }
    }

    let document = canvas_el.owner_document()?;
    let el = document.create_element("div").ok()?;
    let wrapper: web_sys::HtmlElement = el.dyn_into().ok()?;
    let _ = wrapper.set_attribute("data-fret-ime-wrapper", "1");

    let style = wrapper.style();
    let _ = style.set_property("position", "relative");
    let _ = style.set_property("margin", "0");
    let _ = style.set_property("padding", "0");
    let _ = style.set_property("border", "0");
    let _ = style.set_property("overflow", "hidden");
    let _ = style.set_property("display", "block");

    let parent = canvas_el.parent_node()?;
    let wrapper_node: web_sys::Node = wrapper.clone().unchecked_into();
    let canvas_node: web_sys::Node = canvas_el.clone().unchecked_into();

    let _ = parent.replace_child(&wrapper_node, &canvas_node);
    let _ = wrapper.append_child(&canvas_node);

    let el = document.create_element("div").ok()?;
    let overlay: web_sys::HtmlElement = el.dyn_into().ok()?;
    let _ = overlay.set_attribute("data-fret-ime-overlay", "1");
    let _ = overlay.set_attribute("data-fret-ime-mount", "1");
    let style = overlay.style();
    let _ = style.set_property("position", "absolute");
    let _ = style.set_property("left", "0");
    let _ = style.set_property("top", "0");
    let _ = style.set_property("width", "100%");
    let _ = style.set_property("height", "100%");
    let _ = style.set_property("pointer-events", "none");
    let _ = style.set_property("overflow", "hidden");
    let _ = wrapper.append_child(&overlay);

    Some(overlay)
}
