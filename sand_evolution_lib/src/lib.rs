pub mod cells;
pub mod cs;
pub mod ecs;
pub mod evolution_app;
pub mod export_file;
pub mod fps_meter;
pub mod gbuffer;
pub mod shared_state;
pub mod state;
pub mod update;

pub mod resources;

use ::egui::FontDefinitions;
use chrono::Timelike;
use egui_wgpu_backend::{RenderPass, ScreenDescriptor};
use egui_winit_platform::{Platform, PlatformDescriptor};
use evolution_app::EvolutionApp;
use fps_meter::FpsMeter;
use state::State;
use std::cell::RefCell;
use std::rc::Rc;
use winit::event::Event::*;
use winit::event_loop::{ControlFlow, EventLoop, EventLoopBuilder};
use std::collections::HashMap;
use std::fmt::{Display, Formatter};

use crate::evolution_app::UserEventInfo;
const INITIAL_WIDTH: u32 = 1920;
const INITIAL_HEIGHT: u32 = 1080;

#[derive(Debug)]
struct CellTypeNotFound {
    name: String
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
use web_sys::{Navigator, Window};
#[cfg(target_arch = "wasm32")]
pub fn copy_text_to_clipboard(text: &str) -> Result<(), Box<dyn std::error::Error>> {
    code_to_file(&text.to_owned());
    Ok(())
}
#[cfg(target_arch = "wasm32")]
pub fn copy_text_from_clipboard() -> Result<String, Box<dyn std::error::Error>> {
    Ok("".to_owned())
}

use crate::shared_state::SharedState;
#[cfg(not(target_arch = "wasm32"))]
use clipboard::ClipboardProvider;
use futures::TryFutureExt;
use image::Luma;
use log::Level::Error;

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

#[cfg(feature = "wasm")]
use wasm_bindgen::prelude::*;
#[cfg(feature = "wasm")]
extern "C" {
    #[wasm_bindgen(js_namespace = Math)]
    fn random() -> f64;
}
#[cfg(feature = "wasm")]
pub fn my_rand() -> i64 {
    (random() * 10000.0) as i64
}

use crate::ecs::systems::{EntityScriptSystem, GravitySystem, MoveSystem};
use crate::export_file::code_to_file;
use crate::resources::rhai_resource::{RhaiResource, RhaiResourceStorage};
use crate::state::UpdateResult;
#[cfg(not(feature = "wasm"))]
use rand::Rng;
use rhai::EvalAltResult;
use specs::{Dispatcher, RunNow, WorldExt};
use wgpu::Queue;
use winit::dpi::PhysicalSize;

#[cfg(not(feature = "wasm"))]
pub fn my_rand() -> i64 {
    let mut rng = rand::thread_rng();
    rng.gen_range(0..10000) // Generating a random number between 0 and 9999
}

pub struct GameContext {
    pub world: specs::World,
    pub dispatcher: specs::Dispatcher<'static, 'static>,
    pub state: State,
}

impl GameContext {
    pub fn new(state: State) -> Self {
        let mut world = specs::World::new();
        let mut dispatcher = specs::DispatcherBuilder::new()
            .with(EntityScriptSystem, "entity_script__system", &[])
            .with(GravitySystem, "gravity_system", &[])
            .with(MoveSystem, "move_system", &[])
            .build();
        dispatcher.setup(&mut world);

        GameContext {
            world,
            dispatcher,
            state,
        }
    }

