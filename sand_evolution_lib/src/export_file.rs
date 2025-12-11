use std::error::Error;

#[cfg(target_arch = "wasm32")]
use wasm_bindgen::JsCast;
#[cfg(target_arch = "wasm32")]
use web_sys::{BlobPropertyBag, Url};

#[cfg(not(target_arch = "wasm32"))]
use std::fs;
#[cfg(not(target_arch = "wasm32"))]
pub fn code_to_file(data: &str) -> Result<(), Box<dyn Error>> {
    if let Some(path) = rfd::FileDialog::new()
        .set_file_name("exported.txt")
        .add_filter("Text", &["txt"])
        .add_filter("All", &["*"])
        .save_file()
    {
        fs::write(path, data)?;
    }
    Ok(())
}

#[cfg(not(target_arch = "wasm32"))]
pub fn scene_to_file(data: &str) -> Result<(), Box<dyn Error>> {
    if let Some(path) = rfd::FileDialog::new()
        .set_file_name("scene.toml")
        .add_filter("TOML", &["toml"])
        .add_filter("All", &["*"])
        .save_file()
    {
        fs::write(path, data)?;
    }
    Ok(())
}

#[cfg(target_arch = "wasm32")]
pub fn code_to_file(data: &str) -> Result<(), Box<dyn Error>> {
    use js_sys::Uint8Array;

    let buffer = Uint8Array::from(data.as_bytes());

    let window = web_sys::window().expect("window not found");
    let document = window.document().expect("document not found");
    let body = document.body().expect("body not found");

    // Create a Blob from the data
    let blob_parts = js_sys::Array::new();
    blob_parts.push(&buffer);
    let mut blob_options = BlobPropertyBag::new();
    blob_options.set_type("text/plain");
    let blob = web_sys::Blob::new_with_u8_array_sequence_and_options(
        &blob_parts,
        &blob_options,
    ).map_err(|e| format!("Failed to create blob: {:?}", e))?;

    // Create object URL from blob
    let url = Url::create_object_url_with_blob(&blob)
        .map_err(|e| format!("Failed to create object URL: {:?}", e))?;

    let link_element = document
        .create_element("a")
        .expect("link element creation failed");
    body.append_child(&link_element)
        .expect("link element appending failed");
    link_element
        .set_attribute("href", &url)
        .expect("failed to set an attribute");
    link_element
        .set_attribute("download", "exported.txt")
        .expect("failed to set an attribute");
    let html_link_element = link_element
        .dyn_into::<web_sys::HtmlElement>()
        .expect("html link element casting failed");
    html_link_element.click();

    // Clean up the object URL after a short delay
    let url_clone = url.clone();
    let closure = wasm_bindgen::closure::Closure::wrap(Box::new(move || {
        let _ = Url::revoke_object_url(&url_clone);
    }) as Box<dyn FnMut()>);
    window.set_timeout_with_callback_and_timeout_and_arguments_0(
        closure.as_ref().unchecked_ref(),
        100,
    ).map_err(|e| format!("Failed to set timeout: {:?}", e))?;
    closure.forget();

    Ok(())
}

