use std::io::ErrorKind;
use cgmath::num_traits::clamp;
use egui::{Color32, ComboBox, Context};
use winit::{dpi::PhysicalPosition, event_loop::EventLoopProxy};

use crate::{cells::{stone::Stone, void::Void, wood::Wood}, copy_text_to_clipboard, cs, export_file::write_to_file, fps_meter::FpsMeter, state::{State, UpdateResult}};
use crate::export_file::code_to_file;

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
    pub selected_option: String,
    pub options: Vec<String>,
    pub cursor_position: Option<PhysicalPosition<f64>>,
    pub pressed: bool,
    pub hovered: bool,
    script: String,
    pub need_to_recompile: bool,
    pub script_error: String,
    executor: Executor,
    pub ast: Option<rhai::AST>
}

pub fn compact_number_string(n: f32) -> String {
    let abs = cgmath::num_traits::abs(n);

    if abs < 999.0 {
        return format!("{}", abs);
    }

    if abs < 999999.0 {
        return format!("{:.2}k", abs as f32 / 1000.0);
    }

    if abs < 999999999.0 {
        return format!("{:.2}M", abs as f32 / 1000000.0);
    }

    if abs < 999999999999.0 {
        return format!("{:.2}G", abs as f32 / 1000000000.0);
    }

    return format!("{:.2}T", abs as f32 / 1000000000000.0);
}

pub enum UserEventInfo {
    ImageImport(Vec<u8>),
    TextImport(Vec<u8>),
}

impl EvolutionApp {

    pub fn get_script(&mut self) -> &str {
        self.script.as_str()
    }