    pub fn dispatch(&mut self) {
        self.dispatcher.dispatch(&self.world);
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
pub async fn run(w: f32, h: f32, data: &[u8], script: String, base64: bool) {
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

    // We use the egui_winit_platform crate as the platform.
    let mut platform = Platform::new(PlatformDescriptor {
        physical_width: size.width as u32,
        physical_height: size.height as u32,
        scale_factor: window.scale_factor(),
        font_definitions: FontDefinitions::default(),
        style: Default::default(),
    });

    // We use the egui_wgpu_backend crate as the render backend.
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

    let mut evolution_app = EvolutionApp::new();

    if base64 == true {
        evolution_app.set_script(script);
    } else {
        if let Ok(decoded) = base64::decode(script) {
            if let Ok(decoded_str) = String::from_utf8(decoded) {
                evolution_app.set_script(decoded_str.as_str());
            }
        }
    }

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

        //rhai_scope.push_constant("RES_X", dimensions.0);
        //rhai_scope.push_constant("RES_Y", dimensions.1);

        {
            let moved_clone = shared_state_rc.clone();
            rhai.register_fn("set_cell", move |x: i64, y: i64, t: i64| {
                moved_clone
                    .borrow_mut()
                    .set_pixel(x as i32, y as i32, t as u8);
            });
        }
        {
            rhai.register_fn("type_id", move |name: &str| -> Result<i64, Box<EvalAltResult>> {
                if id_dict.contains_key(name) {
                    Ok(id_dict[name] as i64)
                } else {
                    Err(EvalAltResult::ErrorSystem(
                        "SystemError".into(),
                        Box::new(CellTypeNotFound{name: name.to_string()})
                    ).into())
                }
            });
        }
        rhai.register_fn("rand", move || -> i64 { my_rand() });
        rhai_scope.push("time", 0f64);
        game_context.world.insert(RhaiResource {
            storage: Some(RhaiResourceStorage {
                engine: rhai,
                scope: rhai_scope,
            }),
        });
    }

    let event_loop_proxy = event_loop.create_proxy();

    let start_time = instant::now();
    let mut last_frame_time = start_time;
    let mut collected_delta = 0.0;
    let mut event_loop_shared_state = shared_state_rc.clone();
    let mut upd_result = UpdateResult::default();
    event_loop.run(move |event, _, control_flow| {
        // Pass the winit events to the platform integration.
        platform.handle_event(&event);

        match event {
            RedrawRequested(..) => {
                let frame_start_time = (instant::now() - start_time) / 1000.0;
                let delta_t = frame_start_time - last_frame_time;
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

                let mut steps_per_this_frame = ((delta_t + collected_delta)
                    * evolution_app.simulation_steps_per_second as f64)
                    .floor();
                if evolution_app.simulation_steps_per_second != 0 {
                    let one_tick_delta = 1.0 / evolution_app.simulation_steps_per_second as f64;
                    collected_delta += delta_t - steps_per_this_frame * one_tick_delta;
                    if evolution_app.need_to_recompile {
                        if let Some(mut rhai_resource) = game_context
                            .world
                            .get_mut::<RhaiResource>()
                        {
                            if let Some(storage) = &mut rhai_resource.storage {
                                evolution_app.compile_script(storage);
                            } else {
                                println!("Warning: RhaiResource.storage is None");
                            }
                        } else {
                            println!("Warning: RhaiResource not found in the world");
                        }
                    }
                    let sim_steps = steps_per_this_frame as i32;

                    ///
                    /// UPDATE
                    ///
                    let upd_result = game_context.update(
                        &queue,
                        sim_steps,
                        &mut evolution_app,
                        &event_loop_shared_state,
                        window.inner_size(),
                        window.scale_factor(),
                    );
                    game_context.dispatch();
                    if upd_result.dropping {
                        evolution_app.simulation_steps_per_second -= 10;
                        if evolution_app.simulation_steps_per_second < 0 {
                            evolution_app.simulation_steps_per_second = 0
                        }
                    }
                }

                _ = game_context.state.render(
                    &device,
                    &queue,
                    &output_view,
                );

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
                    device_id,
                    input,
                    is_synthetic,
                } => {
                    if let winit::event::ElementState::Pressed = input.state {
                        if input.modifiers.logo() || input.modifiers.ctrl() {
                            // logo key is Command on macOS
                            if let Some(virtual_keycode) = input.virtual_keycode {
                                match virtual_keycode {
                                    winit::event::VirtualKeyCode::C => {
                                        _ = copy_text_to_clipboard(evolution_app.get_script());
                                    }
                                    winit::event::VirtualKeyCode::V => {
                                        if let Ok(result) = copy_text_from_clipboard() {
                                            evolution_app.set_script(result.as_str());
                                        }
                                    }
                                    _ => {}
                                }
                            }
                        }
                    }
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
                    }
                }
                _ => {}
            },
            UserEvent(event) => match event {
                UserEventInfo::ImageImport(image) => {
                    match image::load_from_memory(&image) {
                        Ok(img) => {
                            game_context.state.diffuse_rgba =
                                img.into_luma8()
                        }
                        _ => panic!("Invalid image format"),
                    };
                }
                UserEventInfo::TextImport(text) => {
                    match String::from_utf8(text) {
                        // Assuming `image` is a Vec<u8>
                        Ok(text) => {
                            evolution_app.set_script(text.as_str());
                        }
                        Err(_) => {
                            panic!("Invalid UTF-8 data"); // Or handle this error more gracefully
                        }
                    }
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
