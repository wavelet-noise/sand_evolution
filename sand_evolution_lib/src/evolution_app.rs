use cgmath::num_traits::clamp;
use egui::{ComboBox, Context};
use winit::dpi::PhysicalPosition;

use crate::{
    cells::wood::Wood,
    cs,
    fps_meter::FpsMeter,
    state::{State, UpdateResult},
};

pub struct EvolutionApp {
    pub number_of_cells_to_add: i32,
    pub number_of_structures_to_add: i32,
    pub simulation_steps_per_frame: i32,
    pub selected_option: String,
    pub options: Vec<String>,
    pub cursor_position: Option<PhysicalPosition<f64>>,
    pub pressed: bool,
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

impl EvolutionApp {
    pub(crate) fn ui(
        &mut self,
        context: &Context,
        state: &mut State,
        fps_meter: &mut FpsMeter,
        upd_result: &UpdateResult,
    ) {
        egui::Window::new("Monitor")
            .default_pos(egui::pos2(340.0, 5.0))
            .fixed_size(egui::vec2(200.0, 100.0))
            .show(context, |ui| {
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
            });

        egui::Window::new("Toolbox")
            .default_pos(egui::pos2(5.0, 5.0))
            .fixed_size(egui::vec2(200., 100.))
            .show(context, |ui| {
                ui.heading("Simulation settings");
                ui.add(
                    egui::Slider::new(&mut self.simulation_steps_per_frame, 0..=50)
                        .text("Simulation steps per frame"),
                );
                ui.separator();
                ui.heading("Spawn particles");
                ui.add(
                    egui::Slider::new(&mut self.number_of_cells_to_add, 0..=10000 as i32)
                        .text("Number of cells to add"),
                );
                ui.label("Click to add");

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

                if ui.button("Spawn").clicked() {
                    let mut buf = [0u8; 10000 + 1];
                    _ = getrandom::getrandom(&mut buf);

                    for i in 0..self.number_of_cells_to_add as usize {
                        let px =
                            (((buf[i] as u32) << 8) | buf[i + 1] as u32) % cs::SECTOR_SIZE.x as u32;
                        let py = cs::SECTOR_SIZE.y as u32 - i as u32 % 32 - 2;
                        state.diffuse_rgba.put_pixel(
                            px,
                            py,
                            image::Luma([state.pal_container.dict[&self.selected_option]]),
                        );
                    }
                }

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
            });
    }

    pub fn new() -> Self {
        let number_of_cells_to_add = 500;
        let number_of_structures_to_add = 100;
        let simulation_steps_per_frame = 5;
        let selected_option: String = "water".to_owned();
        let options: Vec<String> = Vec::new();
        Self {
            number_of_cells_to_add,
            number_of_structures_to_add,
            simulation_steps_per_frame,
            selected_option,
            options,
            cursor_position: None,
            pressed: false,
        }
    }
}
