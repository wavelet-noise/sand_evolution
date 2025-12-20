use cgmath::num_traits::clamp;
use egui::{Color32, ComboBox, Context};
use std::cell::RefCell;
use std::collections::VecDeque;
use std::rc::Rc;
use winit::{dpi::PhysicalPosition, event_loop::EventLoopProxy};

use crate::export_file::{code_to_file, scene_to_file};
use crate::projects::ProjectDescription;
use crate::resources::rhai_resource::RhaiResourceStorage;
use crate::{
    cells::{stone::Stone, void::Void, wood::Wood},
    copy_text_to_clipboard, cs,
    editor::{EditorHierarchy, EditorInspector, EditorState, UndoRedo},
    export_file::write_to_file,
    fps_meter::FpsMeter,
    state::{State, UpdateResult},
};
use specs::WorldExt;

/// Simple manager for window styling (background color, text color, etc.).
/// For now all app windows share the same style.
pub struct WindowStyleManager {
    pub window_background: Color32,
    pub text_color: Color32,
}

impl WindowStyleManager {
    /// Apply common frame (background) to a window builder.
    pub fn apply<'a>(&self, window: egui::Window<'a>) -> egui::Window<'a> {
        window.frame(
            egui::Frame::window(&egui::Style::default()).fill(self.window_background),
        )
    }

    /// Apply common text color to a window's UI.
    pub fn apply_to_ui(&self, ui: &mut egui::Ui) {
        ui.visuals_mut().override_text_color = Some(self.text_color);
    }
}

impl Default for WindowStyleManager {
    fn default() -> Self {
        Self {
            // Slightly translucent dark background.
            window_background: Color32::from_rgba_unmultiplied(20, 10, 10, 180),
            // Brighter, warm-ish text.
            text_color: Color32::from_rgb(240, 230, 230),
        }
    }
}

struct Executor {
    #[cfg(not(target_arch = "wasm32"))]
    pool: futures::executor::ThreadPool,
}

impl Executor {
    fn new() -> Self {
        Self {
            #[cfg(not(target_arch = "wasm32"))]
            pool: futures::executor::ThreadPool::new().unwrap(),
        }
    }

    #[cfg(not(target_arch = "wasm32"))]
    fn execute<F: futures::Future<Output = ()> + Send + 'static>(&self, f: F) {
        self.pool.spawn_ok(f);
    }

    #[cfg(target_arch = "wasm32")]
    fn execute<F: futures::Future<Output = ()> + 'static>(&self, f: F) {
        wasm_bindgen_futures::spawn_local(f);
    }
}

pub struct EvolutionApp {
    pub number_of_cells_to_add: i32,
    pub number_of_structures_to_add: i32,
    pub simulation_steps_per_second: i32,
    /// Whether automatic simulation stepping is paused (manual step buttons still work).
    pub simulation_paused: bool,
    /// Extra simulation steps to run once (used by "step" buttons in the UI).
    pub pending_simulation_steps: i32,
    pub selected_option: String,
    pub options: Vec<String>,
    pub cursor_position: Option<PhysicalPosition<f64>>,
    pub pressed: bool,
    pub hovered: bool,
    /// Info about the currently hovered simulation cell (if any).
    pub hover_info: Option<HoverInfo>,
    script: String, // For backward compatibility - stores the script of the selected object
    pub selected_object_name: String, // Name of the selected object for editing
    last_loaded_object: String, // Last loaded object (for tracking changes)
    script_modified: bool, // Script modification flag
    pub need_to_recompile: bool,
    pub script_error: String,
    executor: Executor,

    // Windows
    pub win_files: bool,
    pub win_script_editor: bool,
    pub win_simulation: bool,
    pub win_graphics: bool,
    pub win_templates: bool, // templates / projects window
    pub win_palette: bool,   // palette window
    pub win_hover: bool,     // hover info window

    // GitHub project support
    pub projects: Vec<ProjectDescription>,
    pub selected_project: Option<usize>,
    pub project_loading: bool,
    pub project_error: String,
    pub projects_fetched: bool, // Track if we've attempted to fetch from GitHub

    // Last generated share URL for templates
    pub last_load_url: String,

    // Editor state
    pub editor_state: EditorState,
    pub undo_redo: UndoRedo,

    // Script log storage - circular buffer with a limit of 30 entries
    pub script_log: Rc<RefCell<VecDeque<String>>>,
    pub show_log_window: bool,

    // Display mode: Normal or Temperature map
    pub display_mode: DisplayMode,
    /// Which temperature field is visualized (ambient / per‑cell / combined).
    pub heat_vis_mode: HeatVisMode,
    /// Number of per‑cell temperature diffusion iterations per simulation tick.
    /// Позволяет усиливать «проводимость» без изменения самой схемы.
    pub cell_diffusion_iterations: i32,
    /// Centralized window style (background color, etc.).
    pub window_style: WindowStyleManager,
}

