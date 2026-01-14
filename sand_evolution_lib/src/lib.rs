pub mod cells;
pub mod cs;
pub mod ecs;
pub mod editor;
pub mod evolution_app;
pub mod export_file;
pub mod fps_meter;
pub mod gbuffer;
pub mod shared_state;
pub mod state;
pub mod update;

pub mod projects;
mod random;
pub mod resources;
pub mod rhai_lib;

use crate::evolution_app::UserEventInfo;
use ::egui::FontDefinitions;
use chrono::Timelike;
use egui_wgpu_backend::{RenderPass, ScreenDescriptor};
use egui_winit_platform::{Platform, PlatformDescriptor};
use evolution_app::EvolutionApp;
use fps_meter::FpsMeter;
use state::State;
use std::cell::RefCell;
use std::collections::HashMap;
use std::fmt::{Display, Formatter};
use std::rc::Rc;
use winit::event::Event::*;
use winit::event_loop::{ControlFlow, EventLoop, EventLoopBuilder};
const INITIAL_WIDTH: u32 = 1920;
const INITIAL_HEIGHT: u32 = 1080;

#[derive(Debug)]
struct CellTypeNotFound {
    name: String,
}

impl Display for CellTypeNotFound {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "Cell type not found: {0}", self.name)
    }
}

impl std::error::Error for CellTypeNotFound {}

#[cfg(target_arch = "wasm32")]
use wasm_bindgen::prelude::*;
#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
struct Vertex {
    position: [f32; 3],
    uv: [f32; 2],
}
impl Vertex {
    fn desc<'a>() -> wgpu::VertexBufferLayout<'a> {
        wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<Vertex>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &[
                wgpu::VertexAttribute {
                    offset: 0,
                    shader_location: 0,
                    format: wgpu::VertexFormat::Float32x3,
                },
                wgpu::VertexAttribute {
                    offset: std::mem::size_of::<[f32; 3]>() as wgpu::BufferAddress,
                    shader_location: 1,
                    format: wgpu::VertexFormat::Float32x2,
                },
            ],
        }
    }
}
const VERTICES: &[Vertex] = &[
    Vertex {
        position: [-1.0, -1.0, 0.0],
        uv: [0.0, 0.0],
    },
    Vertex {
        position: [1.0, -1.0, 0.0],
        uv: [1.0, 0.0],
    },
    Vertex {
        position: [-1.0, 1.0, 0.0],
        uv: [0.0, 1.0],
    },
    Vertex {
        position: [1.0, 1.0, 0.0],
        uv: [1.0, 1.0],
    },
];

#[cfg(target_arch = "wasm32")]
use crate::export_file::code_to_file;
#[cfg(target_arch = "wasm32")]
use web_sys::{Navigator, Window};
#[cfg(target_arch = "wasm32")]
pub fn copy_text_to_clipboard(text: &str) -> Result<(), Box<dyn std::error::Error>> {
    code_to_file(&text.to_owned())?;
    Ok(())
}
#[cfg(target_arch = "wasm32")]
pub fn copy_text_from_clipboard() -> Result<String, Box<dyn std::error::Error>> {
    // In the browser, clipboard is only available through async API
    // This function will be called from an async context
    Ok("".to_owned())
}

#[cfg(target_arch = "wasm32")]
pub async fn copy_text_from_clipboard_async() -> Result<String, Box<dyn std::error::Error>> {
    use wasm_bindgen::JsCast;
    use wasm_bindgen_futures::JsFuture;
    use web_sys::Clipboard;

    let window = web_sys::window().ok_or("No window")?;
    let navigator = window.navigator();
    let clipboard_js = js_sys::Reflect::get(&navigator, &"clipboard".into())
        .map_err(|e| format!("Failed to get clipboard: {:?}", e))?;
    let clipboard: Clipboard = clipboard_js
        .dyn_into()
        .map_err(|e| format!("Failed to cast to Clipboard: {:?}", e))?;

    let promise = clipboard.read_text();
    let js_value = JsFuture::from(promise)
        .await
        .map_err(|e| format!("Clipboard read error: {:?}", e))?;
    let text = js_value.as_string().ok_or("Failed to get text")?;

    Ok(text)
}

use crate::shared_state::SharedState;
#[cfg(not(target_arch = "wasm32"))]
use clipboard::ClipboardProvider;

#[cfg(not(target_arch = "wasm32"))]
pub fn copy_text_to_clipboard(text: &str) -> Result<(), Box<dyn std::error::Error>> {
    let mut ctx: clipboard::ClipboardContext = clipboard::ClipboardProvider::new()?;
    ctx.set_contents(text.to_owned())?;
    Ok(())
}

#[cfg(not(target_arch = "wasm32"))]
pub fn copy_text_from_clipboard() -> Result<String, Box<dyn std::error::Error>> {
    let mut ctx: clipboard::ClipboardContext = clipboard::ClipboardProvider::new()?;
    ctx.get_contents()
}

use crate::ecs::components::{
    Children, Name, Parent, Position, Rotation, Scale, Script, ScriptType, Velocity,
};
use crate::ecs::systems::{EntityScriptSystem, GravitySystem, MoveSystem};
use crate::resources::rhai_resource::{RhaiResource, RhaiResourceStorage};
use crate::state::UpdateResult;
use specs::{Builder, Entity, RunNow, WorldExt};
use wgpu::Queue;
use winit::dpi::PhysicalSize;

pub struct GameContext {
    pub world: specs::World,
    pub dispatcher: specs::Dispatcher<'static, 'static>,
    pub state: State,
}

