use cgmath::num_traits::clamp;
use egui::{Color32, ComboBox, Context};
use winit::{dpi::PhysicalPosition, event_loop::EventLoopProxy};
use std::cell::RefCell;
use std::rc::Rc;
use std::collections::VecDeque;

use crate::export_file::code_to_file;
use crate::projects::{ProjectDescription};
use crate::resources::rhai_resource::RhaiResourceStorage;
use crate::{
    cells::{stone::Stone, void::Void, wood::Wood},
    copy_text_to_clipboard, cs,
    export_file::write_to_file,
    fps_meter::FpsMeter,
    state::{State, UpdateResult},
    editor::{EditorState, EditorToolbar, EditorViewport, EditorInspector, EditorHierarchy, UndoRedo, GizmoSystem, InputHandler, AddPanel},
};
use specs::WorldExt;

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
    script: String, // Для обратной совместимости - хранит скрипт выбранного объекта
    pub selected_object_name: String, // Имя выбранного объекта для редактирования
    last_loaded_object: String, // Последний загруженный объект (для отслеживания изменений)
    script_modified: bool, // Флаг изменений скрипта
    pub need_to_recompile: bool,
    pub script_error: String,
    executor: Executor,

    pub w1: bool,
    pub w2: bool,
    pub w3: bool,
    pub w4: bool, // templates / projects window

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
    
    // Script log storage - кольцевой буфер с ограничением в 30 записей
    pub script_log: Rc<RefCell<VecDeque<String>>>,
    pub show_log_window: bool,
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
    ProjectsLoaded(Vec<ProjectDescription>),
    ProjectLoadError(String),
}

impl EvolutionApp {
    pub fn get_script(&mut self) -> &str {
        self.script.as_str()
    }

    pub fn set_script(&mut self, value: &str) -> bool {
        self.script = value.to_owned();
        self.script_modified = true; // Импортированный скрипт считается измененным
        self.need_to_recompile = true;
        true
    }

