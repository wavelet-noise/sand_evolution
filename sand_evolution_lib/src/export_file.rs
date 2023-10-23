use std::error::Error;
use image::ImageEncoder;

#[cfg(target_arch = "wasm32")]
use wasm_bindgen::JsCast;
#[cfg(target_arch = "wasm32")]
use web_sys::{BlobPropertyBag, Url};
#[cfg(not(target_arch = "wasm32"))]
pub fn write_to_file(
    data: &image::ImageBuffer<image::Luma<u8>, Vec<u8>>,
) -> Result<(), Box<dyn Error>> {
    _ = data.save("exported.png");
    Ok(())
}

#[cfg(target_arch = "wasm32")]
pub fn write_to_file(
    data: &image::ImageBuffer<image::Luma<u8>, Vec<u8>>,
) -> Result<(), Box<dyn Error>> {
    let mut buffer = Vec::new();
    let result = image::codecs::png::PngEncoder::new(&mut buffer).write_image(
        &data,
        crate::cs::SECTOR_SIZE.x as u32,
        crate::cs::SECTOR_SIZE.y as u32,
        image::ColorType::L8,
    )?;

    // Create a Uint8Array object from the binary buffer
    let data_url = format!("data:image/png;base64,{}", base64::encode(&buffer));

    let window = web_sys::window().expect("window not found");
    let document = window.document().expect("document not found");
    let body = document.body().expect("body not found");
    let link = data_url.clone();

    let link_element = document
        .create_element("a")
        .expect("link element creation failed");
    body.append_child(&link_element)
        .expect("link element appending failed");
    link_element
        .set_attribute("href", &link)
        .expect("failed to set an attribute");
    link_element
        .set_attribute("download", "exported.png")
        .expect("failed to set an attribute");
    let html_link_element = link_element
        .dyn_into::<web_sys::HtmlElement>()
        .expect("html link element casting failed");
    html_link_element.click();


    Ok(())
}