#[derive(Debug, Clone, Copy)]
pub struct HoverInfo {
    pub x: cs::PointType,
    pub y: cs::PointType,
    pub cell_id: u8,
    /// Температура самой клетки (локальная, 1:1 с сеткой клеток).
    pub cell_temperature: f32,
    /// Амбиентная (сглаженная, низкочастотная) температура в окрестности курсора.
    pub ambient_temperature: f32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DisplayMode {
    Normal,
    Temperature,
    Both,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HeatVisMode {
    /// Show only the low‑resolution ambient temperature field.
    Ambient,
    /// Show only the per‑cell temperature field (1:1 with cells).
    Cells,
    /// Show a combined view (current behaviour, max(ambient, cell)).
    Combined,
}

pub fn compact_number_string(n: f32) -> String {
    let abs = cgmath::num_traits::abs(n);

    if abs < 999.0 {
        return format!("{}", abs);
    }

    if abs < 999999.0 {
        return format!("{:.2}k", abs / 1000.0);
    }

    if abs < 999999999.0 {
        return format!("{:.2}M", abs / 1000000.0);
    }

    if abs < 999999999999.0 {
        return format!("{:.2}G", abs / 1000000000.0);
    }

    format!("{:.2}T", abs / 1000000000000.0)
}

pub enum UserEventInfo {
    ImageImport(Vec<u8>),
    TextImport(Vec<u8>),
    SceneImport(Vec<u8>),
    /// GitHub templates currently don't ship an entity list / scene.
    /// When applying such a template, we reset the ECS world entities to the hardcoded defaults.
    ResetWorldEntitiesToHardcoded,
    ProjectsLoaded(Vec<ProjectDescription>),
    ProjectLoadError(String),
}

impl EvolutionApp {
    pub fn get_script(&mut self) -> &str {
        self.script.as_str()
    }

    pub fn set_script(&mut self, value: &str) -> bool {
        self.script = value.to_owned();
        self.script_modified = true; // Imported script is considered modified
        self.need_to_recompile = true;
        true
    }

    /// Get object script by name from world
    pub fn get_object_script(&self, world: &specs::World, object_name: &str) -> Option<String> {
        use crate::ecs::components::{Name, Script};
        use specs::Join;

        let names = world.read_storage::<Name>();
        let scripts = world.read_storage::<Script>();
        let entities = world.entities();

        for (entity, name_comp) in (&entities, &names).join() {
            if name_comp.name == object_name {
                if let Some(script) = scripts.get(entity) {
                    return Some(script.script.clone());
                }
            }
        }
        None
    }

    /// Set object script by name in world
    pub fn set_object_script(&mut self, world: &mut specs::World, object_name: &str, script: &str) {
        use crate::ecs::components::{Name, Script};
        use specs::Join;

        let names = world.read_storage::<Name>();
        let mut scripts = world.write_storage::<Script>();
        let entities = world.entities();

        for (entity, name_comp) in (&entities, &names).join() {
            if name_comp.name == object_name {
                if let Some(script_comp) = scripts.get_mut(entity) {
                    script_comp.script = script.to_owned();
                    script_comp.raw = true;
                    // If this is a one-shot script, allow it to run again after updating the code.
                    script_comp.has_run = false;
                    self.need_to_recompile = true;
                }
            }
        }
    }

    #[cfg(target_arch = "wasm32")]
    fn start_fetch_github_projects(&mut self, event_loop_proxy: &EventLoopProxy<UserEventInfo>) {
        if self.project_loading {
            return;
        }
        self.project_loading = true;
        self.project_error.clear();

        let proxy = event_loop_proxy.clone();
        self.executor.execute(async move {
            match crate::projects::fetch_github_projects().await {
                Ok(projects) => {
                    let _ = proxy.send_event(UserEventInfo::ProjectsLoaded(projects));
                }
                Err(e) => {
                    let _ = proxy.send_event(UserEventInfo::ProjectLoadError(format!(
                        "GitHub projects error: {:?}",
                        e
                    )));
                }
            }
        });
    }

    #[cfg(not(target_arch = "wasm32"))]
    fn start_fetch_github_projects(&mut self, _event_loop_proxy: &EventLoopProxy<UserEventInfo>) {
        self.project_error =
            "Loading projects from GitHub is only supported in the Web build.".to_owned();
    }

    #[cfg(target_arch = "wasm32")]
    fn start_load_project_from_github(
        &mut self,
        index: usize,
        event_loop_proxy: &EventLoopProxy<UserEventInfo>,
    ) {
        if index >= self.projects.len() {
            return;
        }
        self.project_loading = true;
        self.project_error.clear();

        let proj = self.projects[index].clone();
        let proxy = event_loop_proxy.clone();
        self.executor.execute(async move {
            match crate::projects::fetch_project_assets(&proj).await {
                Ok((maybe_image, script_text, image_error)) => {
                    // This project load path does not include a scene/entity list.
                    // Reset entities to the current hardcoded defaults before applying assets.
                    let _ = proxy.send_event(UserEventInfo::ResetWorldEntitiesToHardcoded);
                    if let Some(img) = maybe_image {
                        let _ = proxy.send_event(UserEventInfo::ImageImport(img));
                    }
                    let _ = proxy.send_event(UserEventInfo::TextImport(script_text.into_bytes()));
                    // Report image loading errors if any (non-critical, script loaded successfully)
                    if let Some(err_msg) = image_error {
                        let _ = proxy.send_event(UserEventInfo::ProjectLoadError(format!(
                            "⚠️ Background image failed to load (script loaded successfully): {}",
                            err_msg
                        )));
                    }
                }
                Err(e) => {
                    let _ = proxy.send_event(UserEventInfo::ProjectLoadError(format!(
                        "Project load error: {:?}",
                        e
                    )));
                }
            }
        });
    }

    #[cfg(not(target_arch = "wasm32"))]
    fn start_load_project_from_github(
        &mut self,
        _index: usize,
        _event_loop_proxy: &EventLoopProxy<UserEventInfo>,
    ) {
        self.project_error =
            "Loading project from GitHub is only supported in the Web build.".to_owned();
    }

    pub(crate) fn ui(
        &mut self,
        context: &Context,
        state: &mut State,
        fps_meter: &mut FpsMeter,
        upd_result: &UpdateResult,
        event_loop_proxy: &EventLoopProxy<UserEventInfo>,
        any_win_hovered: &mut bool,
        world: &mut specs::World,
    ) {
        // Update editor toasts
        self.editor_state
            .update_toasts(upd_result.update_time as f32 / 1000.0);

        // Handle keyboard shortcuts for editor
        // TODO: Fix input handling for egui 0.19 - temporarily disabled
        // Keyboard shortcuts will be handled through other means

        let mut win_files: bool = self.win_files;
        let mut win_script_editor: bool = self.win_script_editor;
        let mut win_simulation: bool = self.win_simulation;
        let mut win_templates: bool = self.win_templates;
        let mut win_palette: bool = self.win_palette;
        let mut win_hover: bool = self.win_hover;

        // Calculate panel layout only once at startup
        // Using typical screen size (1920x1080) for initial layout
        if self.editor_state.hierarchy_pos.is_none() {
            // Typical viewport dimensions (will be adjusted if needed)
            let viewport_width = 1920.0;
            let viewport_height = 1080.0;

            // Calculate panel dimensions based on viewport
            let panel_width = 350.0; // Moderate width, comfortable to use
            let panel_height = (viewport_height - 30.0) / 2.0; // Half height minus spacing
            let margin = 10.0;

            // Calculate positions
            let right_x = viewport_width - panel_width - margin;
            let hierarchy_y = margin;
            let inspector_y = hierarchy_y + panel_height + margin;

            // Store in state
            self.editor_state.hierarchy_pos = Some((right_x, hierarchy_y));
            self.editor_state.hierarchy_size = Some((panel_width, panel_height));
            self.editor_state.inspector_pos = Some((right_x, inspector_y));
            self.editor_state.inspector_size = Some((panel_width, panel_height));
        }

        // Editor Hierarchy - right column, top half
        let (hierarchy_x, hierarchy_y) = self.editor_state.hierarchy_pos.unwrap_or((1560.0, 10.0));
        let (hierarchy_w, hierarchy_h) = self.editor_state.hierarchy_size.unwrap_or((350.0, 530.0));
        self.window_style
            .apply(egui::Window::new("Hierarchy"))
            .default_pos(egui::pos2(hierarchy_x, hierarchy_y))
            .default_size(egui::vec2(hierarchy_w, hierarchy_h))
            .show(context, |ui| {
                self.window_style.apply_to_ui(ui);
                EditorHierarchy::ui(ui, &mut self.editor_state, world);
            });

        // Editor Inspector - right column, bottom half
        let (inspector_x, inspector_y) = self.editor_state.inspector_pos.unwrap_or((1560.0, 550.0));
        let (inspector_w, inspector_h) = self.editor_state.inspector_size.unwrap_or((350.0, 530.0));
        self.window_style
            .apply(egui::Window::new("Inspector"))
            .default_pos(egui::pos2(inspector_x, inspector_y))
            .default_size(egui::vec2(inspector_w, inspector_h))
            .show(context, |ui| {
                self.window_style.apply_to_ui(ui);
                EditorInspector::ui(ui, &mut self.editor_state, world);
            });

        // Handle request to open scripts window for a specific object
        if let Some(object_name) = self.editor_state.open_scripts_for_object.take() {
            self.selected_object_name = object_name;
            win_script_editor = true;
        }

        // Show toasts
        self.show_toasts(context);

        self.window_style
            .apply(egui::Window::new("🪟 Windows"))
            .default_pos(egui::pos2(10.0, 10.0))
            .resizable(false)
            .show(context, |ui| {
                self.window_style.apply_to_ui(ui);
                let toggle_btn = |ui: &mut egui::Ui, value: &mut bool, label: &str| {
                    if ui.add(egui::SelectableLabel::new(*value, label)).clicked() {
                        *value = !*value;
                    }
                };

                ui.heading("Windows");
                ui.add_space(4.0);

                toggle_btn(ui, &mut win_files, "📁 Files");
                toggle_btn(ui, &mut win_script_editor, "📝 Script Editor");
                toggle_btn(ui, &mut self.show_log_window, "📜 Script Log");
                toggle_btn(ui, &mut win_simulation, "⏱ Simulation");
                toggle_btn(ui, &mut self.win_graphics, "🎛 Graphics");
                toggle_btn(ui, &mut win_templates, "🧩 Templates");
                toggle_btn(ui, &mut win_palette, "🎨 Palette");
                toggle_btn(ui, &mut win_hover, "🔎 Hover");

                ui.separator();

                ui.heading("Display");
                ui.add_space(4.0);
                ui.horizontal_wrapped(|ui| {
                    if ui
                        .add(egui::SelectableLabel::new(
                            self.display_mode == DisplayMode::Normal,
                            "🖼 Normal",
                        ))
                        .clicked()
                    {
                        self.display_mode = DisplayMode::Normal;
                    }

                    if ui
                        .add(egui::SelectableLabel::new(
                            self.display_mode == DisplayMode::Temperature,
                            "🌡 Temperature (°)",
                        ))
                        .clicked()
                    {
                        self.display_mode = DisplayMode::Temperature;
                    }

                    if ui
                        .add(egui::SelectableLabel::new(
                            self.display_mode == DisplayMode::Both,
                            "🧪 Both",
                        ))
                        .clicked()
                    {
                        self.display_mode = DisplayMode::Both;
                    }
                });
                ui.add_space(6.0);
                ui.label("Temperature source");
                ComboBox::from_id_source("heat_source_mode")
                    .width(180.0)
                    .selected_text(match self.heat_vis_mode {
                        HeatVisMode::Ambient => "Ambient",
                        HeatVisMode::Cells => "Cells",
                        HeatVisMode::Combined => "Ambient + Cells",
                    })
                    .show_ui(ui, |ui| {
                        ui.selectable_value(&mut self.heat_vis_mode, HeatVisMode::Ambient, "Ambient");
                        ui.selectable_value(&mut self.heat_vis_mode, HeatVisMode::Cells, "Cells");
                        ui.selectable_value(
                            &mut self.heat_vis_mode,
                            HeatVisMode::Combined,
                            "Ambient + Cells",
                        );
                    });
            });

        self.window_style
            .apply(egui::Window::new("🔎 Hover"))
            .open(&mut win_hover)
            .default_pos(egui::pos2(340.0, 440.0))
            .resizable(false)
            .show(context, |ui| {
                self.window_style.apply_to_ui(ui);
                ui.heading("Hovered cell");
                ui.add_space(4.0);

                if let Some(info) = self.hover_info {
                    let cell_name = state
                        .pal_container
                        .pal
                        .get(info.cell_id as usize)
                        .map(|c| c.name())
                        .unwrap_or("<unknown>");

                    ui.label(format!("Pos: ({}, {})", info.x, info.y));
                    ui.label(format!("Cell: {} (id {})", cell_name, info.cell_id));
                    ui.label(format!("Cell temp: {:.1}°", info.cell_temperature));
                    ui.label(format!("Ambient temp: {:.1}°", info.ambient_temperature));
                } else {
                    ui.label("Move cursor over the simulation to inspect.");
                }
            });

        self.window_style
            .apply(egui::Window::new("📁 Files"))
            .open(&mut win_files)
            .default_pos(egui::pos2(340.0, 5.0))
            .default_size(egui::vec2(320.0, 420.0))
            .show(context, |ui| {
                self.window_style.apply_to_ui(ui);
                egui::CollapsingHeader::new("📦 Project")
                    .default_open(true)
                    .show(ui, |ui| {
                        ui.label("Repository:");
                        let url = "https://github.com/wavelet-noise/sand_evolution";
                        if ui.hyperlink(url).clicked() {
                            _ = webbrowser::open(url);
                        }
                    });

                ui.separator();

                egui::CollapsingHeader::new("🗺 Map")
                    .default_open(true)
                    .show(ui, |ui| {
                        ui.heading("Edit");
                        if ui.button("🧹 Clear").clicked() {
                            Self::clear_map(state);
                        }
                        if ui.button("🎲 Generate random (basic)").clicked() {
                            state.generate_simple();
                        }
                        if ui.button("↩ Restore from URL").clicked() {
                            state.diffuse_rgba = state.loaded_rgba.clone();
                            state.reset_temperatures();
                        }

                        ui.separator();
                        ui.heading("Import / Export");
                        if ui.button("💾 Export PNG").clicked() {
                            if let Err(err) = write_to_file(&state.diffuse_rgba) {
                                panic!("Error: {}", err);
                            }
                        }

                        if ui.button("📂 Open PNG").clicked() {
                            let dialog = rfd::AsyncFileDialog::new()
                                .add_filter("Images", &["png"])
                                .pick_file();

                            let event_loop_proxy = event_loop_proxy.clone();
                            self.executor.execute(async move {
                                if let Some(file) = dialog.await {
                                    let bytes = file.read().await;
                                    event_loop_proxy
                                        .send_event(create_event_with_data(bytes))
                                        .ok();
                                }
                            });
                        }
                    });

                ui.separator();

                egui::CollapsingHeader::new("🎬 Scene")
                    .default_open(true)
                    .show(ui, |ui| {
                        ui.heading("Import / Export");
                        if ui.button("💾 Export TOML").clicked() {
                            let toml_text = self.export_scene_to_toml(world);
                            if let Err(err) = scene_to_file(&toml_text) {
                                panic!("Error: {}", err);
                            }
                        }

                        if ui.button("📂 Open TOML").clicked() {
                            let dialog = rfd::AsyncFileDialog::new()
                                .add_filter("TOML", &["toml"])
                                .add_filter("All", &["*"])
                                .pick_file();

                            let event_loop_proxy = event_loop_proxy.clone();
                            self.executor.execute(async move {
                                if let Some(file) = dialog.await {
                                    let bytes = file.read().await;
                                    event_loop_proxy
                                        .send_event(create_event_with_scene(bytes))
                                        .ok();
                                }
                            });
                        }
                    });

                *any_win_hovered |= context.is_pointer_over_area();
            });
        self.win_files = win_files;

        // Limit the maximum size of Script Editor window to the application size
        let screen_height = context.available_rect().height();
        let max_window_height = screen_height * 0.95; // 95% of screen height with a small margin
                                                      // Fix the window size so it cannot become too large
        let fixed_height = max_window_height.min(600.0);

        self.window_style
            .apply(egui::Window::new("📝 Script Editor"))
            .open(&mut win_script_editor)
            .default_pos(egui::pos2(560.0, 5.0))
            .fixed_size(egui::vec2(600.0, fixed_height))
            .show(context, |ui| {
                self.window_style.apply_to_ui(ui);
                use crate::ecs::components::Name;
                use specs::Join;

                // Get the list of all objects (first collect the data)
                let mut object_names: Vec<String> = Vec::new();
                {
                    let names = world.read_storage::<Name>();
                    let entities = world.entities();
                    for (_, name_comp) in (&entities, &names).join() {
                        object_names.push(name_comp.name.clone());
                    }
                }
                object_names.sort();

                // Top panel with object selection
                ui.horizontal(|ui| {
                    ui.label("Object:");
                    egui::ComboBox::from_id_source("object_selector")
                        .width(200.0)
                        .selected_text(&self.selected_object_name)
                        .show_ui(ui, |ui| {
                            for name in &object_names {
                                ui.selectable_value(
                                    &mut self.selected_object_name,
                                    name.clone(),
                                    name,
                                );
                            }
                        });
                });

                if self.selected_object_name != self.last_loaded_object {
                    if let Some(script_text) =
                        self.get_object_script(world, &self.selected_object_name)
                    {
                        self.script = script_text;
                    } else {
                        self.script = "".to_owned();
                    }
                    self.last_loaded_object = self.selected_object_name.clone();
                    self.script_modified = false;
                    // Clear errors when switching objects
                    self.script_error.clear();
                }

                ui.separator();

                // Toolbar
                ui.horizontal(|ui| {
                    // Enable/disable script button
                    if ui
                        .button(if state.toggled {
                            "⏸ Disable"
                        } else {
                            "▶ Enable"
                        })
                        .clicked()
                    {
                        state.toggled = !state.toggled;
                    }

                    ui.separator();

                    // Export/import buttons
                    if ui.button("📤 Export").clicked() {
                        code_to_file(self.script.as_str());
                    }

                    if ui.button("📥 Import").clicked() {
                        let dialog = rfd::AsyncFileDialog::new()
                            .add_filter("Text", &["txt"])
                            .add_filter("All", &["*"])
                            .pick_file();

                        let event_loop_proxy = event_loop_proxy.clone();
                        self.executor.execute(async move {
                            if let Some(file) = dialog.await {
                                let bytes = file.read().await;
                                event_loop_proxy
                                    .send_event(create_event_with_text(bytes))
                                    .ok();
                            }
                        });
                    }

                    ui.separator();

                    // Button to open log window
                    if ui.button("📋 Log").clicked() {
                        self.show_log_window = true;
                    }
                });

                // Error display (always reserve space to avoid losing focus)
                ui.separator();
                // Always reserve a fixed height for the error area
                // This prevents UI rebuild and loss of focus
                let error_area_height = ui.text_style_height(&egui::TextStyle::Body) + 8.0;
                let error_id = egui::Id::new(format!("script_error_{}", self.selected_object_name));
                let (_id, error_rect) =
                    ui.allocate_space(egui::vec2(ui.available_width(), error_area_height));

                // Show error only if it exists, using the reserved space
                // Use a stable ID to prevent rebuild
                // Don't call allocate_ui_at_rect when there's no error to avoid rebuild
                if !self.script_error.is_empty() {
                    ui.allocate_ui_at_rect(error_rect, |ui| {
                        ui.push_id(error_id, |ui| {
                            ui.horizontal(|ui| {
                                ui.colored_label(
                                    egui::Color32::from_rgb(255, 100, 100),
                                    "⚠ Error:",
                                );
                                ui.label(&self.script_error);
                            });
                        });
                    });
                }

                ui.separator();

                // Code editor with improved interface
                // Use a stable ID to prevent rebuild
                let script_label_id =
                    egui::Id::new(format!("script_label_{}", self.selected_object_name));
                ui.push_id(script_label_id, |ui| {
                    ui.label(format!("Script: {}", self.selected_object_name));
                });

                // Improved editor with better size
                // Use a stable ID to preserve focus
                // Stretch the editor to full available height
                let available_height = ui.available_height();
                egui::ScrollArea::vertical()
                    .auto_shrink([false, false])
                    .enable_scrolling(true)
                    .always_show_scroll(true)
                    .max_height(available_height)
                    .show(ui, |ui| {
                        // Use a stable ID based on object name to preserve focus
                        let text_edit_id =
                            egui::Id::new(format!("script_editor_{}", self.selected_object_name));
                        // Allocate space for TextEdit so it stretches vertically
                        let (_, text_edit_rect) =
                            ui.allocate_space(egui::vec2(ui.available_width(), available_height));
                        ui.allocate_ui_at_rect(text_edit_rect, |ui| {
                            let text_edit = egui::TextEdit::multiline(&mut self.script)
                                .id(text_edit_id)
                                .font(egui::TextStyle::Monospace)
                                .desired_width(f32::INFINITY);

                            let response = ui.add(text_edit);

                            // Track changes
                            if response.changed() {
                                self.script_modified = true;
                            }

                            // Handle clipboard paste if TextEdit is focused
                            if response.has_focus() {
                                let modifiers = ui.input().modifiers;
                                let paste_pressed = (modifiers.command || modifiers.ctrl)
                                    && ui.input().key_pressed(egui::Key::V);

                                if paste_pressed {
                                    #[cfg(target_arch = "wasm32")]
                                    {
                                        // In browser use async API
                                        let event_loop_proxy = event_loop_proxy.clone();
                                        self.executor.execute(async move {
                                            if let Ok(text) =
                                                crate::copy_text_from_clipboard_async().await
                                            {
                                                let _ = event_loop_proxy.send_event(
                                                    UserEventInfo::TextImport(text.into_bytes()),
                                                );
                                            }
                                        });
                                    }

                                    #[cfg(not(target_arch = "wasm32"))]
                                    {
                                        // On desktop use synchronous API
                                        if let Ok(text) = crate::copy_text_from_clipboard() {
                                            // Insert text at current cursor position or at the end
                                            // For simplicity, insert at the end, as getting cursor position is difficult
                                            self.script.push_str(&text);
                                            self.script_modified = true;
                                        }
                                    }
                                }
                            }
                        });
                    });

                // Script information (use a stable ID to prevent rebuild)
                ui.separator();
                let stats_id = egui::Id::new(format!("script_stats_{}", self.selected_object_name));
                ui.push_id(stats_id, |ui| {
                    ui.horizontal(|ui| {
                        let line_count = self.script.lines().count();
                        let char_count = self.script.chars().count();
                        ui.label(format!(
                            "Lines: {} | Characters: {}",
                            line_count, char_count
                        ));
                    });
                });

                *any_win_hovered |= context.is_pointer_over_area()
            });
        self.win_script_editor = win_script_editor;

        // Script Log Window
        self.window_style
            .apply(egui::Window::new("Script Log"))
            .open(&mut self.show_log_window)
            .default_pos(egui::pos2(1160.0, 5.0))
            .default_size(egui::vec2(400.0, 500.0))
            .show(context, |ui| {
                self.window_style.apply_to_ui(ui);
                ui.horizontal(|ui| {
                    if ui.button("Clear").clicked() {
                        self.script_log.borrow_mut().clear();
                    }
                    ui.label(format!("Messages: {}", self.script_log.borrow().len()));
                });

                ui.separator();

                // Display logs (last 30 entries from circular buffer)
                egui::ScrollArea::vertical()
                    .auto_shrink([false, false])
                    .stick_to_bottom(true)
                    .show(ui, |ui| {
                        let logs = self.script_log.borrow();
                        // VecDeque already contains only the last 30 entries thanks to the circular buffer
                        for (index, log_entry) in logs.iter().enumerate() {
                            ui.push_id(index, |ui| {
                                ui.label(log_entry);
                            });
                        }
                    });

                *any_win_hovered |= context.is_pointer_over_area()
            });

        self.window_style
            .apply(egui::Window::new("⏱ Simulation"))
            .open(&mut win_simulation)
            .default_pos(egui::pos2(5.0, 5.0))
            .default_size(egui::vec2(280.0, 460.0))
            .resizable(true)
            .show(context, |ui| {
                self.window_style.apply_to_ui(ui);
                // Simulation Configuration
                ui.heading("Simulation control");
                ui.horizontal(|ui| {
                    let is_running = !self.simulation_paused;
                    if ui
                        .button(if is_running { "⏸ Pause" } else { "▶ Play" })
                        .clicked()
                    {
                        self.simulation_paused = !self.simulation_paused;
                    }

                    let can_step = self.simulation_steps_per_second > 0;

                    let mut step1 = ui.add_enabled(can_step, egui::Button::new("Step ×1"));
                    if !can_step {
                        step1 = step1.on_hover_text("Increase speed above 0 to enable stepping");
                    }
                    if step1.clicked() {
                        self.pending_simulation_steps += 1;
                    }

                    let mut step10 = ui.add_enabled(can_step, egui::Button::new("Step ×10"));
                    if !can_step {
                        step10 = step10.on_hover_text("Increase speed above 0 to enable stepping");
                    }
                    if step10.clicked() {
                        self.pending_simulation_steps += 10;
                    }
                });

                ui.add_space(4.0);
                ui.heading("Simulation speed");
                ui.add(
                    egui::Slider::new(&mut self.simulation_steps_per_second, 1..=480)
                        // Keep slider limited, but don't clamp the value when it's typed/edited.
                        .clamp_to_range(false)
                        .text("Steps per second"),
                );

                ui.add_space(6.0);
                ui.heading("Heat diffusion");
                ui.add(
                    egui::Slider::new(&mut self.cell_diffusion_iterations, 1..=32)
                        .clamp_to_range(true)
                        .text("Cell diffusion iterations / tick"),
                );

                ui.separator();
                ui.heading("Temperature");
                ui.add(
                    egui::Slider::new(
                        &mut state.global_temperature,
                        crate::state::TEMP_MIN..=crate::state::TEMP_MAX,
                    )
                    // Keep slider limited, but allow any numeric input via the slider's value editor.
                    .clamp_to_range(false)
                    .text("Global temperature (°)"),
                );
                if ui.button("Reset global temperature").clicked() {
                    state.global_temperature = 21.0;
                }

                ui.separator();
                ui.heading("Day / Night");
                ui.horizontal(|ui| {
                    ui.checkbox(&mut state.day_night.paused, "Pause cycle");
                    if ui.button("Reset").clicked() {
                        state.day_night.time_of_day_seconds = 0.0;
                    }
                });
                ui.add(
                    egui::Slider::new(&mut state.day_night.day_length_seconds, 5.0..=600.0)
                        .clamp_to_range(false)
                        .text("Day length (s)"),
                );
                ui.add(
                    egui::Slider::new(&mut state.day_night.speed, 0.0..=20.0)
                        .clamp_to_range(false)
                        .text("Speed"),
                );
                ui.label(format!(
                    "sim_time: {:.1}s | time_of_day: {:.1}s",
                    state.sim_time_seconds, state.day_night.time_of_day_seconds
                ));

                ui.separator();
                ui.label(format!(
                    "fps: {}",
                    compact_number_string(fps_meter.next() as f32)
                ));
                let sim_step_avg_time_str = if self.simulation_paused
                    || self.simulation_steps_per_second <= 0
                {
                    "Simulation Step Avg Time: ON PAUSE".to_string()
                } else {
                    format!(
                        "Simulation Step Avg Time: {:.1} ms.",
                        upd_result.simulation_step_average_time
                    )
                };
                ui.label(sim_step_avg_time_str);
                ui.label(format!(
                    "Frame Processing Time: {:.1} ms.",
                    upd_result.update_time
                ));
                if upd_result.dropping {
                    ui.colored_label(Color32::from_rgb(255, 0, 0), "frame drop");
                } else {
                    ui.label("running ok");
                }

                // Structure Spawning
                ui.separator();
                ui.heading("Structure Spawning");
                ui.add(
                    egui::Slider::new(&mut self.number_of_structures_to_add, 0..=10000)
                        .clamp_to_range(false)
                        .text("Number of structures to add"),
                );
                ui.label("Click to add");

                if ui.button("Wooden platforms").clicked() {
                    self.spawn_platforms(state);
                }

                if ui.button("Cubes").clicked() {
                    self.spawn_blocks(state);
                }

                *any_win_hovered |= context.is_pointer_over_area()
            });
        self.win_simulation = win_simulation;

        // Graphics settings window
        let mut win_graphics: bool = self.win_graphics;
        self.window_style
            .apply(egui::Window::new("🎛 Graphics"))
            .open(&mut win_graphics)
            .default_pos(egui::pos2(300.0, 10.0))
            .default_size(egui::vec2(320.0, 220.0))
            .resizable(true)
            .show(context, |ui| {
                self.window_style.apply_to_ui(ui);
                ui.heading("Shadows");
                ui.add_space(6.0);

                ui.add(
                    // 0..1 = normal strength, 1..2 = push towards pure black.
                    egui::Slider::new(&mut state.day_night.shadow_strength, 0.0..=2.0)
                        .clamp_to_range(false)
                        .text("Strength"),
                );

                ui.add(
                    egui::Slider::new(&mut state.day_night.shadow_length_steps, 1.0..=64.0)
                        .clamp_to_range(false)
                        .text("Length (steps)"),
                );

                ui.add(
                    egui::Slider::new(&mut state.day_night.shadow_distance_falloff, 0.0..=4.0)
                        .clamp_to_range(false)
                        .text("Distance falloff"),
                );

                ui.label("Tip: falloff=0 disables distance attenuation.");

                ui.separator();
                ui.heading("Background");
                ui.add_space(6.0);
                ui.add(
                    egui::Slider::new(&mut state.world_settings.bg_saturation, 0.0..=1.0)
                        .clamp_to_range(false)
                        .text("Saturation"),
                );
                ui.add(
                    egui::Slider::new(&mut state.world_settings.bg_brightness, 0.0..=5.0)
                        .clamp_to_range(false)
                        .text("Brightness"),
                );
                ui.label("0 = grayscale, 1 = full color.");
                *any_win_hovered |= context.is_pointer_over_area();
            });
        self.win_graphics = win_graphics;
        // Separate window for GitHub templates / projects
        self.window_style
            .apply(egui::Window::new("🧩 Templates"))
            .open(&mut win_templates)
            .default_pos(egui::pos2(780.0, 5.0))
            .default_size(egui::vec2(420.0, 360.0))
            .show(context, |ui| {
                self.window_style.apply_to_ui(ui);
                ui.heading("GitHub templates");

                ui.separator();

                // Visual feedback while loading
                if self.project_loading {
                    ui.horizontal(|ui| {
                        ui.add(egui::Spinner::new());
                        ui.label("Loading projects from GitHub…");
                    });
                }

                if !self.project_error.is_empty() {
                    ui.colored_label(Color32::from_rgb(255, 0, 0), &self.project_error);
                }

                ui.separator();

                // Two-column layout: list on the left, details on the right
                ui.columns(2, |columns| {
                    // LEFT: scrollable, clean list of templates (no extra frames)
                    egui::ScrollArea::vertical()
                        .auto_shrink([false, true])
                        .show(&mut columns[0], |ui| {
                            for (idx, project) in self.projects.iter().enumerate() {
                                let is_selected = self.selected_project == Some(idx);

                                let label_text = project.display_name.clone();

                                if ui
                                    .selectable_label(is_selected, label_text)
                                    .on_hover_text(format!(
                                        "id: {}\nscript: {}\n{}",
                                        project.id,
                                        project.script_url,
                                        project
                                            .image_url
                                            .as_deref()
                                            .unwrap_or("no background image")
                                    ))
                                    .clicked()
                                {
                                    self.selected_project = Some(idx);
                                }
                            }
                        });

                    let right = &mut columns[1];

                    // RIGHT: details + actions for currently selected template
                    if let Some(idx) = self.selected_project {
                        if idx < self.projects.len() {
                            // Clone to avoid holding an immutable borrow of `self`
                            let project = self.projects[idx].clone();

                            right.heading("Selected template");
                            right.label(egui::RichText::new(&project.display_name).strong());

                            right.add_space(4.0);
                            right.label("Script URL:");
                            let mut script_url_display = project.script_url.clone();
                            right.add(
                                egui::TextEdit::multiline(&mut script_url_display)
                                    .desired_width(right.available_width())
                                    .font(egui::TextStyle::Monospace)
                                    .interactive(false),
                            );

                            right.add_space(4.0);
                            match &project.image_url {
                                Some(url) => {
                                    right.label("Background URL:");
                                    let mut bg_url_display = url.clone();
                                    right.add(
                                        egui::TextEdit::multiline(&mut bg_url_display)
                                            .desired_width(right.available_width())
                                            .font(egui::TextStyle::Monospace)
                                            .interactive(false),
                                    );
                                }
                                None => {
                                    right.label(
                                        egui::RichText::new("Background: none").small().italics(),
                                    );
                                }
                            }

                            right.separator();

                            // Primary action: load template
                            if right
                                .add_sized(
                                    egui::vec2(right.available_width(), 24.0),
                                    egui::Button::new("Apply template"),
                                )
                                .clicked()
                            {
                                if !self.project_loading {
                                    self.start_load_project_from_github(idx, event_loop_proxy);
                                }
                            }

                            // Generate share URL (only shows it; real copy only on native)
                            if right
                                .add_sized(
                                    egui::vec2(right.available_width(), 24.0),
                                    egui::Button::new("Generate load URL"),
                                )
                                .clicked()
                            {
                                let mut full_url =
                                    "https://wavelet-noise.github.io/sand_evolution/".to_owned();

                                if let Some(bg_url) = project.image_url.as_ref() {
                                    full_url.push_str("?save=");
                                    full_url.push_str(bg_url);
                                    full_url.push_str("&script_file=");
                                    full_url.push_str(&project.script_url);
                                } else {
                                    full_url.push_str("?script_file=");
                                    full_url.push_str(&project.script_url);
                                }

                                // Remember URL to show it in read‑only field
                                self.last_load_url = full_url.clone();

                                // On native builds also copy to clipboard
                                #[cfg(not(target_arch = "wasm32"))]
                                {
                                    let _ = copy_text_to_clipboard(&full_url);
                                }
                            }

                            if !self.last_load_url.is_empty() {
                                right.add_space(4.0);
                                right.label("Load URL:");
                                right.add(
                                    egui::TextEdit::singleline(&mut self.last_load_url)
                                        .desired_width(right.available_width())
                                        .interactive(false),
                                );
                            }
                        } else {
                            right.label("No template selected.");
                        }
                    } else {
                        right.label("Click a template on the left to see details.");
                    }
                });

                *any_win_hovered |= context.is_pointer_over_area()
            });

        // Auto-fetch projects when window is open but projects haven't been fetched yet
        if win_templates && !self.projects_fetched && !self.project_loading {
            self.start_fetch_github_projects(event_loop_proxy);
        }

        self.win_templates = win_templates;

        // Floating palette window (movable, positioned at bottom by default)
        let input_rect = context.input().screen_rect;
        let palette_y = (input_rect.height() - 70.0).max(50.0);
        let palette_width = (input_rect.width() - 20.0).max(400.0);
        self.window_style
            .apply(egui::Window::new("🎨 Palette"))
            .open(&mut win_palette)
            .default_pos(egui::pos2(10.0, palette_y))
            .fixed_size(egui::vec2(palette_width, 50.0))
            .resizable(true)
            .collapsible(true)
            .show(context, |ui| {
                self.window_style.apply_to_ui(ui);
                ui.horizontal(|ui| {
                    // Brush size slider
                    ui.spacing_mut().slider_width = 100.0;
                    ui.add(
                        egui::Slider::new(&mut self.number_of_cells_to_add, 1..=2000)
                            .clamp_to_range(false)
                            .show_value(true)
                            .text("🖌"),
                    );

                    ui.separator();

                    // Scrollable horizontal palette
                    egui::ScrollArea::horizontal()
                        .auto_shrink([false, false])
                        .show(ui, |ui| {
                            ui.horizontal(|ui| {
                                ui.spacing_mut().item_spacing = egui::vec2(4.0, 4.0);

                                // Iterate over all registered cell types (except id 0 / Void).
                                for cell in state.pal_container.pal.iter() {
                                    let id = cell.id();
                                    if id == 0 {
                                        continue;
                                    }

                                    let dict_name = cell.name();
                                    let color = cell.display_color();
                                    let color32 = Color32::from_rgb(color[0], color[1], color[2]);
                                    let is_selected = self.selected_option == dict_name;

                                    // Button with color square and name
                                    let button_response =
                                        ui.allocate_ui(egui::vec2(90.0, 28.0), |ui| {
                                            let (rect, response) = ui.allocate_exact_size(
                                                egui::vec2(90.0, 28.0),
                                                egui::Sense::click(),
                                            );

                                            if ui.is_rect_visible(rect) {
                                                let painter = ui.painter();

                                                // Background
                                                let bg_color = if is_selected {
                                                    Color32::from_rgb(70, 90, 120)
                                                } else if response.hovered() {
                                                    Color32::from_rgb(55, 55, 65)
                                                } else {
                                                    Color32::from_rgb(45, 45, 55)
                                                };

                                                painter.rect_filled(
                                                    rect,
                                                    egui::Rounding::same(4.0),
                                                    bg_color,
                                                );

                                                // Border for selected
                                                if is_selected {
                                                    painter.rect_stroke(
                                                        rect,
                                                        egui::Rounding::same(4.0),
                                                        egui::Stroke::new(
                                                            2.0,
                                                            Color32::from_rgb(100, 180, 255),
                                                        ),
                                                    );
                                                }

                                                // Color square (left side)
                                                let color_rect = egui::Rect::from_min_size(
                                                    rect.min + egui::vec2(4.0, 4.0),
                                                    egui::vec2(20.0, 20.0),
                                                );
                                                painter.rect_filled(
                                                    color_rect,
                                                    egui::Rounding::same(3.0),
                                                    color32,
                                                );

                                                // Text (right of color)
                                                painter.text(
                                                    rect.min
                                                        + egui::vec2(28.0, rect.height() / 2.0),
                                                    egui::Align2::LEFT_CENTER,
                                                    dict_name,
                                                    egui::FontId::proportional(11.0),
                                                    Color32::WHITE,
                                                );
                                            }

                                            response
                                        });

                                    if button_response.inner.clicked() {
                                        self.selected_option = dict_name.to_string();
                                    }
                                }
                            });
                        });
                });

                *any_win_hovered |= context.is_pointer_over_area()
            });

        self.win_palette = win_palette;
        self.win_hover = win_hover;
    }

    fn show_toasts(&mut self, ctx: &Context) {
        use egui::Color32;

        let mut remaining_toasts = Vec::new();
        std::mem::swap(&mut remaining_toasts, &mut self.editor_state.toasts);

        for (i, toast) in remaining_toasts.iter().enumerate() {
            let color = match toast.level {
                crate::editor::state::ToastLevel::Info => Color32::from_rgb(100, 150, 255),
                crate::editor::state::ToastLevel::Warning => Color32::from_rgb(255, 200, 100),
                crate::editor::state::ToastLevel::Error => Color32::from_rgb(255, 50, 50),
            };

            self.window_style
                .apply(egui::Window::new(""))
                .title_bar(false)
                .resizable(false)
                .fixed_pos(egui::pos2(10.0, 500.0 + (i as f32 * 40.0)))
                .show(ctx, |ui| {
                    self.window_style.apply_to_ui(ui);
                    ui.colored_label(color, &toast.message);
                });
        }

        std::mem::swap(&mut remaining_toasts, &mut self.editor_state.toasts);
    }

    pub fn compile_script(
        &mut self,
        rhai: &mut RhaiResourceStorage,
        world: &mut specs::World,
        script_entity: specs::Entity,
    ) {
        let script_text = self.script.clone();
        let result = rhai
            .engine
            .compile_with_scope(&mut rhai.scope, script_text.as_str());
        match result {
            Ok(value) => {
                let mut scripts = world.write_storage::<crate::ecs::components::Script>();
                if let Some(script) = scripts.get_mut(script_entity) {
                    script.ast = Some(value);
                    script.script = script_text;
                    script.raw = false;
                }
                self.script_error = "".to_owned();
            }
            Err(err) => {
                let mut scripts = world.write_storage::<crate::ecs::components::Script>();
                if let Some(script) = scripts.get_mut(script_entity) {
                    script.ast = None;
                    script.raw = true;
                }
                self.script_error = err.to_string()
            }
        }
    }

    fn spawn_blocks(&mut self, state: &mut State) {
        for _ in 0..self.number_of_structures_to_add {
            let mut buf = [0u8; 4];
            _ = getrandom::getrandom(&mut buf);

            let nx = (((buf[0] as u32) << 8) | buf[1] as u32) % cs::SECTOR_SIZE.x as u32;
            let ny = (((buf[2] as u32) << 8) | buf[3] as u32) % cs::SECTOR_SIZE.y as u32;

            for x in 0..20 {
                for y in 0..20 {
                    state.diffuse_rgba.put_pixel(
                        clamp(nx + x, 0, cs::SECTOR_SIZE.x as u32 - 1),
                        clamp(ny + y, 0, cs::SECTOR_SIZE.y as u32 - 1),
                        image::Luma([Wood::id()]),
                    );
                }
            }
        }
    }

    fn spawn_platforms(&mut self, state: &mut State) {
        for _ in 0..self.number_of_structures_to_add {
            let mut buf = [0u8; 4];
            _ = getrandom::getrandom(&mut buf);

            let nx = (((buf[0] as u32) << 8) | buf[1] as u32) % cs::SECTOR_SIZE.x as u32;
            let ny = (((buf[2] as u32) << 8) | buf[3] as u32) % cs::SECTOR_SIZE.y as u32;

            for x in 0..50 {
                state.diffuse_rgba.put_pixel(
                    clamp(nx + x, 0, cs::SECTOR_SIZE.x as u32 - 1),
                    clamp(ny, 0, cs::SECTOR_SIZE.y as u32 - 1),
                    image::Luma([Wood::id()]),
                );
            }
        }
    }

    fn clear_map(state: &mut State) {
        state.diffuse_rgba = image::GrayImage::from_fn(
            cs::SECTOR_SIZE.x as u32,
            cs::SECTOR_SIZE.y as u32,
            |x, y| {
                if x > 1
                    && y > 1
                    && x < cs::SECTOR_SIZE.x as u32 - 2
                    && y < cs::SECTOR_SIZE.y as u32 - 2
                {
                    return image::Luma([Void::id()]);
                } else {
                    return image::Luma([Stone::id()]);
                }
            },
        );
        state.reset_temperatures();
    }

    pub fn new() -> Self {
        Self::new_with_log(Rc::new(RefCell::new(VecDeque::with_capacity(30))))
    }

    pub fn new_with_log(script_log: Rc<RefCell<VecDeque<String>>>) -> Self {
        let number_of_cells_to_add = 500;
        let number_of_structures_to_add = 100;
        let selected_option: String = "water".to_owned();
        let options: Vec<String> = Vec::new();
        let executor = Executor::new();
        Self {
            number_of_cells_to_add,
            number_of_structures_to_add,
            simulation_steps_per_second: 240,
            simulation_paused: false,
            pending_simulation_steps: 0,
            selected_option,
            options,
            cursor_position: None,
            pressed: false,
            hovered: false,
            hover_info: None,
            executor,
            script: r"let a = 0; for i in 0..10 { a += i; };".to_owned(),
            selected_object_name: "World Script".to_owned(),
            last_loaded_object: String::new(),
            script_modified: false,
            script_error: "".to_owned(),
            need_to_recompile: true,

            win_files: true,
            win_script_editor: false,
            win_simulation: false,
            win_graphics: false,
            win_templates: false,
            win_palette: true, // Palette window open by default
            win_hover: true,

            projects: crate::projects::demo_projects(),
            selected_project: None,
            project_loading: false,
            project_error: String::new(),
            projects_fetched: false,

            last_load_url: String::new(),

            editor_state: EditorState::new(),
            undo_redo: UndoRedo::new(),

            script_log,
            show_log_window: false,

            display_mode: DisplayMode::Normal,
            heat_vis_mode: HeatVisMode::Combined,
            cell_diffusion_iterations: 1,
            window_style: WindowStyleManager::default(),
        }
    }
}

fn create_event_with_data(bytes: Vec<u8>) -> UserEventInfo {
    UserEventInfo::ImageImport(bytes)
}

fn create_event_with_text(bytes: Vec<u8>) -> UserEventInfo {
    UserEventInfo::TextImport(bytes)
}

fn create_event_with_scene(bytes: Vec<u8>) -> UserEventInfo {
    UserEventInfo::SceneImport(bytes)
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
struct SceneToml {
    #[serde(default = "default_scene_version")]
    version: u32,
    #[serde(default)]
    entity: Vec<SceneEntityToml>,
}

fn default_scene_version() -> u32 {
    1
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
struct SceneEntityToml {
    name: String,
    position: Option<Vec2Toml>,
    rotation: Option<f32>,
    scale: Option<Vec2Toml>,
    velocity: Option<Vec2Toml>,
    script: Option<ScriptToml>,
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
struct Vec2Toml {
    x: f32,
    y: f32,
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
struct ScriptToml {
    code: String,
    script_type: Option<String>,
    run_once: Option<bool>,
}

impl EvolutionApp {
    pub fn export_scene_to_toml(&self, world: &specs::World) -> String {
        use crate::ecs::components::{Name, Position, Rotation, Scale, Script, Velocity};
        use specs::Join;

        let entities = world.entities();
        let names = world.read_storage::<Name>();
        let positions = world.read_storage::<Position>();
        let rotations = world.read_storage::<Rotation>();
        let scales = world.read_storage::<Scale>();
        let velocities = world.read_storage::<Velocity>();
        let scripts = world.read_storage::<Script>();

        let mut out: Vec<SceneEntityToml> = Vec::new();
        for (entity, name) in (&entities, &names).join() {
            let pos = positions.get(entity).map(|p| Vec2Toml { x: p.x, y: p.y });
            let rot = rotations.get(entity).map(|r| r.angle);
            let scale = scales.get(entity).map(|s| Vec2Toml { x: s.x, y: s.y });
            let vel = velocities.get(entity).map(|v| Vec2Toml { x: v.x, y: v.y });
            let script = scripts.get(entity).map(|s| ScriptToml {
                code: s.script.clone(),
                script_type: Some(match s.script_type {
                    crate::ecs::components::ScriptType::World => "World".to_owned(),
                    crate::ecs::components::ScriptType::Entity => "Entity".to_owned(),
                }),
                run_once: Some(s.run_once),
            });

            out.push(SceneEntityToml {
                name: name.name.clone(),
                position: pos,
                rotation: rot,
                scale,
                velocity: vel,
                script,
            });
        }

        out.sort_by(|a, b| a.name.cmp(&b.name));

        let scene = SceneToml {
            version: 1,
            entity: out,
        };
        toml::to_string_pretty(&scene).unwrap_or_else(|_| String::new())
    }

    pub fn import_scene_from_toml(
        &mut self,
        world: &mut specs::World,
        toml_text: &str,
    ) -> Result<(), String> {
        use crate::ecs::components::{
            Name, Position, Rotation, Scale, Script, ScriptType, Velocity,
        };
        use specs::{Builder, Join, WorldExt};

        let parsed: SceneToml = toml::from_str(toml_text).map_err(|e| e.to_string())?;

        // If the imported "world" has no entity list (common for older / template files),
        // reset to the current hardcoded defaults instead of leaving an empty scene.
        if parsed.entity.is_empty() {
            self.reset_world_entities_to_hardcoded(world);
            return Ok(());
        }

        // Clear editor selection to avoid dangling entity handles.
        self.editor_state.selected_entities.clear();

        // Delete all entities in the world (resources stay intact).
        let to_delete: Vec<specs::Entity> = {
            let entities = world.entities();
            (&entities).join().collect()
        };
        for e in to_delete {
            let _ = world.delete_entity(e);
        }

        // Recreate entities from TOML.
        let mut has_world_script = false;
        for e in parsed.entity {
            if e.name == "World Script" {
                has_world_script = true;
            }

            let mut builder = world.create_entity().with(Name {
                name: e.name.clone(),
            });

            if let Some(p) = e.position {
                builder = builder.with(Position { x: p.x, y: p.y });
            }
            if let Some(angle) = e.rotation {
                builder = builder.with(Rotation { angle });
            }
            if let Some(s) = e.scale {
                builder = builder.with(Scale { x: s.x, y: s.y });
            }
            if let Some(v) = e.velocity {
                builder = builder.with(Velocity { x: v.x, y: v.y });
            }
            if let Some(script) = e.script {
                let st = match script.script_type.as_deref() {
                    Some("World") => ScriptType::World,
                    Some("Entity") => ScriptType::Entity,
                    Some(other) if other.eq_ignore_ascii_case("world") => ScriptType::World,
                    Some(other) if other.eq_ignore_ascii_case("entity") => ScriptType::Entity,
                    _ => {
                        if e.name == "World Script" {
                            ScriptType::World
                        } else {
                            ScriptType::Entity
                        }
                    }
                };

                builder = builder.with(Script {
                    script: script.code,
                    ast: None,
                    raw: true,
                    script_type: st,
                    run_once: script.run_once.unwrap_or(false),
                    has_run: false,
                });
            }

            builder.build();
        }

        if !has_world_script {
            world
                .create_entity()
                .with(Name {
                    name: "World Script".to_owned(),
                })
                .with(Script {
                    script: "".to_owned(),
                    ast: None,
                    raw: true,
                    script_type: ScriptType::World,
                    run_once: false,
                    has_run: false,
                })
                .build();
        }

        // After importing, reset selection to world script so script UI has a stable target.
        self.selected_object_name = "World Script".to_owned();
        self.last_loaded_object.clear();
        self.need_to_recompile = true;
        Ok(())
    }

    pub fn reset_world_entities_to_hardcoded(&mut self, world: &mut specs::World) {
        use specs::{Join, WorldExt};

        // Clear editor selection to avoid dangling entity handles.
        self.editor_state.selected_entities.clear();

        // Delete all entities in the world (resources stay intact).
        let to_delete: Vec<specs::Entity> = {
            let entities = world.entities();
            (&entities).join().collect()
        };
        for e in to_delete {
            let _ = world.delete_entity(e);
        }

        // Recreate the default hardcoded entities (World Script, Dummy, Cooler, ...).
        crate::init_hardcoded_entities(world);

        // Reset selection to world script so script UI has a stable target.
        self.selected_object_name = "World Script".to_owned();
        self.last_loaded_object.clear();
        self.need_to_recompile = true;
    }
}