pub(crate) fn init_hardcoded_entities(world: &mut specs::World) {
    // Create an object for the world script
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

    // Create a Dummy entity for testing
    world
        .create_entity()
        .with(Name {
            name: "Dummy".to_owned(),
        })
        .with(Position { x: 100.0, y: 200.0 })
        .with(Velocity { x: 1.0, y: 0.0 })
        .with(Script {
            script: r#"// Dummy object script
// This is a test entity with Position, Velocity and Script components
"#
            .to_owned(),
            ast: None,
            raw: true,
            script_type: ScriptType::Entity,
            run_once: false,
            has_run: false,
        })
        .build();

    // Create Cooler entity - cools the top row of cells every tick
    world
        .create_entity()
        .with(Name {
            name: "Cooler".to_owned(),
        })
        .with(Script {
            script: r#"// Cooler object script - cools the top row of cells every tick
let top_row_y = 511;
let cool_temp = -10.0;

// Cool all cells in the top row every tick
for x in 0..GRID_WIDTH {
    set_temperature(x, top_row_y, cool_temp);
}
"#
            .to_owned(),
            ast: None,
            raw: true,
            script_type: ScriptType::Entity,
            run_once: false,
            has_run: false,
        })
        .build();
}

impl GameContext {
    pub fn new(state: State) -> Self {
        let mut world = specs::World::new();

        // Register all components
        world.register::<Name>();
        world.register::<Script>();
        world.register::<Position>();
        world.register::<Velocity>();
        world.register::<Rotation>();
        world.register::<Scale>();
        world.register::<Parent>();
        world.register::<Children>();

        let mut dispatcher = specs::DispatcherBuilder::new()
            .with(EntityScriptSystem, "entity_script__system", &[])
            .with(GravitySystem, "gravity_system", &[])
            .with(MoveSystem, "move_system", &[])
            .build();
        dispatcher.setup(&mut world);

        init_hardcoded_entities(&mut world);

        GameContext {
            world,
            dispatcher,
            state,
        }
    }

    pub fn dispatch(&mut self) {
        self.dispatcher.dispatch(&self.world);
    }

    /// Find entity by name
    pub fn find_entity_by_name(&self, name: &str) -> Option<Entity> {
        use specs::Join;
        let names = self.world.read_storage::<Name>();
        let entities = self.world.entities();

        for (entity, name_comp) in (&entities, &names).join() {
            if name_comp.name == name {
                return Some(entity);
            }
        }
        None
    }

    pub fn update(
        &mut self,
        queue: &Queue,
        steps_per_this_frame: i32,
        evolution_app: &mut EvolutionApp,
        shared_state: &Rc<RefCell<SharedState>>,
        size: PhysicalSize<u32>,
        scale_factor: f64,
    ) -> UpdateResult {
        self.state.update(
            &queue,
            steps_per_this_frame as i32,
            evolution_app,
            &mut self.world,
            shared_state,
            size,
            scale_factor,
        )
    }
}

