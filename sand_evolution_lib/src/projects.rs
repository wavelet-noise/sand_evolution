use serde::{Deserialize, Serialize};

/// Description of a sand evolution project: background image + Rhai script.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ProjectDescription {
    /// Identifier, usually a file name (e.g. `galaxy.rhai`).
    pub id: String,
    /// Human-friendly name to show in UI (e.g. `galaxy`).
    pub display_name: String,
    /// URL to the Rhai script.
    pub script_url: String,
    /// Optional URL to the background image (PNG/JPEG).
    pub image_url: Option<String>,
}

impl ProjectDescription {
    /// Serialize a single project description to a pretty TOML string.
    pub fn to_toml_pretty(&self) -> String {
        toml::to_string_pretty(self).unwrap_or_else(|_| String::new())
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub(crate) struct ProjectList {
    project: Vec<ProjectDescription>,
}

/// Built-in demo projects used as a fallback when GitHub is unavailable or
/// when running outside the browser.
pub fn demo_projects() -> Vec<ProjectDescription> {
    vec![
        ProjectDescription {
            id: "zeus2.rhai".to_owned(),
            display_name: "zeus2 (demo)".to_owned(),
            script_url:
                "https://raw.githubusercontent.com/wavelet-noise/sand_evolution_maps/refs/heads/main/zeus2.rhai"
                    .to_owned(),
            image_url: Some(
                "https://raw.githubusercontent.com/wavelet-noise/sand_evolution_maps/main/empty_box.png"
                    .to_owned(),
            ),
        },
        ProjectDescription {
            id: "zeus2_script_only.rhai".to_owned(),
            display_name: "zeus2 (script only demo)".to_owned(),
            script_url:
                "https://raw.githubusercontent.com/wavelet-noise/sand_evolution_maps/refs/heads/main/zeus2.rhai"
                    .to_owned(),
            image_url: None,
        },
    ]
}

/// Serialize a list of project descriptions into a TOML string.
pub fn serialize_projects(projects: &[ProjectDescription]) -> String {
    let list = ProjectList {
        project: projects.to_vec(),
    };
    toml::to_string_pretty(&list).unwrap_or_else(|_| String::new())
}

#[cfg(not(target_arch = "wasm32"))]
pub fn save_projects_to_file(
    path: &str,
    projects: &[ProjectDescription],
) -> Result<(), Box<dyn std::error::Error>> {
    let toml_text = serialize_projects(projects);
    std::fs::write(path, toml_text)?;
    Ok(())
}

#[cfg(not(target_arch = "wasm32"))]
pub fn load_projects_from_file(
    path: &str,
) -> Result<Vec<ProjectDescription>, Box<dyn std::error::Error>> {
    let data = std::fs::read_to_string(path)?;
    let parsed: ProjectList = toml::from_str(&data)?;
    Ok(parsed.project)
}

// GitHub integration is available only on wasm builds.
#[cfg(target_arch = "wasm32")]
mod github {
    use super::{ProjectDescription, ProjectList};
    use serde::Deserialize;
    use wasm_bindgen::prelude::*;
    use wasm_bindgen_futures::JsFuture;

    #[derive(Debug, Deserialize)]
    struct GithubContentItem {
        pub name: String,
        #[serde(rename = "download_url")]
        pub download_url: String,
        #[serde(rename = "type")]
        pub item_type: String,
    }

    const PROJECTS_TOML_URL: &str =
        "https://raw.githubusercontent.com/wavelet-noise/sand_evolution_maps/main/projects.toml";

    /// Fetch list of projects from GitHub repository.
    pub async fn fetch_github_projects() -> Result<Vec<ProjectDescription>, JsValue> {
        let window = web_sys::window().ok_or_else(|| JsValue::from_str("no window"))?;

        let resp_value = JsFuture::from(window.fetch_with_str(PROJECTS_TOML_URL)).await?;
        let toml_resp: web_sys::Response = resp_value.dyn_into()?;

        if !toml_resp.ok() {
            return Err(JsValue::from_str("Failed to load projects.toml"));
        }

        let text_js = JsFuture::from(toml_resp.text()?).await?;
        let toml_text = text_js.as_string()
            .ok_or_else(|| JsValue::from_str("projects.toml is not a valid string"))?;

        let project_list: ProjectList = toml::from_str(&toml_text)
            .map_err(|e| JsValue::from_str(&format!("Failed to parse projects.toml: {}", e)))?;

        Ok(project_list.project)
    }

    /// Fetch assets (optional background image + Rhai script text) for a specific project.
    /// Returns (image_bytes, script_text, image_error_message)
    pub async fn fetch_project_assets(
        project: &ProjectDescription,
    ) -> Result<(Option<Vec<u8>>, String, Option<String>), JsValue> {
        let window = web_sys::window().ok_or_else(|| JsValue::from_str("no window"))?;

        let mut image_bytes: Option<Vec<u8>> = None;
        let mut image_error: Option<String> = None;

        if let Some(image_url) = &project.image_url {
            // Image loading is optional - if it fails, we continue without it but report the error
            match JsFuture::from(window.fetch_with_str(image_url)).await {
                Ok(resp_value) => {
                    match resp_value.dyn_into::<web_sys::Response>() {
                        Ok(resp) => {
                            if resp.ok() {
                                match resp.array_buffer() {
                                    Ok(buffer_future) => {
                                        match JsFuture::from(buffer_future).await {
                                            Ok(buffer) => {
                                                match buffer.dyn_into::<js_sys::ArrayBuffer>() {
                                                    Ok(array_buffer) => {
                                                        let array = js_sys::Uint8Array::new(&array_buffer);
                                                        let mut bytes = vec![0u8; array.length() as usize];
                                                        array.copy_to(&mut bytes[..]);
                                                        image_bytes = Some(bytes);
                                                    }
                                                    Err(e) => {
                                                        image_error = Some(format!(
                                                            "Failed to convert image buffer: {:?}",
                                                            e
                                                        ));
                                                    }
                                                }
                                            }
                                            Err(e) => {
                                                image_error = Some(format!(
                                                    "Failed to read image buffer: {:?}",
                                                    e
                                                ));
                                            }
                                        }
                                    }
                                    Err(e) => {
                                        image_error = Some(format!(
                                            "Failed to get image array buffer: {:?}",
                                            e
                                        ));
                                    }
                                }
                            } else {
                                let status = resp.status();
                                let status_text = resp.status_text();
                                image_error = Some(format!(
                                    "Image request failed: {} {} for URL: {}",
                                    status, status_text, image_url
                                ));
                            }
                        }
                        Err(e) => {
                            image_error = Some(format!(
                                "Failed to convert image response: {:?}",
                                e
                            ));
                        }
                    }
                }
                Err(e) => {
                    image_error = Some(format!(
                        "Failed to fetch image from {}: {:?}",
                        image_url, e
                    ));
                }
            }
        }

        let resp_value = JsFuture::from(window.fetch_with_str(&project.script_url)).await?;
        let resp: web_sys::Response = resp_value.dyn_into()?;
        if !resp.ok() {
            let status = resp.status();
            let status_text = resp.status_text();
            return Err(JsValue::from_str(&format!(
                "Script request failed: {} {}",
                status, status_text
            )));
        }
        let text_js = JsFuture::from(resp.text()?).await?;
        let script_text = text_js
            .as_string()
            .unwrap_or_else(|| "// failed to load script".to_owned());

        Ok((image_bytes, script_text, image_error))
    }
}

#[cfg(target_arch = "wasm32")]
pub use github::{fetch_github_projects, fetch_project_assets};


