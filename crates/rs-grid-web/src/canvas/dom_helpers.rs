use wasm_bindgen::JsCast;
use web_sys::HtmlElement;

pub(super) fn document() -> web_sys::Document {
    web_sys::window()
        .expect("no window")
        .document()
        .expect("no document")
}

pub(super) fn make_el(doc: &web_sys::Document, tag: &str) -> HtmlElement {
    doc.create_element(tag)
        .expect("create element")
        .dyn_into::<HtmlElement>()
        .expect("element is HtmlElement")
}

pub(super) fn set_styles(el: &HtmlElement, styles: &[(&str, &str)]) {
    let s = el.style();
    for (prop, val) in styles {
        s.set_property(prop, val).expect("set style property");
    }
}