const INDICES: &[u16] = &[0, 1, 3, 0, 3, 2];
#[cfg_attr(target_arch = "wasm32", wasm_bindgen)]
pub async fn run(w: f32, h: f32, data: &[u8], script: String) {
    let mut fps_meter = FpsMeter::new();

    cfg_if::cfg_if! {
        if #[cfg(target_arch = "wasm32")] {
            std::panic::set_hook(Box::new(console_error_panic_hook::hook));
            console_log::init_with_level(log::Level::Warn).expect("Could't initialize logger");
        } else {
            env_logger::init();
        }
    }

    let event_loop: EventLoop<UserEventInfo> =
        EventLoopBuilder::<UserEventInfo>::with_user_event().build();

    let window = winit::window::WindowBuilder::new()
        .with_decorations(true)
        .with_resizable(true)
        .with_transparent(false)
        .with_title("sand evolution v0.1")
        .with_inner_size(winit::dpi::LogicalSize {
            width: w,
            height: h,
        })
        .build(&event_loop)
        .unwrap();

    #[cfg(target_arch = "wasm32")]
    {
        use winit::dpi::PhysicalSize;
        window.set_inner_size(winit::dpi::LogicalSize::new(w, h));

        use winit::platform::web::WindowExtWebSys;
        web_sys::window()
            .and_then(|win| win.document())
            .and_then(|doc| {
                let dst = doc.get_element_by_id("wasm-example")?;
                let canvas = web_sys::Element::from(window.canvas());
                dst.append_child(&canvas).ok()?;
                Some(())
            })
            .expect("Couldn't append canvas to document body.");
    }

    let instance = wgpu::Instance::new(wgpu::Backends::all());
    let surface = unsafe { instance.create_surface(&window) };

    let adapter = instance
        .request_adapter(&wgpu::RequestAdapterOptions {
            power_preference: wgpu::PowerPreference::default(),
            compatible_surface: Some(&surface),
            force_fallback_adapter: false,
        })
        .await
        .unwrap();

    let (device, queue) = adapter
        .request_device(
            &wgpu::DeviceDescriptor {
                label: None,
                features: wgpu::Features::empty(),
                // WebGL doesn't support all of wgpu's features, so if
                // we're building for the web we'll have to disable some.
                limits: if cfg!(target_arch = "wasm32") {
                    wgpu::Limits::downlevel_webgl2_defaults()
                } else {
                    wgpu::Limits::default()
                },
            },
            None, // Trace path
        )
        .await
        .unwrap();

    let size = window.inner_size();
    let surface_format = surface.get_supported_formats(&adapter)[0];
    let mut surface_config = wgpu::SurfaceConfiguration {
        usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
        format: surface_format,
        width: size.width,
        height: size.height,
        present_mode: wgpu::PresentMode::Fifo,
    };
    surface.configure(&device, &surface_config);

    let mut platform = Platform::new(PlatformDescriptor {
        physical_width: size.width as u32,
        physical_height: size.height as u32,
        scale_factor: window.scale_factor(),
        font_definitions: FontDefinitions::default(),
        style: Default::default(),
    });

    let mut egui_rpass = RenderPass::new(&device, surface_format, 1);

    // Display the demo application that ships with egui.
    //let mut demo_app = egui_demo_lib::DemoWindows::default();

    let shared_state_rc = Rc::new(RefCell::new(SharedState::new()));
    let mut game_context = GameContext::new(State::new(
        &device,
        &queue,
        &surface_config,
        &surface,
        surface_format,
    ));
    game_context.state.update_with_data(data);

    // Create shared log storage for scripts before creating EvolutionApp
    // Use VecDeque as a circular buffer with a limit of 30 entries
    use std::collections::VecDeque;
    let script_log_rc = Rc::new(RefCell::new(VecDeque::<String>::with_capacity(30)));

    let mut evolution_app = EvolutionApp::new_with_log(script_log_rc.clone());

    evolution_app.set_script(script.as_str());

    // Update the world script object with the initial script
    evolution_app.set_object_script(&mut game_context.world, "World Script", script.as_str());

    let mut id_dict: HashMap<String, u8> = HashMap::new();

    for a in game_context.state.pal_container.pal.iter() {
        if a.id() != 0 {
            evolution_app.options.push(a.name().to_owned());
            id_dict.insert(a.name().to_owned(), a.id());
        }
    }

    {
        let mut rhai = rhai::Engine::new();
        let mut rhai_scope = rhai::Scope::new();

        // Register functions
        rhai_lib::register_rhai(
            &mut rhai,
            &mut rhai_scope,
            shared_state_rc.clone(),
            id_dict,
            None,
            script_log_rc.clone(),
            None,
        );

        game_context.world.insert(RhaiResource {
            storage: Some(RhaiResourceStorage {
                engine: rhai,
                scope: rhai_scope,
                state_ptr: std::cell::Cell::new(std::ptr::null_mut()),
                script_log: script_log_rc.clone(),
            }),
        });
    }

    let event_loop_proxy = event_loop.create_proxy();

    let start_time = instant::now();
    let mut last_frame_time = 0.0;
    let event_loop_shared_state = shared_state_rc.clone();
    let upd_result = UpdateResult::default();
    event_loop.run(move |event, _, control_flow| {
        // Pass the winit events to the platform integration.
        platform.handle_event(&event);

        match event {
            RedrawRequested(..) => {
                let frame_start_time = (instant::now() - start_time) / 1000.0;
                // Protect against the "spiral of death": cap the dt used for stepping.
                // If a frame takes long, simulating the full backlog can make the next frame even heavier.
                let mut delta_t = frame_start_time - last_frame_time;
                if delta_t.is_nan() || delta_t.is_infinite() {
                    delta_t = 0.0;
                }
                // Cap the wall-clock delta used for stepping to prevent spiral of death.
                delta_t = delta_t.min(0.25);
                last_frame_time = frame_start_time;
                platform.update_time(frame_start_time);

                let output_frame = match surface.get_current_texture() {
                    Ok(frame) => frame,
                    Err(wgpu::SurfaceError::Outdated) => {
                        // This error occurs when the app is minimized on Windows.
                        // Silently return here to prevent spamming the console with:
                        // "The underlying surface has changed, and therefore the swap chain must be updated"
                        return;
                    }
                    Err(e) => {
                        eprintln!("Dropped frame with error: {}", e);
                        return;
                    }
                };

                let output_view = output_frame
                    .texture
                    .create_view(&wgpu::TextureViewDescriptor::default());

                // Calculate how many simulation steps to run this frame.
                // When paused, automatic stepping is disabled, but manual step buttons
                // in the UI can still queue discrete steps.
                let mut sim_steps: i32 = if !evolution_app.simulation_paused
                    && evolution_app.simulation_steps_per_second > 0
                {
                    // Steps are based on the current frame's delta time
                    let desired_steps = (delta_t * evolution_app.simulation_steps_per_second as f64)
                        .floor() as i32;
                    desired_steps.max(0)
                } else {
                    // While paused (or with zero speed) don't run automatic simulation steps.
                    0
                };

                // Apply any queued manual steps (from "Step ×1/×10" buttons).
                if evolution_app.pending_simulation_steps > 0 {
                    sim_steps += evolution_app.pending_simulation_steps;
                    evolution_app.pending_simulation_steps = 0;
                }

                if evolution_app.need_to_recompile {
                    // First, find the entity
                    let script_entity =
                        game_context.find_entity_by_name(&evolution_app.selected_object_name);

                    if let Some(script_entity) = script_entity {
                        // Get rhai_resource separately
                        let rhai_resource_opt = game_context.world.get_mut::<RhaiResource>();

                        if let Some(rhai_resource) = rhai_resource_opt {
                            if let Some(storage) = &mut rhai_resource.storage {
                                // Compile the script of the selected object
                                let script_text = evolution_app.get_script().to_owned();
                                let result = storage
                                    .engine
                                    .compile_with_scope(&mut storage.scope, script_text.as_str());

                                match result {
                                    Ok(value) => {
                                        let mut scripts = game_context
                                            .world
                                            .write_storage::<crate::ecs::components::Script>(
                                        );
                                        if let Some(script) = scripts.get_mut(script_entity) {
                                            script.ast = Some(value);
                                            script.script = script_text;
                                            script.raw = false;
                                            // If this is a one-shot script, allow it to run again after edit/compile.
                                            script.has_run = false;
                                        }
                                        evolution_app.script_error = "".to_owned();
                                    }
                                    Err(err) => {
                                        let mut scripts = game_context
                                            .world
                                            .write_storage::<crate::ecs::components::Script>(
                                        );
                                        if let Some(script) = scripts.get_mut(script_entity) {
                                            script.ast = None;
                                            script.raw = true;
                                            script.has_run = false;
                                        }
                                        evolution_app.script_error = err.to_string()
                                    }
                                }
                            } else {
                                println!("Warning: RhaiResource.storage is None");
                            }
                        } else {
                            println!("Warning: RhaiResource not found in the world");
                        }
                    }
                }

                // UPDATE (also runs on pause with sim_steps=0, to keep uniforms/UI responsive)
                let upd_result = game_context.update(
                    &queue,
                    sim_steps,
                    &mut evolution_app,
                    &event_loop_shared_state,
                    window.inner_size(),
                    window.scale_factor(),
                );

                // dispatch() is now called inside update_tick on each simulation tick.
                // Keep one call for cases when simulation is not running at all.
                if sim_steps == 0
                    && (evolution_app.simulation_paused
                        || evolution_app.simulation_steps_per_second <= 0)
                {
                    game_context.dispatch();
                }

                _ = game_context.state.render(&device, &queue, &output_view);

                // Begin to draw the UI frame.
                //
                //    //////
                //    /    /
                //    //////
                //

                platform.begin_frame();

                let mut any_win_hovered = false;

                evolution_app.ui(
                    &platform.context(),
                    &mut game_context.state,
                    &mut fps_meter,
                    &upd_result,
                    &event_loop_proxy,
                    &mut any_win_hovered,
                    &mut game_context.world,
                );

                evolution_app.hovered = any_win_hovered;

                // End the UI frame. We could now handle the output and draw the UI with the backend.
                let full_output = platform.end_frame(Some(&window));
                let paint_jobs = platform.context().tessellate(full_output.shapes);

                let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
                    label: Some("Render Encoder"),
                });

                // Upload all resources for the GPU.
                let screen_descriptor = ScreenDescriptor {
                    physical_width: surface_config.width,
                    physical_height: surface_config.height,
                    scale_factor: window.scale_factor() as f32,
                };
                let tdelta: egui::TexturesDelta = full_output.textures_delta;
                egui_rpass
                    .add_textures(&device, &queue, &tdelta)
                    .expect("add texture ok");
                egui_rpass.update_buffers(&device, &queue, &paint_jobs, &screen_descriptor);

                // Record all render passes.
                egui_rpass
                    .execute(
                        &mut encoder,
                        &output_view,
                        &paint_jobs,
                        &screen_descriptor,
                        None,
                    )
                    .unwrap();

                // Submit the commands.
                queue.submit(std::iter::once(encoder.finish()));

                // Redraw egui
                output_frame.present();

                egui_rpass
                    .remove_textures(tdelta)
                    .expect("remove texture ok");

                // Let wgpu process internal queues and retire resources/command buffers.
                // Some backends behave poorly if you never poll the device.
                device.poll(wgpu::Maintain::Poll);

                // Support reactive on windows only, but not on linux.
                // if _output.needs_repaint {
                //     *control_flow = ControlFlow::Poll;
                // } else {
                //     *control_flow = ControlFlow::Wait;
                // }
            }
            MainEventsCleared {} => {
                window.request_redraw();
            }
            WindowEvent { event, .. } => match event {
                winit::event::WindowEvent::Resized(size) => {
                    // Resize with 0 width and height is used by winit to signal a minimize event on Windows.
                    // See: https://github.com/rust-windowing/winit/issues/208
                    // This solves an issue where the app would panic when minimizing on Windows.
                    if size.width > 0 && size.height > 0 {
                        surface_config.width = size.width;
                        surface_config.height = size.height;
                        surface.configure(&device, &surface_config);
                    }
                }
                winit::event::WindowEvent::CursorMoved { position, .. } => {
                    evolution_app.cursor_position = Some(position);
                }
                winit::event::WindowEvent::CloseRequested => {
                    *control_flow = ControlFlow::Exit;
                }
                winit::event::WindowEvent::KeyboardInput {
                    device_id: _,
                    input: _,
                    is_synthetic: _,
                } => {
                    // Don't handle keyboard events here - egui TextEdit handles
                    // clipboard paste through standard key combinations (Cmd+V/Ctrl+V)
                    // Handling through winit causes conflicts and panics on macOS
                }
                winit::event::WindowEvent::MouseInput { state, button, .. } => {
                    if button == winit::event::MouseButton::Left {
                        if state == winit::event::ElementState::Pressed {
                            evolution_app.pressed = true;
                        } else {
                            evolution_app.pressed = false;
                        }
                    }
                }
                winit::event::WindowEvent::DroppedFile(file_path) => {
                    // Load the image and create a texture from it
                    let img = image::open(file_path).unwrap().to_luma8();
                    let dimensions = img.dimensions();

                    if dimensions.0 == cs::SECTOR_SIZE.x as u32
                        && dimensions.1 == cs::SECTOR_SIZE.y as u32
                    {
                        game_context.state.diffuse_rgba = img;
                        game_context.state.reset_temperatures();
                    }
                }
                _ => {}
            },
            UserEvent(event) => match event {
                UserEventInfo::ImageImport(image) => {
                    game_context.state.update_with_data(&image);
                    evolution_app.project_loading = false;
                }
                UserEventInfo::TextImport(text) => {
                    match String::from_utf8(text) {
                        Ok(text) => {
                            evolution_app.set_script(text.as_str());
                            // Also set the script to World Script entity so it actually runs
                            evolution_app.set_object_script(&mut game_context.world, "World Script", text.as_str());
                        }
                        Err(_) => {
                            panic!("Invalid UTF-8 data");
                        }
                    }
                    evolution_app.project_loading = false;
                }
                UserEventInfo::ResetWorldEntitiesToHardcoded => {
                    evolution_app.reset_world_entities_to_hardcoded(&mut game_context.world);
                }
                UserEventInfo::SceneImport(bytes) => {
                    match String::from_utf8(bytes) {
                        Ok(text) => {
                            match evolution_app
                                .import_scene_from_toml(&mut game_context.world, &text)
                            {
                                Ok(()) => {
                                    evolution_app.editor_state.add_toast(
                                        "Scene imported".to_owned(),
                                        crate::editor::state::ToastLevel::Info,
                                    );
                                }
                                Err(err) => {
                                    evolution_app.editor_state.add_toast(
                                        format!("Scene import error: {}", err),
                                        crate::editor::state::ToastLevel::Error,
                                    );
                                }
                            }
                        }
                        Err(_) => {
                            evolution_app.editor_state.add_toast(
                                "Scene import error: invalid UTF-8".to_owned(),
                                crate::editor::state::ToastLevel::Error,
                            );
                        }
                    }
                    evolution_app.project_loading = false;
                }
                UserEventInfo::ProjectsLoaded(projects) => {
                    evolution_app.projects = projects;
                    evolution_app.project_loading = false;
                    evolution_app.projects_fetched = true;
                }
                UserEventInfo::ProjectLoadError(err) => {
                    evolution_app.project_error = err;
                    evolution_app.project_loading = false;
                    evolution_app.projects_fetched = true; // Mark as fetched even on error to prevent retry loop
                }
            },
            _ => (),
        }
    });
}