    pub fn set_script(&mut self, value: &str) -> bool {
        self.script = value.to_owned();
        self.need_to_recompile = true;
        true
    }
    pub(crate) fn ui(
        &mut self,
        context: &Context,
        state: &mut State,
        fps_meter: &mut FpsMeter,
        upd_result: &UpdateResult,
        event_loop_proxy: &EventLoopProxy<UserEventInfo>,
        any_win_hovered: &mut bool
    ) {
        egui::Window::new("Configuration")
            .default_pos(egui::pos2(340.0, 5.0))
            .fixed_size(egui::vec2(200.0, 100.0))
            .show(context, |ui| {
                let url = "https://github.com/wavelet-noise/sand_evolution";
                if ui.hyperlink(url).clicked() {
                    _ = webbrowser::open(url);
                }

                ui.separator();

                if ui.button("Clear Map").clicked() {
                    Self::clear_map(state);
                }

                if ui.button("Generate Basic Random Map").clicked() {
                    state.generate_simple();
                }

                if ui.button("Restore Map from URL").clicked() {
                    state.diffuse_rgba = state.loaded_rgba.clone();
                }

                // Map Operations
                ui.separator();
                ui.heading("Map Operations");
                if ui.button("Export map").clicked() {
                    if let Err(err) = write_to_file(&state.diffuse_rgba) {
                        panic!("Error: {}", err);
                    }
                }

                if ui.button("Open map").clicked() {
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

                *any_win_hovered |= context.is_pointer_over_area()
            });

        egui::Window::new("Level script")
            .default_pos(egui::pos2(560.0, 5.0))
            .default_size(egui::vec2(500., 500.))
            .show(context, |ui| {
                ui.text_edit_multiline(&mut self.script);

                if ui.button(if state.toggled { "Disable script" } else { "Enable script" }).clicked() {
                    state.toggled = !state.toggled;
                }
                ui.colored_label(Color32::from_rgb(255, 0,0),&self.script_error);

                if ui.button("Export code").clicked() {
                     code_to_file(self.script.as_str());
                }

                if ui.button("Import code").clicked() {
                    let dialog = rfd::AsyncFileDialog::new()
                        .add_filter("Text", &["txt"])
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

                *any_win_hovered |= context.is_pointer_over_area()
            });

        egui::Window::new("Simulation")
            .default_pos(egui::pos2(5.0, 5.0))
            .fixed_size(egui::vec2(200., 100.))
            .show(context, |ui| {
                // Simulation Configuration
                ui.heading("Pause or simulation speed");
                ui.add(egui::Slider::new(&mut self.simulation_steps_per_second, 0..=240).text("Simulation steps per second"));

                ui.separator();
                ui.label(format!(
                    "fps: {}",
                    compact_number_string(fps_meter.next() as f32)
                ));
                let sim_step_avg_time_str = if self.simulation_steps_per_second == 0 {
                    "Simulation Step Avg Time: ON PAUSE".to_string()
                } else {
                    format!(
                        "Simulation Step Avg Time: {:.1} ms.",
                        upd_result.simulation_step_average_time
                    )
                };
                ui.label(sim_step_avg_time_str);
                ui.label(format!("Frame Processing Time: {:.1} ms.", upd_result.update_time));

                // Particle Spawning
                ui.separator();
                ui.heading("Particle Spawning");
                ui.label("Hold left mouse button to spawn particles");
                ComboBox::from_id_source("dropdown_list")
                    .selected_text(&self.selected_option)
                    .show_ui(ui, |ui| {
                        self.options.iter().for_each(|option| {
                            ui.selectable_value(&mut self.selected_option, option.to_string(), option.to_string());
                        });
                    });
                ui.label(format!("Selected: {}", self.selected_option));

                // Structure Spawning
                ui.separator();
                ui.heading("Structure Spawning");
                ui.add(egui::Slider::new(&mut self.number_of_structures_to_add, 0..=10000).text("Number of structures to add"));
                ui.label("Click to add");

                if ui.button("Wooden platforms").clicked() {
                    self.spawn_platforms(state);
                }

                if ui.button("Cubes").clicked() {
                    self.spawn_blocks(state);
                }

                *any_win_hovered |= context.is_pointer_over_area()
            });
    }

    pub fn compile_script(&mut self, state: &mut State) {
        let result = state.rhai.compile_with_scope(&mut state.rhai_scope, self.script.as_str());
        match result {
            Ok(value) => {
                self.ast = Some(value);
                self.script_error = "".to_owned();
            }
            Err(err) => {
                self.ast = None;
                self.script_error = err.to_string()
            }
        }
    }

    fn spawn_blocks(&mut self, state: &mut State) {
        for _ in 0..self.number_of_structures_to_add {
            let mut buf = [0u8; 4];
            _ = getrandom::getrandom(&mut buf);

            let nx =
                (((buf[0] as u32) << 8) | buf[1] as u32) % cs::SECTOR_SIZE.x as u32;
            let ny =
                (((buf[2] as u32) << 8) | buf[3] as u32) % cs::SECTOR_SIZE.y as u32;

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

            let nx =
                (((buf[0] as u32) << 8) | buf[1] as u32) % cs::SECTOR_SIZE.x as u32;
            let ny =
                (((buf[2] as u32) << 8) | buf[3] as u32) % cs::SECTOR_SIZE.y as u32;

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
    }

    pub fn new() -> Self {
        let number_of_cells_to_add = 500;
        let number_of_structures_to_add = 100;
        let selected_option: String = "water".to_owned();
        let options: Vec<String> = Vec::new();
        let executor = Executor::new();
        Self {
            number_of_cells_to_add,
            number_of_structures_to_add,
            simulation_steps_per_second: 120,
            selected_option,
            options,
            cursor_position: None,
            pressed: false,
            hovered: false,
            executor,
            script: r"let a = 0; for i in 0..10 { a += i; };".to_owned(),

            script_error: "".to_owned(),
            ast: None,
            need_to_recompile: true,
        }
    }
}

fn create_event_with_data(bytes: Vec<u8>) -> UserEventInfo {
    UserEventInfo::ImageImport(bytes)
}

fn create_event_with_text(bytes: Vec<u8>) -> UserEventInfo {
    UserEventInfo::TextImport(bytes)
}
