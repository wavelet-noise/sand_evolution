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
struct ProjectList {
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
    use super::ProjectDescription;
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

    const GITHUB_CONTENT_URL: &str =
        "https://api.github.com/repos/wavelet-noise/sand_evolution_maps/contents";

    /// Fetch list of projects from GitHub repository.
    pub async fn fetch_github_projects() -> Result<Vec<ProjectDescription>, JsValue> {
        let window = web_sys::window().ok_or_else(|| JsValue::from_str("no window"))?;

        let resp_value = JsFuture::from(window.fetch_with_str(GITHUB_CONTENT_URL)).await?;
        let resp: web_sys::Response = resp_value.dyn_into()?;

        if !resp.ok() {
            let status = resp.status();
            let status_text = resp.status_text();
            return Err(JsValue::from_str(&format!(
                "GitHub request failed: {} {}",
                status, status_text
            )));
        }

        let json = JsFuture::from(resp.json()?).await?;

        let items: Vec<GithubContentItem> =
            serde_wasm_bindgen::from_value(json).map_err(|e| JsValue::from_str(&e.to_string()))?;

        let mut scripts: Vec<GithubContentItem> = Vec::new();
        let mut images: Vec<GithubContentItem> = Vec::new();

        for item in items.into_iter() {
            if item.item_type == "file" {
                if item.name.ends_with(".rhai") {
                    scripts.push(item);
                } else {
                    let lower = item.name.to_lowercase();
                    if lower.ends_with(".png") || lower.ends_with(".jpg") || lower.ends_with(".jpeg")
                    {
                        images.push(item);
                    }
                }
            }
        }

        use std::collections::HashMap;
        let mut image_by_base: HashMap<String, String> = HashMap::new();
        for img in images.into_iter() {
            let lower = img.name.to_lowercase();
            let base = lower
                .strip_suffix(".png")
                .or_else(|| lower.strip_suffix(".jpg"))
                .or_else(|| lower.strip_suffix(".jpeg"))
                .unwrap_or(&lower)
                .to_owned();
            image_by_base.entry(base).or_insert(img.download_url);
        }

        let mut result = Vec::new();
        for script in scripts.into_iter() {
            let base = script
                .name
                .strip_suffix(".rhai")
                .unwrap_or(script.name.as_str())
                .to_owned();
            let key = base.to_lowercase();
            let image_url = image_by_base.get(&key).cloned();
            result.push(ProjectDescription {
                id: script.name.clone(),
                display_name: base,
                script_url: script.download_url.clone(),
                image_url,
            });
        }

        Ok(result)
    }

    /// Fetch assets (optional background image + Rhai script text) for a specific project.
    pub async fn fetch_project_assets(
        project: &ProjectDescription,
    ) -> Result<(Option<Vec<u8>>, String), JsValue> {
        let window = web_sys::window().ok_or_else(|| JsValue::from_str("no window"))?;

        let mut image_bytes: Option<Vec<u8>> = None;

        if let Some(image_url) = &project.image_url {
            let resp_value = JsFuture::from(window.fetch_with_str(image_url)).await?;
            let resp: web_sys::Response = resp_value.dyn_into()?;
            if resp.ok() {
                let buffer = JsFuture::from(resp.array_buffer()?).await?;
                let buffer: js_sys::ArrayBuffer = buffer.dyn_into()?;
                let array = js_sys::Uint8Array::new(&buffer);
                let mut bytes = vec![0u8; array.length() as usize];
                array.copy_to(&mut bytes[..]);
                image_bytes = Some(bytes);
            }
            // Note: If image fetch fails (non-ok response), we silently continue
            // since background images are optional. The script will still load.
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

        Ok((image_bytes, script_text))
    }
}

#[cfg(target_arch = "wasm32")]
pub use github::{fetch_github_projects, fetch_project_assets};