/// Time of day as seconds since midnight. Used for clock in demo app.
pub fn seconds_since_midnight() -> f64 {
    let time = chrono::Local::now().time();
    time.num_seconds_from_midnight() as f64 + 1e-9 * (time.nanosecond() as f64)
}

#[cfg(test)]
#[cfg(not(target_arch = "wasm32"))] // Tests require filesystem access, not available in wasm32
mod tests {
    use super::*;
    use crate::cells::CellRegistry;
    use crate::ecs::components::{Name, Script, ScriptType};
    use crate::rhai_lib;
    use crate::shared_state::SharedState;
    use specs::{Builder, World, WorldExt};
    use std::collections::{HashMap, VecDeque};
    use std::fs;
    use std::path::Path;
    use std::rc::Rc;
    use std::cell::RefCell;

    fn get_maps_dir() -> std::path::PathBuf {
        let maps_dir = Path::new("/Users/olga/Rust/sand_evolution_maps");
        
        if maps_dir.exists() {
            return maps_dir.to_path_buf();
        }
        
        // Try relative paths as fallback
        let manifest_dir = env!("CARGO_MANIFEST_DIR");
        let parent = Path::new(manifest_dir).parent();
        if let Some(p) = parent {
            let sibling = p.join("../sand_evolution_maps");
            if sibling.exists() {
                return sibling;
            }
        }
        
        panic!("sand_evolution_maps directory not found. Tried: {:?}\nManifest dir: {}", maps_dir, manifest_dir);
    }