#[cfg(target_arch = "wasm32")]
pub fn scene_to_file(data: &str) -> Result<(), Box<dyn Error>> {
    use js_sys::Uint8Array;

    let buffer = Uint8Array::from(data.as_bytes());

    let window = web_sys::window().expect("window not found");
    let document = window.document().expect("document not found");
    let body = document.body().expect("body not found");

    // Create a Blob from the data
    let blob_parts = js_sys::Array::new();
    blob_parts.push(&buffer);
    let mut blob_options = BlobPropertyBag::new();
    blob_options.set_type("text/plain");
    let blob = web_sys::Blob::new_with_u8_array_sequence_and_options(
        &blob_parts,
        &blob_options,
    )
    .map_err(|e| format!("Failed to create blob: {:?}", e))?;

    // Create object URL from blob
    let url = Url::create_object_url_with_blob(&blob)
        .map_err(|e| format!("Failed to create object URL: {:?}", e))?;

    let link_element = document
        .create_element("a")
        .expect("link element creation failed");
    body.append_child(&link_element)
        .expect("link element appending failed");
    link_element
        .set_attribute("href", &url)
        .expect("failed to set an attribute");
    link_element
        .set_attribute("download", "scene.toml")
        .expect("failed to set an attribute");
    let html_link_element = link_element
        .dyn_into::<web_sys::HtmlElement>()
        .expect("html link element casting failed");
    html_link_element.click();

    // Clean up the object URL after a short delay
    let url_clone = url.clone();
    let closure = wasm_bindgen::closure::Closure::wrap(Box::new(move || {
        let _ = Url::revoke_object_url(&url_clone);
    }) as Box<dyn FnMut()>);
    window
        .set_timeout_with_callback_and_timeout_and_arguments_0(
            closure.as_ref().unchecked_ref(),
            100,
        )
        .map_err(|e| format!("Failed to set timeout: {:?}", e))?;
    closure.forget();

    Ok(())
}

#[cfg(not(target_arch = "wasm32"))]
pub fn write_to_file(
    data: &image::ImageBuffer<image::Luma<u8>, Vec<u8>>,
) -> Result<(), Box<dyn Error>> {
    if let Some(path) = rfd::FileDialog::new()
        .set_file_name("exported.png")
        .add_filter("PNG Image", &["png"])
        .add_filter("All", &["*"])
        .save_file()
    {
        data.save(path)?;
    }
    Ok(())
}

#[cfg(target_arch = "wasm32")]
pub fn write_to_file(
    data: &image::ImageBuffer<image::Luma<u8>, Vec<u8>>,
) -> Result<(), Box<dyn Error>> {
    use std::io::Cursor;
    use js_sys::Uint8Array;

    let mut buffer = Vec::new();
    let mut cursor = Cursor::new(&mut buffer);
    
    // Convert ImageBuffer to DynamicImage and save as PNG
    let dynamic_image = image::DynamicImage::ImageLuma8(data.clone());
    dynamic_image.write_to(&mut cursor, image::ImageOutputFormat::Png)?;

    // Create a Uint8Array object from the binary buffer
    let uint8_array = Uint8Array::from(buffer.as_slice());

    let window = web_sys::window().expect("window not found");
    let document = window.document().expect("document not found");
    let body = document.body().expect("body not found");

    // Create a Blob from the PNG data
    let blob_parts = js_sys::Array::new();
    blob_parts.push(&uint8_array);
    let mut blob_options = BlobPropertyBag::new();
    blob_options.set_type("image/png");
    let blob = web_sys::Blob::new_with_u8_array_sequence_and_options(
        &blob_parts,
        &blob_options,
    ).map_err(|e| format!("Failed to create blob: {:?}", e))?;

    // Create object URL from blob
    let url = Url::create_object_url_with_blob(&blob)
        .map_err(|e| format!("Failed to create object URL: {:?}", e))?;

    let link_element = document
        .create_element("a")
        .expect("link element creation failed");
    body.append_child(&link_element)
        .expect("link element appending failed");
    link_element
        .set_attribute("href", &url)
        .expect("failed to set an attribute");
    link_element
        .set_attribute("download", "exported.png")
        .expect("failed to set an attribute");
    let html_link_element = link_element
        .dyn_into::<web_sys::HtmlElement>()
        .expect("html link element casting failed");
    html_link_element.click();

    // Clean up the object URL after a short delay
    let url_clone = url.clone();
    let closure = wasm_bindgen::closure::Closure::wrap(Box::new(move || {
        let _ = Url::revoke_object_url(&url_clone);
    }) as Box<dyn FnMut()>);
    window.set_timeout_with_callback_and_timeout_and_arguments_0(
        closure.as_ref().unchecked_ref(),
        100,
    ).map_err(|e| format!("Failed to set timeout: {:?}", e))?;
    closure.forget();

    Ok(())
}
