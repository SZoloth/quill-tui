//! Browser-based file I/O using Web APIs

use wasm_bindgen::prelude::*;
use web_sys::{Blob, HtmlAnchorElement, Url};

use quill_core::Document;

/// Download JSON as a file
pub fn download_json(filename: &str, json: &str) -> Result<(), JsValue> {
    let window = web_sys::window().ok_or("No window")?;
    let document = window.document().ok_or("No document")?;

    // Create a blob from the JSON content
    let blob_parts = js_sys::Array::new();
    blob_parts.push(&JsValue::from_str(json));

    let blob_options = web_sys::BlobPropertyBag::new();
    blob_options.set_type("application/json");

    let blob = Blob::new_with_str_sequence_and_options(&blob_parts, &blob_options)?;

    // Create an object URL for the blob
    let url = Url::create_object_url_with_blob(&blob)?;

    // Create a temporary anchor element and trigger download
    let anchor: HtmlAnchorElement = document
        .create_element("a")?
        .dyn_into()?;

    anchor.set_href(&url);
    anchor.set_download(filename);
    anchor.click();

    // Clean up the object URL
    Url::revoke_object_url(&url)?;

    Ok(())
}

/// Save document to localStorage
pub fn save_to_storage(key: &str, doc: &Document) -> Result<(), JsValue> {
    let window = web_sys::window().ok_or("No window")?;
    let storage = window.local_storage()?.ok_or("No localStorage")?;

    let json = quill_core::to_json(doc)
        .map_err(|e| JsValue::from_str(&e.to_string()))?;

    storage.set_item(key, &json)?;

    Ok(())
}

/// Load document from localStorage
pub fn load_from_storage(key: &str) -> Result<Option<String>, JsValue> {
    let window = web_sys::window().ok_or("No window")?;
    let storage = window.local_storage()?.ok_or("No localStorage")?;

    Ok(storage.get_item(key)?)
}
