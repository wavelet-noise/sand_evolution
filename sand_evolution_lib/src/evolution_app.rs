use cgmath::num_traits::clamp;
use egui::{ComboBox, Context};
use winit::{dpi::PhysicalPosition, event_loop::EventLoopProxy};

use crate::{
    cells::{stone::Stone, void::Void, wood::Wood},
    cs,
    export_file::write_to_file,
    fps_meter::FpsMeter,
    state::{State, UpdateResult},
};

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
    pub simulation_steps_per_frame: i32,
    pub selected_option: String,
    pub options: Vec<String>,
    pub cursor_position: Option<PhysicalPosition<f64>>,
    pub pressed: bool,
    pub hovered: bool,
    executor: Executor,
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
}

impl EvolutionApp {
    pub(crate) fn ui(
        &mut self,
        context: &Context,
        state: &mut State,
        fps_meter: &mut FpsMeter,
        upd_result: &UpdateResult,
        event_loop_proxy: &EventLoopProxy<UserEventInfo>,
        any_win_hovered: &mut bool
    ) {
        egui::Window::new("Monitor")
            .default_pos(egui::pos2(340.0, 5.0))
            .fixed_size(egui::vec2(200.0, 100.0))
            .show(context, |ui| {    
                    
                let url = "https://github.com/wavelet-noise/sand_evolution";
                if ui.hyperlink(url).clicked() {
                    _ = webbrowser::open(url);
                }

                ui.label(
                    [
                        "CO2 level:",
                        compact_number_string(state.prng.carb() as f32).as_str(),
                    ]
                    .join(" "),
                );
                ui.separator();
                ui.label(format!(
                    "fps: {}",
                    compact_number_string(fps_meter.next() as f32)
                ));
                let sim_step_avg_time_str = if self.simulation_steps_per_frame == 0 {
                    "sim. step avg time: ON PAUSE".to_string()
                } else {
                    format!(
                        "sim. step avg time: {:.1} ms.",
                        upd_result.simulation_step_average_time
                    )
                };
                ui.label(sim_step_avg_time_str);
                ui.label(format!("frame time: {:.1} ms.", upd_result.update_time));

                if ui.button("Clear screen").clicked() {
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

                if ui.button("Simple random map").clicked() {
                    state.generate_simple();
                }

                *any_win_hovered |= ui.ui_contains_pointer();
            });

        egui::Window::new("Toolbox")
            .default_pos(egui::pos2(5.0, 5.0))
            .fixed_size(egui::vec2(200., 100.))
            .show(context, |ui| {
                ui.add(
                    egui::Slider::new(&mut self.simulation_steps_per_frame, 0..=10)
                        .text("Simulation steps per frame"),
                );
                ui.heading("Hold left mouse button to spawn particles");
                let combo = ComboBox::from_id_source("dropdown_list")
                    .selected_text(&self.selected_option)
                    .show_ui(ui, |ui| {
                        for option in self.options.iter() {
                            ui.selectable_value(
                                &mut self.selected_option,
                                option.to_string(),
                                option.to_string(),
                            );
                        }
                    });

                ui.label(format!("Selected: {}", self.selected_option));

                ui.separator();
                ui.heading("Spawn structures");
                ui.add(
                    egui::Slider::new(&mut self.number_of_structures_to_add, 0..=10000 as i32)
                        .text("Number of structures to add"),
                );
                ui.label("Click to add");

                if ui.button("Wooden platforms").clicked() {
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

                if ui.button("Cubes").clicked() {
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

                ui.separator();

                if ui.button("Export map").clicked() {
                    let result = write_to_file(&state.diffuse_rgba);
                    match result {
                        Ok(()) => {}
                        Err(err) => {
                            // handle the error
                            panic!("Error: {}", err);
                        }
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

                *any_win_hovered |= ui.ui_contains_pointer();
            });
    }

    pub fn new() -> Self {
        let number_of_cells_to_add = 500;
        let number_of_structures_to_add = 100;
        let simulation_steps_per_frame = 5;
        let selected_option: String = "water".to_owned();
        let options: Vec<String> = Vec::new();
        let executor = Executor::new();
        Self {
            number_of_cells_to_add,
            number_of_structures_to_add,
            simulation_steps_per_frame,
            selected_option,
            options,
            cursor_position: None,
            pressed: false,
            hovered: false,
            executor,
        }
    }
}

fn create_event_with_data(bytes: Vec<u8>) -> UserEventInfo {
    UserEventInfo::ImageImport(bytes)
}
