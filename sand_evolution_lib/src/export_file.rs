use egui::Ui;
use image::{DynamicImage, ImageEncoder, Luma};
use std::io::{Cursor, Write};
use std::{error::Error, fs::File};

#[cfg(target_arch = "wasm32")]
use js_sys::Uint8Array;
#[cfg(target_arch = "wasm32")]
use wasm_bindgen::JsCast;
#[cfg(target_arch = "wasm32")]
use web_sys::{BlobPropertyBag, Url};

use crate::cs;

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
    let _ = image::codecs::png::PngEncoder::new(&mut buffer).write_image(
        &data,
        cs::SECTOR_SIZE.x as u32,
        cs::SECTOR_SIZE.y as u32,
        image::ColorType::L8,
    )?;

    let js_array = Uint8Array::from(buffer.as_slice());

    let mut pr_bag = BlobPropertyBag::new();
    pr_bag.type_("image/png");
    let blob = web_sys::Blob::new_with_u8_array_sequence_and_options(&js_array, &pr_bag);

    // Create a URL from the Blob
    let url = web_sys::Url::create_object_url_with_blob(&blob.ok().unwrap());

    let window = web_sys::window().expect("window not found");
    let document = window.document().expect("document not found");
    let body = document.body().expect("body not found");
    let link = url.clone().unwrap();

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
