use wasm_bindgen::prelude::*;

#[wasm_bindgen]
pub fn recognize_page_gray_u8(gray: &[u8], width: u32, height: u32) -> Result<String, JsValue> {
    static INITIALIZED: std::sync::OnceLock<()> = std::sync::OnceLock::new();
    INITIALIZED.get_or_init(|| {
        crate::infer::backend::set_backend(Box::new(crate::infer::web_backend::WebBackend))
            .expect("browser backend must initialize before inference");
    });
    let options = crate::PageOptions {
        trace: true,
        ..crate::PageOptions::default()
    };
    crate::recognize_page_gray_u8(gray, width as usize, height as usize, &options)
        .and_then(|page| Ok(serde_json::to_string(&page.trace)?))
        .map_err(|error| JsValue::from_str(&format!("{error:#}")))
}