    /// Test that compiles all Rhai scripts from sand_evolution_maps directory.
    /// 
    /// This test verifies that all scripts have valid syntax and can be compiled.
    /// 
    /// **How to verify this test works:**
    /// 1. Add a syntax error to any .rhai file (e.g., `set_cell(100, 100, "sand"` - missing closing paren)
    /// 2. Run: `cargo test --package sand_evolution_lib --lib test_compile_all_scripts_from_maps`
    /// 3. The test should fail with a compilation error
    #[test]
    fn test_compile_all_scripts_from_maps() {
        let maps_dir = get_maps_dir();
        compile_scripts_in_dir(&maps_dir);
    }
    
    /// Test that compiles AND executes all Rhai scripts from sand_evolution_maps directory.
    /// 
    /// This test:
    /// - Compiles all scripts (catches syntax errors)
    /// - Executes each script for 10 ticks (catches runtime errors)
    /// - Verifies scripts run without errors
    /// 
    /// **How to verify this test works:**
    /// 1. Add a runtime error to any .rhai file (e.g., `undefined_function(123);`)
    /// 2. Run: `cargo test --package sand_evolution_lib --lib test_compile_and_execute_all_scripts_from_maps`
    /// 3. The test should fail with a runtime error message showing which tick failed
    /// 
    /// **Types of errors caught:**
    /// - Compile errors: syntax mistakes, undefined variables at compile time
    /// - Runtime errors: calling undefined functions, type mismatches, index out of bounds
    #[test]
    fn test_compile_and_execute_all_scripts_from_maps() {
        let maps_dir = get_maps_dir();
        compile_and_test_scripts(&maps_dir, true);
    }
    
    /// Generate snapshots (PNG images) for all scripts after 10 ticks.
    /// 
    /// This test runs each script for 10 ticks and saves the resulting map as a PNG file.
    /// Files are saved in a "snapshots" subdirectory with "_snapshot.png" suffix.
    /// 
    /// **Usage:**
    /// ```bash
    /// cargo test --package sand_evolution_lib --lib generate_snapshots -- --ignored --nocapture
    /// ```
    /// 
    /// The `--ignored` flag is required because this test is marked with `#[ignore]` 
    /// to prevent it from running during normal test execution (it generates files).
    /// 
    /// **Output:**
    /// Snapshots are saved to `sand_evolution_maps/snapshots/` directory.
    #[test]
    #[ignore] // Ignored by default, run explicitly with: cargo test -- --ignored
    fn generate_snapshots() {
        let maps_dir = get_maps_dir();
        generate_script_snapshots(&maps_dir);
    }
    
    fn compile_scripts_in_dir(maps_dir: &Path) {
        compile_and_test_scripts(maps_dir, false);
    }
    