    /// Получить скрипт объекта по имени из world
    pub fn get_object_script(&self, world: &specs::World, object_name: &str) -> Option<String> {
        use specs::Join;
        use crate::ecs::components::{Name, Script};
        
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

    /// Установить скрипт объекта по имени в world
    pub fn set_object_script(&mut self, world: &mut specs::World, object_name: &str, script: &str) {
        use specs::Join;
        use crate::ecs::components::{Name, Script};
        
        let names = world.read_storage::<Name>();
        let mut scripts = world.write_storage::<Script>();
        let entities = world.entities();
        
        for (entity, name_comp) in (&entities, &names).join() {
            if name_comp.name == object_name {
                if let Some(script_comp) = scripts.get_mut(entity) {
                    script_comp.script = script.to_owned();
                    script_comp.raw = true;
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
        self.editor_state.update_toasts(upd_result.update_time as f32 / 1000.0);
        
        // Handle keyboard shortcuts for editor
        // TODO: Fix input handling for egui 0.19 - temporarily disabled
        // Keyboard shortcuts will be handled through other means
        
        // Handle mouse input for editor
        InputHandler::handle_mouse_input(context, &mut self.editor_state, world, &mut self.undo_redo);
        
        let mut w1: bool = self.w1;
        let mut w2: bool = self.w2;
        let mut w3: bool = self.w3;
        let mut w4: bool = self.w4;
        let mut w6: bool = self.editor_state.show_grid; // Editor viewport window

        // Editor toolbar
        if self.editor_state.show_toolbar {
            egui::Window::new("Editor Toolbar")
                .default_pos(egui::pos2(10.0, 50.0))
                .title_bar(false)
                .resizable(false)
                .show(context, |ui| {
                    EditorToolbar::ui(ui, &mut self.editor_state, &self.undo_redo);
                });
        }
        
        // Editor viewport
        egui::Window::new("Viewport")
            .default_pos(egui::pos2(200.0, 50.0))
            .default_size(egui::vec2(800.0, 600.0))
            .open(&mut w6)
            .show(context, |ui| {
                let viewport_rect = ui.available_rect_before_wrap();
                EditorViewport::ui(ui, &mut self.editor_state, viewport_rect);
                
                // Draw gizmos
                let painter = ui.painter();
                let gizmo = GizmoSystem;
                gizmo.draw(&painter, &self.editor_state, world);
                
                // Status bar
                ui.separator();
                let cursor_pos = context.pointer_latest_pos()
                    .map(|p| (p.x, p.y));
                EditorViewport::status_bar(ui, &self.editor_state, cursor_pos);
            });
        
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
        egui::Window::new("Hierarchy")
            .default_pos(egui::pos2(hierarchy_x, hierarchy_y))
            .default_size(egui::vec2(hierarchy_w, hierarchy_h))
            .show(context, |ui| {
                EditorHierarchy::ui(ui, &mut self.editor_state, world);
            });
        
        // Editor Inspector - right column, bottom half
        let (inspector_x, inspector_y) = self.editor_state.inspector_pos.unwrap_or((1560.0, 550.0));
        let (inspector_w, inspector_h) = self.editor_state.inspector_size.unwrap_or((350.0, 530.0));
        egui::Window::new("Inspector")
            .default_pos(egui::pos2(inspector_x, inspector_y))
            .default_size(egui::vec2(inspector_w, inspector_h))
            .show(context, |ui| {
                EditorInspector::ui(ui, &mut self.editor_state, world);
            });
        
        // Handle request to open scripts window for a specific object
        if let Some(object_name) = self.editor_state.open_scripts_for_object.take() {
            self.selected_object_name = object_name;
            w2 = true;
        }
        
        // Add Panel - 90% высоты экрана
        // Получаем высоту экрана из контекста
        let screen_height = context.available_rect().height();
        let panel_height = screen_height * 0.9;
        egui::Window::new("Add Objects")
            .default_pos(egui::pos2(320.0, 100.0))
            .default_size(egui::vec2(200.0, panel_height))
            .resizable(false)
            .show(context, |ui| {
                egui::ScrollArea::vertical()
                    .max_height(ui.available_height())
                    .show(ui, |ui| {
                AddPanel::ui(ui, &mut self.editor_state, world);
                    });
            });
        
        // Show toasts
        self.show_toasts(context);
        
        egui::Window::new("Swithes")
        .default_pos(egui::pos2(10.0,10.0))
        .show(context, |ui| {
            if ui.button("Config").clicked() {
                w1 = !w1;
            }
            if ui.button("Script").clicked() {
                w2 = !w2;
            }
            if ui.button("Sim").clicked() {
                w3 = !w3;
            }
            if ui.button("Templates").clicked() {
                w4 = !w4;
            }
            if ui.button("Editor").clicked() {
                w6 = !w6;
            }
        });
        
        self.editor_state.show_grid = w6;

        
        egui::Window::new("Configuration")
            .open(&mut w1)
            .default_pos(egui::pos2(340.0, 5.0))
            .default_size(egui::vec2(200.0, 100.0))
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
        self.w1 = w1;
        
        // Ограничиваем максимальный размер окна Script Editor размером приложения
        let screen_height = context.available_rect().height();
        let max_window_height = screen_height * 0.95; // 95% высоты экрана с небольшим отступом
        // Фиксируем размер окна, чтобы оно не могло стать слишком большим
        let fixed_height = max_window_height.min(600.0);
        
        egui::Window::new("Script Editor")
            .open(&mut w2)
            .default_pos(egui::pos2(560.0, 5.0))
            .fixed_size(egui::vec2(600.0, fixed_height))
            .show(context, |ui| {
                use specs::Join;
                use crate::ecs::components::Name;
                
                // Получаем список всех объектов (сначала собираем данные)
                let mut object_names: Vec<String> = Vec::new();
                {
                    let names = world.read_storage::<Name>();
                    let entities = world.entities();
                    for (_, name_comp) in (&entities, &names).join() {
                        object_names.push(name_comp.name.clone());
                    }
                }
                object_names.sort();
                
                // Верхняя панель с выбором объекта
                ui.horizontal(|ui| {
                    ui.label("Object:");
                    egui::ComboBox::from_id_source("object_selector")
                        .width(200.0)
                        .selected_text(&self.selected_object_name)
                        .show_ui(ui, |ui| {
                            for name in &object_names {
                                ui.selectable_value(&mut self.selected_object_name, name.clone(), name);
                            }
                        });
                });
                
                // Проверяем изменение выбранного объекта и загружаем скрипт только при изменении
                // Важно: проверка должна быть ПОСЛЕ рендера ComboBox, чтобы изменения обнаруживались в том же кадре
                if self.selected_object_name != self.last_loaded_object {
                    if let Some(script_text) = self.get_object_script(world, &self.selected_object_name) {
                        self.script = script_text;
                    } else {
                        self.script = "".to_owned();
                    }
                    self.last_loaded_object = self.selected_object_name.clone();
                    self.script_modified = false;
                    // Очищаем ошибки при переключении объекта
                    self.script_error.clear();
                }
                
                ui.separator();
                
                // Панель инструментов
                ui.horizontal(|ui| {
                    // Кнопка включения/выключения скрипта
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
                    
                    // Кнопки экспорта/импорта
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
                    
                    // Кнопка открытия окна лога
                    if ui.button("📋 Log").clicked() {
                        self.show_log_window = true;
                    }
                });
                
                // Отображение ошибок (всегда резервируем место, чтобы не терять фокус)
                ui.separator();
                // Всегда резервируем фиксированную высоту для области ошибки
                // Это предотвращает перестройку UI и потерю фокуса
                let error_area_height = ui.text_style_height(&egui::TextStyle::Body) + 8.0;
                let error_id = egui::Id::new(format!("script_error_{}", self.selected_object_name));
                let (_id, error_rect) = ui.allocate_space(egui::vec2(ui.available_width(), error_area_height));
                
                // Показываем ошибку только если она есть, используя зарезервированное место
                // Используем стабильный ID для предотвращения перестройки
                // Не вызываем allocate_ui_at_rect когда ошибки нет, чтобы не вызывать перестройку
                if !self.script_error.is_empty() {
                    ui.allocate_ui_at_rect(error_rect, |ui| {
                        ui.push_id(error_id, |ui| {
                            ui.horizontal(|ui| {
                                ui.colored_label(egui::Color32::from_rgb(255, 100, 100), "⚠ Error:");
                                ui.label(&self.script_error);
                            });
                        });
                    });
                }
                
                ui.separator();
                
                // Редактор кода с улучшенным интерфейсом
                // Используем стабильный ID для предотвращения перестройки
                let script_label_id = egui::Id::new(format!("script_label_{}", self.selected_object_name));
                ui.push_id(script_label_id, |ui| {
                    ui.label(format!("Script: {}", self.selected_object_name));
                });
                
                // Улучшенный редактор с лучшим размером
                // Используем стабильный ID для сохранения фокуса
                // Растягиваем редактор по всей доступной высоте
                let available_height = ui.available_height();
                egui::ScrollArea::vertical()
                    .auto_shrink([false, false])
                    .enable_scrolling(true)
                    .always_show_scroll(true)
                    .max_height(available_height)
                    .show(ui, |ui| {
                        // Используем стабильный ID на основе имени объекта для сохранения фокуса
                        let text_edit_id = egui::Id::new(format!("script_editor_{}", self.selected_object_name));
                        // Выделяем пространство для TextEdit, чтобы он растянулся по вертикали
                        let (_, text_edit_rect) = ui.allocate_space(egui::vec2(ui.available_width(), available_height));
                        ui.allocate_ui_at_rect(text_edit_rect, |ui| {
                        let text_edit = egui::TextEdit::multiline(&mut self.script)
                                .id(text_edit_id)
                            .font(egui::TextStyle::Monospace)
                                .desired_width(f32::INFINITY);
                        
                        let response = ui.add(text_edit);
                        
                        // Отслеживаем изменения
                        if response.changed() {
                            self.script_modified = true;
                        }
                        
                        // Обрабатываем вставку из буфера обмена, если TextEdit в фокусе
                        if response.has_focus() {
                            let modifiers = ui.input().modifiers;
                            let paste_pressed = (modifiers.command || modifiers.ctrl) && ui.input().key_pressed(egui::Key::V);
                            
                            if paste_pressed {
                                #[cfg(target_arch = "wasm32")]
                                {
                                    // В браузере используем асинхронный API
                                    let event_loop_proxy = event_loop_proxy.clone();
                                    self.executor.execute(async move {
                                        if let Ok(text) = crate::copy_text_from_clipboard_async().await {
                                            let _ = event_loop_proxy.send_event(UserEventInfo::TextImport(text.into_bytes()));
                                        }
                                    });
                                }
                                
                                #[cfg(not(target_arch = "wasm32"))]
                                {
                                    // На десктопе используем синхронный API
                                    if let Ok(text) = crate::copy_text_from_clipboard() {
                                        // Вставляем текст в текущую позицию курсора или в конец
                                        // Для простоты вставляем в конец, так как получить позицию курсора сложно
                                        self.script.push_str(&text);
                                        self.script_modified = true;
                                    }
                                }
                            }
                        }
                        });
                    });
                
                // Информация о скрипте (используем стабильный ID для предотвращения перестройки)
                ui.separator();
                let stats_id = egui::Id::new(format!("script_stats_{}", self.selected_object_name));
                ui.push_id(stats_id, |ui| {
                    ui.horizontal(|ui| {
                        let line_count = self.script.lines().count();
                        let char_count = self.script.chars().count();
                        ui.label(format!("Lines: {} | Characters: {}", line_count, char_count));
                    });
                });
                
        
                *any_win_hovered |= context.is_pointer_over_area()
            });
        self.w2 = w2;
        
        // Script Log Window
        egui::Window::new("Script Log")
            .open(&mut self.show_log_window)
            .default_pos(egui::pos2(1160.0, 5.0))
            .default_size(egui::vec2(400.0, 500.0))
            .show(context, |ui| {
                ui.horizontal(|ui| {
                    if ui.button("Clear").clicked() {
                        self.script_log.borrow_mut().clear();
                    }
                    ui.label(format!("Messages: {}", self.script_log.borrow().len()));
                });
                
                ui.separator();
                
                // Отображаем логи (последние 30 записей из кольцевого буфера)
                egui::ScrollArea::vertical()
                    .auto_shrink([false, false])
                    .stick_to_bottom(true)
                    .show(ui, |ui| {
                        let logs = self.script_log.borrow();
                        // VecDeque уже содержит только последние 30 записей благодаря кольцевому буферу
                        for (index, log_entry) in logs.iter().enumerate() {
                            ui.push_id(index, |ui| {
                                ui.label(log_entry);
                            });
                        }
                    });
                
                *any_win_hovered |= context.is_pointer_over_area()
            });
        
        egui::Window::new("Simulation")
            .open(&mut w3)
            .default_pos(egui::pos2(5.0, 5.0))
            .fixed_size(egui::vec2(200., 100.))
            .show(context, |ui| {
                // Simulation Configuration
                ui.heading("Pause or simulation speed");
                ui.add(
                    egui::Slider::new(&mut self.simulation_steps_per_second, 0..=480)
                        .text("Simulation steps per second"),
                );

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
                ui.label(format!(
                    "Frame Processing Time: {:.1} ms.",
                    upd_result.update_time
                ));
                if upd_result.dropping {
                    ui.colored_label(Color32::from_rgb(255, 0, 0), "frame drop");
                } else {
                    ui.label("running ok");
                }

                // Particle Spawning
                ui.separator();
                ui.heading("Particle Spawning");
                ui.label("Hold left mouse button to spawn particles");
                ComboBox::from_id_source("dropdown_list")
                    .selected_text(&self.selected_option)
                    .show_ui(ui, |ui| {
                        self.options.iter().for_each(|option| {
                            ui.selectable_value(
                                &mut self.selected_option,
                                option.to_string(),
                                option.to_string(),
                            );
                        });
                    });
                ui.label(format!("Selected: {}", self.selected_option));

                // Structure Spawning
                ui.separator();
                ui.heading("Structure Spawning");
                ui.add(
                    egui::Slider::new(&mut self.number_of_structures_to_add, 0..=10000)
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
        self.w3 = w3;
        // Separate window for GitHub templates / projects
        egui::Window::new("Templates")
            .open(&mut w4)
            .default_pos(egui::pos2(780.0, 5.0))
            .default_size(egui::vec2(420.0, 360.0))
            .show(context, |ui| {
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
                            right.label(
                                egui::RichText::new(&project.display_name)
                                    .strong(),
                            );

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
                                        egui::RichText::new("Background: none")
                                            .small()
                                            .italics(),
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
        if w4 && !self.projects_fetched && !self.project_loading {
            self.start_fetch_github_projects(event_loop_proxy);
        }
        
        self.w4 = w4;
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
            
            egui::Window::new("")
                .title_bar(false)
                .resizable(false)
                .fixed_pos(egui::pos2(10.0, 500.0 + (i as f32 * 40.0)))
                .show(ctx, |ui| {
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
            selected_option,
            options,
            cursor_position: None,
            pressed: false,
            hovered: false,
            executor,
            script: r"let a = 0; for i in 0..10 { a += i; };".to_owned(),
            selected_object_name: "World Script".to_owned(),
            last_loaded_object: String::new(),
            script_modified: false,
            script_error: "".to_owned(),
            need_to_recompile: true,

            w1: false,
            w2: false,
            w3: false,
            w4: false,

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
        }
    }
}

fn create_event_with_data(bytes: Vec<u8>) -> UserEventInfo {
    UserEventInfo::ImageImport(bytes)
}

fn create_event_with_text(bytes: Vec<u8>) -> UserEventInfo {
    UserEventInfo::TextImport(bytes)
}