    fn compile_and_test_scripts(maps_dir: &Path, run_execution_test: bool) {
        // Create cell registry to get id_dict
        let cell_registry = CellRegistry::new();
        let mut id_dict: HashMap<String, u8> = HashMap::new();
        for a in cell_registry.pal.iter() {
            if a.id() != 0 {
                id_dict.insert(a.name().to_owned(), a.id());
            }
        }

        // Find all .rhai files
        let mut script_files = Vec::new();
        if let Ok(entries) = fs::read_dir(maps_dir) {
            for entry in entries {
                if let Ok(entry) = entry {
                    let path = entry.path();
                    if path.extension().and_then(|s| s.to_str()) == Some("rhai") {
                        script_files.push(path);
                    }
                }
            }
        }

        if script_files.is_empty() {
            panic!("No .rhai files found in {:?}", maps_dir);
        }

        // Sort for consistent output
        script_files.sort();

        // Try to compile and optionally test execution of each script
        let mut failed_scripts = Vec::new();
        for script_path in &script_files {
            let script_name = script_path.file_name()
                .and_then(|n| n.to_str())
                .unwrap_or("unknown");
            
            match fs::read_to_string(script_path) {
                Ok(script_content) => {
                    // Test compilation
                    let mut rhai = rhai::Engine::new();
                    rhai.set_max_expr_depths(100, 100);
                    let mut rhai_scope = rhai::Scope::new();
                    let shared_state_rc = Rc::new(RefCell::new(SharedState::new()));
                    let script_log_rc = Rc::new(RefCell::new(VecDeque::<String>::with_capacity(30)));
                    
                    rhai_lib::register_rhai(
                        &mut rhai,
                        &mut rhai_scope,
                        shared_state_rc.clone(),
                        id_dict.clone(),
                        None,
                        script_log_rc.clone(),
                        None,
                    );
                    
                    match rhai.compile_with_scope(&mut rhai_scope, script_content.as_str()) {
                        Ok(ast) => {
                            println!("✓ Successfully compiled: {}", script_name);
                            
                            // If execution testing is enabled, run the script for several ticks
                            if run_execution_test {
                                // Reset deterministic RNG before each script test
                                use crate::random::{set_deterministic_rng};
                                use rand::SeedableRng;
                                let seed = [42u8; 32];
                                let deterministic_rng = Rc::new(RefCell::new(rand::rngs::StdRng::from_seed(seed)));
                                set_deterministic_rng(deterministic_rng);
                                
                                match test_script_execution(
                                    &rhai,
                                    &mut rhai_scope,
                                    &ast,
                                    &shared_state_rc,
                                    &script_log_rc,
                                    script_name,
                                    maps_dir,
                                ) {
                                    Ok(_) => {
                                        println!("  ✓ Execution test passed");
                                    }
                                    Err(err) => {
                                        eprintln!("  ✗ Execution test failed: {}", err);
                                        failed_scripts.push((script_name.to_string(), format!("Execution error: {}", err)));
                                    }
                                }
                            }
                        }
                        Err(err) => {
                            eprintln!("✗ Failed to compile {}: {}", script_name, err);
                            failed_scripts.push((script_name.to_string(), err.to_string()));
                        }
                    }
                }
                Err(err) => {
                    eprintln!("✗ Failed to read {}: {}", script_name, err);
                    failed_scripts.push((script_name.to_string(), format!("Read error: {}", err)));
                }
            }
        }

        // Report results
        if !failed_scripts.is_empty() {
            // Clear deterministic RNG before panic
            #[cfg(not(target_arch = "wasm32"))]
            if run_execution_test {
                crate::random::clear_deterministic_rng();
            }
            eprintln!("\nFailed {} out of {} scripts:", failed_scripts.len(), script_files.len());
            for (name, error) in &failed_scripts {
                eprintln!("  - {}: {}", name, error);
            }
            panic!("Some scripts failed. See errors above.");
        }

        // Clear deterministic RNG after all tests are complete
        #[cfg(not(target_arch = "wasm32"))]
        if run_execution_test {
            crate::random::clear_deterministic_rng();
        }

        let test_type = if run_execution_test { "compiled and executed" } else { "compiled" };
        println!("\n✓ All {} scripts {} successfully!", script_files.len(), test_type);
    }
    
    fn test_script_execution(
        engine: &rhai::Engine,
        scope: &mut rhai::Scope,
        ast: &rhai::AST,
        shared_state: &Rc<RefCell<SharedState>>,
        script_log: &Rc<RefCell<VecDeque<String>>>,
        script_name: &str,
        maps_dir: &Path,
    ) -> Result<(), String> {
        use crate::cs::SECTOR_SIZE;
        
        // Note: Deterministic RNG should be set before calling this function
        // This ensures each script test starts with the same seed
        
        // Create a minimal world for script execution
        let mut world = World::new();
        world.register::<Script>();
        world.register::<Name>();
        
        // Create a world script entity (not actually used, but good for testing structure)
        let _script_entity = world.create_entity()
            .with(Name { name: "World Script".to_string() })
            .with(Script {
                script: "".to_string(),
                ast: Some(ast.clone()),
                raw: false,
                script_type: ScriptType::World,
                run_once: false,
                has_run: false,
            })
            .build();
        
        // Initialize scope variables that scripts expect
        scope.set_value("tick", 0i64);
        scope.set_value("time", 0.0f64);
        scope.set_value("sim_time", 0.0f64);
        scope.set_value("time_of_day", 0.0f64);
        scope.set_value("day_length", 86400.0f64);
        scope.set_value("frame", 0i64);
        scope.set_value("GRID_WIDTH", SECTOR_SIZE.x as i64);
        scope.set_value("GRID_HEIGHT", SECTOR_SIZE.y as i64);
        
        // Take snapshot of initial state
        let initial_points = shared_state.borrow().points.len();
        let initial_state_snapshot: Vec<(i32, i32, u8)> = shared_state.borrow().points.iter()
            .map(|(p, c)| (p.x, p.y, *c))
            .collect();
        
        // Run script for 10 ticks
        const NUM_TICKS: i64 = 10;
        let mut runtime_errors = Vec::new();
        
        for tick in 0..NUM_TICKS {
            scope.set_value("tick", tick);
            scope.set_value("time", tick as f64 * 0.016); // ~60fps
            scope.set_value("sim_time", tick as f64 * 0.016);
            scope.set_value("time_of_day", (tick as f64 * 0.016) % 86400.0);
            scope.set_value("frame", tick);
            
            // Execute the script
            match engine.run_ast_with_scope(scope, ast) {
                Ok(_) => {}
                Err(err) => {
                    runtime_errors.push(format!("Tick {}: {}", tick, err));
                }
            }
            
            // Check for script log errors
            let log = script_log.borrow();
            for entry in log.iter() {
                if entry.to_lowercase().contains("error") {
                    runtime_errors.push(format!("Tick {}: Script log error: {}", tick, entry));
                }
            }
        }
        
        // Take snapshot of final state
        let final_points = shared_state.borrow().points.len();
        let final_state_snapshot: Vec<(i32, i32, u8)> = shared_state.borrow().points.iter()
            .map(|(p, c)| (p.x, p.y, *c))
            .collect();
        
        // Report results
        println!("    Initial points: {}, Final points: {}, State changed: {}", 
                 initial_points, 
                 final_points,
                 initial_points != final_points || initial_state_snapshot != final_state_snapshot);
        
        // Check for runtime errors
        if !runtime_errors.is_empty() {
            return Err(format!("Runtime errors occurred:\n  {}", runtime_errors.join("\n  ")));
        }
        
        // Create image from current state (same as in generate_script_snapshots)
        let mut current_image = image::GrayImage::new(SECTOR_SIZE.x as u32, SECTOR_SIZE.y as u32);
        
        // Fill with void (0) initially
        for pixel in current_image.pixels_mut() {
            *pixel = image::Luma([0u8]);
        }
        
        // Apply all points set by the script
        let points = shared_state.borrow().points.clone();
        for (point, cell_type) in points.iter() {
            let x = point.x as u32;
            let y = point.y as u32;
            if x < SECTOR_SIZE.x as u32 && y < SECTOR_SIZE.y as u32 {
                current_image.put_pixel(x, y, image::Luma([*cell_type]));
            }
        }
        
        // Load expected snapshot image
        let base_name = script_name.trim_end_matches(".rhai");
        let snapshots_dir = maps_dir.join("snapshots");
        let snapshot_path = snapshots_dir.join(format!("{}_snapshot.png", base_name));
        
        if !snapshot_path.exists() {
            return Err(format!("Snapshot file not found: {:?}. Run 'generate_snapshots' test first to create snapshots.", snapshot_path));
        }
        
        let expected_image = match image::open(&snapshot_path) {
            Ok(img) => img.to_luma8(),
            Err(err) => {
                return Err(format!("Failed to load snapshot image {:?}: {}", snapshot_path, err));
            }
        };
        
        // Compare images pixel by pixel
        if current_image.dimensions() != expected_image.dimensions() {
            return Err(format!(
                "Image dimensions mismatch: current {:?}, expected {:?}",
                current_image.dimensions(),
                expected_image.dimensions()
            ));
        }
        
        let mut mismatches = Vec::new();
        for (x, y, current_pixel) in current_image.enumerate_pixels() {
            let expected_pixel = expected_image.get_pixel(x, y);
            if current_pixel[0] != expected_pixel[0] {
                mismatches.push((x, y, current_pixel[0], expected_pixel[0]));
                // Limit the number of reported mismatches to avoid huge error messages
                if mismatches.len() >= 100 {
                    break;
                }
            }
        }
        
        if !mismatches.is_empty() {
            let mut error_msg = format!(
                "Image comparison failed: {} pixel(s) differ. First {} mismatch(es):\n",
                mismatches.len(),
                mismatches.len().min(10)
            );
            for (i, (x, y, current, expected)) in mismatches.iter().take(10).enumerate() {
                error_msg.push_str(&format!("  Pixel ({}, {}): current={}, expected={}\n", x, y, current, expected));
            }
            if mismatches.len() > 10 {
                error_msg.push_str(&format!("  ... and {} more mismatch(es)\n", mismatches.len() - 10));
            }
            return Err(error_msg);
        }
        
        println!("    ✓ Image matches snapshot");
        
        // Verify script executed successfully
        // Some scripts might check tick % N and only run on certain ticks, which is fine
        // But we should verify that the script actually did something meaningful
        
        // Additional validation: check if script produces any output or side effects
        // This helps ensure the test is actually running the script, not just passing silently
        let script_did_something = final_points > initial_points || 
                                    initial_state_snapshot != final_state_snapshot ||
                                    final_points > 0;
        
        // Note: Some scripts might only run on specific ticks (e.g., tick % 3 != 0),
        // so we don't fail if nothing happened, but we log it for visibility
        if !script_did_something {
            println!("    Warning: Script did not modify state (might be conditional on tick)");
        }
        
        Ok(())
    }
    
    // Guard to ensure deterministic RNG is cleared when test function exits
    struct DeterministicRngGuard;
    
    impl Drop for DeterministicRngGuard {
        fn drop(&mut self) {
            #[cfg(not(target_arch = "wasm32"))]
            crate::random::clear_deterministic_rng();
        }
    }
    
    fn generate_script_snapshots(maps_dir: &Path) {
        use crate::cs::SECTOR_SIZE;
        use crate::random::{set_deterministic_rng, clear_deterministic_rng};
        use rand::SeedableRng;
        
        // Create deterministic RNG with fixed seed for reproducible snapshots
        // Use the same seed as in test_script_execution
        let seed = [42u8; 32];
        let deterministic_rng = Rc::new(RefCell::new(rand::rngs::StdRng::from_seed(seed)));
        set_deterministic_rng(deterministic_rng.clone());
        
        // Ensure RNG is cleared when function exits
        let _guard = DeterministicRngGuard;
        
        // Create snapshots directory
        let snapshots_dir = maps_dir.join("snapshots");
        if let Err(err) = fs::create_dir_all(&snapshots_dir) {
            panic!("Failed to create snapshots directory {:?}: {}", snapshots_dir, err);
        }
        println!("Snapshots will be saved to: {:?}", snapshots_dir);
        
        // Create cell registry to get id_dict
        let cell_registry = CellRegistry::new();
        let mut id_dict: HashMap<String, u8> = HashMap::new();
        for a in cell_registry.pal.iter() {
            if a.id() != 0 {
                id_dict.insert(a.name().to_owned(), a.id());
            }
        }

        // Find all .rhai files
        let mut script_files = Vec::new();
        if let Ok(entries) = fs::read_dir(maps_dir) {
            for entry in entries {
                if let Ok(entry) = entry {
                    let path = entry.path();
                    if path.extension().and_then(|s| s.to_str()) == Some("rhai") {
                        script_files.push(path);
                    }
                }
            }
        }

        if script_files.is_empty() {
            panic!("No .rhai files found in {:?}", maps_dir);
        }

        // Sort for consistent output
        script_files.sort();

        println!("Generating snapshots for {} scripts...", script_files.len());

        // Generate snapshot for each script
        for script_path in &script_files {
            // Reset deterministic RNG before each script to ensure reproducible results
            use rand::SeedableRng;
            let seed = [42u8; 32];
            let deterministic_rng = Rc::new(RefCell::new(rand::rngs::StdRng::from_seed(seed)));
            set_deterministic_rng(deterministic_rng);
            
            let script_name = script_path.file_name()
                .and_then(|n| n.to_str())
                .unwrap_or("unknown");
            
            // Remove .rhai extension and create snapshot path in snapshots directory
            let base_name = script_name.trim_end_matches(".rhai");
            let snapshot_path = snapshots_dir.join(format!("{}_snapshot.png", base_name));
            
            match fs::read_to_string(script_path) {
                Ok(script_content) => {
                    // Setup Rhai engine
                    let mut rhai = rhai::Engine::new();
                    rhai.set_max_expr_depths(100, 100);
                    let mut rhai_scope = rhai::Scope::new();
                    let shared_state_rc = Rc::new(RefCell::new(SharedState::new()));
                    let script_log_rc = Rc::new(RefCell::new(VecDeque::<String>::with_capacity(30)));
                    
                    rhai_lib::register_rhai(
                        &mut rhai,
                        &mut rhai_scope,
                        shared_state_rc.clone(),
                        id_dict.clone(),
                        None,
                        script_log_rc.clone(),
                        None,
                    );
                    
                    match rhai.compile_with_scope(&mut rhai_scope, script_content.as_str()) {
                        Ok(ast) => {
                            // Run script for 10 ticks
                            const NUM_TICKS: i64 = 10;
                            
                            // Initialize scope variables
                            rhai_scope.set_value("GRID_WIDTH", SECTOR_SIZE.x as i64);
                            rhai_scope.set_value("GRID_HEIGHT", SECTOR_SIZE.y as i64);
                            
                            for tick in 0..NUM_TICKS {
                                rhai_scope.set_value("tick", tick);
                                rhai_scope.set_value("time", tick as f64 * 0.016);
                                rhai_scope.set_value("sim_time", tick as f64 * 0.016);
                                rhai_scope.set_value("time_of_day", (tick as f64 * 0.016) % 86400.0);
                                rhai_scope.set_value("frame", tick);
                                
                                // Execute the script
                                if let Err(err) = rhai.run_ast_with_scope(&mut rhai_scope, &ast) {
                                    eprintln!("  ✗ Runtime error at tick {}: {}", tick, err);
                                    continue;
                                }
                            }
                            
                            // Create image from points set by the script
                            let mut image = image::GrayImage::new(SECTOR_SIZE.x as u32, SECTOR_SIZE.y as u32);
                            
                            // Fill with void (0) initially
                            for pixel in image.pixels_mut() {
                                *pixel = image::Luma([0u8]);
                            }
                            
                            // Apply all points set by the script
                            let points = shared_state_rc.borrow().points.clone();
                            for (point, cell_type) in points.iter() {
                                let x = point.x as u32;
                                let y = point.y as u32;
                                if x < SECTOR_SIZE.x as u32 && y < SECTOR_SIZE.y as u32 {
                                    image.put_pixel(x, y, image::Luma([*cell_type]));
                                }
                            }
                            
                            // Save snapshot
                            match image.save(&snapshot_path) {
                                Ok(_) => {
                                    println!("  ✓ Generated snapshot: {}", snapshot_path.file_name().unwrap().to_string_lossy());
                                }
                                Err(err) => {
                                    eprintln!("  ✗ Failed to save snapshot for {}: {}", script_name, err);
                                }
                            }
                        }
                        Err(err) => {
                            eprintln!("  ✗ Failed to compile {}: {}", script_name, err);
                        }
                    }
                }
                Err(err) => {
                    eprintln!("  ✗ Failed to read {}: {}", script_name, err);
                }
            }
        }
        
        // Clear deterministic RNG after all snapshots are generated
        clear_deterministic_rng();
        
        println!("\n✓ Snapshot generation complete!");
    }
}
