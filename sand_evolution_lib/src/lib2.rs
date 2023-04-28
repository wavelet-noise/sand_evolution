mod cells;
mod cs;
mod fps_meter;
mod update;
mod gbuffer;

use std::f32::consts::E;

use cgmath::num_traits::clamp;
use egui::{CentralPanel, ComboBox, Context, FontDefinitions};
use egui_wgpu_backend::{RenderPass, ScreenDescriptor};
use egui_winit_platform::{Platform, PlatformDescriptor};
use fps_meter::FpsMeter;
use update::update_dim;
use wgpu::{InstanceDescriptor, Instance};
use wgpu::{util::DeviceExt, TextureFormat};
use winit::dpi::{LogicalPosition, PhysicalPosition, PhysicalSize};
use winit::event_loop::ControlFlow;
use winit::{event::Event::*, event_loop::EventLoop};

#[cfg(target_arch = "wasm32")]
use wasm_bindgen::prelude::*;
use winit::window::{Window, WindowBuilder};

use crate::cells::stone::Stone;
use crate::cells::wood::Wood;
use crate::cells::{burning_coal, water, CellRegistry, Prng};



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

const INDICES: &[u16] = &[0, 1, 3, 0, 3, 2];

#[repr(C)]
#[derive(Debug, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
struct WorldSettings {
    time: f32,
    _wasm_padding0: f32,
    _wasm_padding1: f32,
    _wasm_padding2: f32,
}

struct UpdateResult {
    simulation_step_average_time: f64,
    update_time: f64,
}

struct Camera {
    eye: cgmath::Point3<f32>,
    target: cgmath::Point3<f32>,
    up: cgmath::Vector3<f32>,
    aspect: f32,
    fovy: f32,
    znear: f32,
    zfar: f32,
}

#[rustfmt::skip]
pub const OPENGL_TO_WGPU_MATRIX: cgmath::Matrix4<f32> = cgmath::Matrix4::new(
    1.0, 0.0, 0.0, 0.0,
    0.0, 1.0, 0.0, 0.0,
    0.0, 0.0, 0.5, 0.0,
    0.0, 0.0, 0.5, 1.0,
);

impl Camera {
    fn build_view_projection_matrix(&self) -> cgmath::Matrix4<f32> {
        // 1.
        let view = cgmath::Matrix4::look_at_rh(self.eye, self.target, self.up);
        // 2.
        let proj = cgmath::perspective(cgmath::Deg(self.fovy), self.aspect, self.znear, self.zfar);

        // 3.
        return OPENGL_TO_WGPU_MATRIX * proj * view;
    }
}

pub struct State {
    render_pipeline: wgpu::RenderPipeline,
    vertex_buffer: wgpu::Buffer,
    index_buffer: wgpu::Buffer,
    num_indices: u32,
    world_settings: WorldSettings,
    settings_buffer: wgpu::Buffer,
    settings_bind_group: wgpu::BindGroup,
    start_time: f64,
    diffuse_bind_group: wgpu::BindGroup,
    diffuse_rgba: image::ImageBuffer<image::Luma<u8>, Vec<u8>>,
    diffuse_texture: wgpu::Texture,
    a: cs::PointType,
    b: cs::PointType,
    last_spawn: f32,
    pal_container: cells::CellRegistry,
    prng: cells::Prng,
    base_texture: wgpu::Texture,
    glow_texture: wgpu::Texture,
}

impl State {
    async fn new(
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        config: &wgpu::SurfaceConfiguration,
        surface: &wgpu::Surface,
    ) -> Self {
        let mut buf = [0u8; 4];
        let mut diffuse_rgba = image::GrayImage::from_fn(
            cs::SECTOR_SIZE.x as u32,
            cs::SECTOR_SIZE.y as u32,
            |x, y| {
                if x > 1
                    && y > 1
                    && x < cs::SECTOR_SIZE.x as u32 - 2
                    && y < cs::SECTOR_SIZE.y as u32 - 2
                {
                    _ = getrandom::getrandom(&mut buf);
                    return image::Luma([
                        if (buf[0] % 7 == 0 && y < cs::SECTOR_SIZE.y as u32 / 2) {
                            buf[1] % 4
                        } else {
                            0
                        },
                    ]);
                } else {
                    return image::Luma([Stone::id()]);
                }
            },
        );

        fn create_render_target(
            device: &wgpu::Device,
            size: wgpu::Extent3d,
            format: wgpu::TextureFormat,
        ) -> wgpu::Texture {
            device.create_texture(&wgpu::TextureDescriptor {
                label: Some("RenderTarget"),
                size,
                mip_level_count: 1,
                sample_count: 1,
                dimension: wgpu::TextureDimension::D2,
                format,
                usage: wgpu::TextureUsages::RENDER_ATTACHMENT
                    | wgpu::TextureUsages::TEXTURE_BINDING,
            })
        }

        for _ in 0..150 {
            _ = getrandom::getrandom(&mut buf);

            let nx = (((buf[0] as u32) << 8) | buf[1] as u32) % cs::SECTOR_SIZE.x as u32;
            let ny = (((buf[2] as u32) << 8) | buf[3] as u32) % cs::SECTOR_SIZE.y as u32;

            for x in 0..50 {
                diffuse_rgba.put_pixel(
                    clamp(nx + x, 0, cs::SECTOR_SIZE.x as u32 - 1),
                    clamp(ny, 0, cs::SECTOR_SIZE.y as u32 - 1),
                    image::Luma([Wood::id()]),
                );
            }
        }

        for _ in 0..100 {
            _ = getrandom::getrandom(&mut buf);

            let nx = (((buf[0] as u32) << 8) | buf[1] as u32) % cs::SECTOR_SIZE.x as u32;
            let ny = (((buf[2] as u32) << 8) | buf[3] as u32) % cs::SECTOR_SIZE.y as u32;

            for x in 0..20 {
                for y in 0..20 {
                    diffuse_rgba.put_pixel(
                        clamp(nx + x, 0, cs::SECTOR_SIZE.x as u32 - 1),
                        clamp(ny + y, 0, cs::SECTOR_SIZE.y as u32 - 1),
                        image::Luma([Wood::id()]),
                    );
                }
            }
        }

        let dimensions = diffuse_rgba.dimensions();

        let texture_size = wgpu::Extent3d {
            width: dimensions.0,
            height: dimensions.1,
            depth_or_array_layers: 1,
        };

        let cell_type_texture = device.create_texture(&wgpu::TextureDescriptor {
            // All textures are stored as 3D, we represent our 2D texture
            // by setting depth to 1.
            size: texture_size,
            mip_level_count: 1, // We'll talk about this a little later
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            // Most images are stored using sRGB so we need to reflect that here.
            format: wgpu::TextureFormat::R8Uint,
            // TEXTURE_BINDING tells wgpu that we want to use this texture in shaders
            // COPY_DST means that we want to copy data to this texture
            usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
            label: Some("diffuse_texture"),
        });

        let viewport_extent = wgpu::Extent3d {
            width: 1024,
            height: 768,
            depth_or_array_layers: 1,
        };

        let base_texture = create_render_target(
            &device,
            viewport_extent,
            wgpu::TextureFormat::Bgra8UnormSrgb,
        );
        let glow_texture =
            create_render_target(&device, viewport_extent, wgpu::TextureFormat::Rgba16Float);

        queue.write_texture(
            // Tells wgpu where to copy the pixel data
            wgpu::ImageCopyTexture {
                texture: &cell_type_texture,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
                aspect: wgpu::TextureAspect::All,
            },
            // The actual pixel data
            &diffuse_rgba,
            // The layout of the texture
            wgpu::ImageDataLayout {
                offset: 0,
                bytes_per_row: std::num::NonZeroU32::new(dimensions.0),
                rows_per_image: std::num::NonZeroU32::new(dimensions.1),
            },
            texture_size,
        );

        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Shader"),
            source: wgpu::ShaderSource::Wgsl(include_str!("shader.wgsl").into()),
        });

        let start_time = instant::now();

        let world_settings = WorldSettings {
            time: 0.0,
            _wasm_padding0: 0.0,
            _wasm_padding1: 1.0,
            _wasm_padding2: 2.0,
        };

        let raw_ptr = &world_settings as *const WorldSettings;
        let raw_ptr_bytes = raw_ptr as *mut u8;

        let settings_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Camera Buffer"),
            contents: unsafe {
                std::slice::from_raw_parts(raw_ptr_bytes, std::mem::size_of::<WorldSettings>())
            },

            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });

        let settings_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                entries: &[wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                }],
                label: Some("settings_bind_group_layout"),
            });

        let settings_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &settings_bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: settings_buffer.as_entire_binding(),
            }],
            label: Some("settings_bind_group"),
        });

        let mut tv = wgpu::TextureViewDescriptor::default().clone();
        tv.format = Some(TextureFormat::R8Uint);
        let diffuse_texture_view = cell_type_texture.create_view(&tv);
        let diffuse_sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            address_mode_w: wgpu::AddressMode::ClampToEdge,
            mag_filter: wgpu::FilterMode::Nearest,
            min_filter: wgpu::FilterMode::Nearest,
            mipmap_filter: wgpu::FilterMode::Nearest,
            ..Default::default()
        });

        let texture_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                entries: &[
                    wgpu::BindGroupLayoutEntry {
                        binding: 0,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Texture {
                            multisampled: false,
                            view_dimension: wgpu::TextureViewDimension::D2,
                            sample_type: wgpu::TextureSampleType::Uint,
                        },
                        count: None,
                    },
                    wgpu::BindGroupLayoutEntry {
                        binding: 1,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        // This should match the filterable field of the
                        // corresponding Texture entry above.
                        ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::NonFiltering),
                        count: None,
                    },
                ],
                label: Some("texture_bind_group_layout"),
            });

        let diffuse_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &texture_bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(&diffuse_texture_view),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::Sampler(&diffuse_sampler),
                },
            ],
            label: Some("diffuse_bind_group"),
        });

        //-------------------------------

        let render_pipeline_layout =
            device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("Render Pipeline Layout"),
                bind_group_layouts: &[&settings_bind_group_layout, &texture_bind_group_layout],
                push_constant_ranges: &[],
            });

        let render_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Render Pipeline"),
            layout: Some(&render_pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: "vs_main",
                buffers: &[Vertex::desc()],
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: "fs_main",
                targets: &[Some(wgpu::ColorTargetState {
                    format: config.format,
                    blend: Some(wgpu::BlendState {
                        color: wgpu::BlendComponent::OVER,
                        alpha: wgpu::BlendComponent::OVER,
                    }),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
            }),
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleList,
                strip_index_format: None,
                front_face: wgpu::FrontFace::Ccw,
                cull_mode: Some(wgpu::Face::Back),
                // Setting this to anything other than Fill requires Features::POLYGON_MODE_LINE
                // or Features::POLYGON_MODE_POINT
                polygon_mode: wgpu::PolygonMode::Fill,
                // Requires Features::DEPTH_CLIP_CONTROL
                unclipped_depth: false,
                // Requires Features::CONSERVATIVE_RASTERIZATION
                conservative: false,
            },
            depth_stencil: None,
            multisample: wgpu::MultisampleState {
                count: 1,
                mask: !0,
                alpha_to_coverage_enabled: false,
            },
            // If the pipeline will be used with a multiview render pass, this
            // indicates how many array layers the attachments will have.
            multiview: None,
        });

        let vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Vertex Buffer"),
            contents: bytemuck::cast_slice(VERTICES),
            usage: wgpu::BufferUsages::VERTEX,
        });
        let index_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Index Buffer"),
            contents: bytemuck::cast_slice(INDICES),
            usage: wgpu::BufferUsages::INDEX,
        });

        let num_indices = INDICES.len() as u32;

        let a = 0;
        let b = 0;

        let last_spawn = -5.0;

        let pal_container = CellRegistry::new();
        let prng = Prng::new();

        Self {
            render_pipeline,
            vertex_buffer,
            index_buffer,
            num_indices,
            world_settings,
            settings_buffer,
            settings_bind_group,
            start_time,
            diffuse_bind_group,
            diffuse_rgba,
            diffuse_texture: cell_type_texture,
            a,
            b,
            last_spawn,
            pal_container,
            prng,
            base_texture,
            glow_texture,
        }
    }

    // pub fn resize(&mut self, new_size: winit::dpi::PhysicalSize<u32>) {
    //     if new_size.width > 0 && new_size.height > 0 {
    //         self.size = new_size;
    //         self.config.width = new_size.width;
    //         self.config.height = new_size.height;
    //         self.surface.configure(&self.device, &self.config);
    //     }
    // }

    // #[allow(unused_variables)]
    // fn input(&mut self, event: &WindowEvent) -> bool {
    //     false
    // }

    fn spawn(&mut self, evolution_app: &mut EvolutionApp, window: &Window) {
        if let Some(position) = evolution_app.cursor_position {
            let scale_factor = window.scale_factor();
            let logical_position: LogicalPosition<f64> =
                LogicalPosition::from_physical(position, scale_factor);
            let window_size = window.inner_size();
            let scaled_window_size = PhysicalSize::new(
                window_size.width as f64 / scale_factor,
                window_size.height as f64 / scale_factor,
            );
            let percentage_position: (f64, f64) = (
                logical_position.x / scaled_window_size.width as f64,
                1.0 - logical_position.y / scaled_window_size.height as f64,
            );

            let mut buf = [0u8; MAXIMUM_NUMBER_OF_CELLS_TO_ADD + 1];
            _ = getrandom::getrandom(&mut buf);

            for i in 0..evolution_app.number_of_cells_to_add as usize {
                let px = percentage_position.0 * cs::SECTOR_SIZE.x as f64
                    + (self.prng.next() as f64 - 128.0) / 25.0;
                let py = percentage_position.1 * cs::SECTOR_SIZE.y as f64
                    + (self.prng.next() as f64 - 128.0) / 25.0;
                    self.diffuse_rgba.put_pixel(
                    px as u32,
                    py as u32,
                    image::Luma([self.pal_container.dict[&evolution_app.selected_option]]),
                );
            }
        }
    }

    fn update(&mut self, queue: &wgpu::Queue, sim_steps: u8, evolution_app: &mut EvolutionApp, window: &Window) -> UpdateResult {
        let update_start_time = instant::now();
        self.world_settings.time = (update_start_time - self.start_time) as f32 / 1000.0;

        queue.write_buffer(
            &self.settings_buffer,
            0,
            bytemuck::cast_slice(&[self.world_settings.time]),
        );

        let sim_upd_start_time = instant::now();

        let dimensions = self.diffuse_rgba.dimensions();

        if evolution_app.pressed {
            self.spawn(evolution_app, window);
        }

        update_dim(self, sim_steps, dimensions);

        let simulation_step_average_time = (instant::now() - sim_upd_start_time) / sim_steps as f64;

        let texture_size = wgpu::Extent3d {
            width: dimensions.0,
            height: dimensions.1,
            depth_or_array_layers: 1,
        };

        //self.diffuse_rgba = output;

        queue.write_texture(
            // Tells wgpu where to copy the pixel data
            wgpu::ImageCopyTexture {
                texture: &self.diffuse_texture,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
                aspect: wgpu::TextureAspect::All,
            },
            // The actual pixel data
            &self.diffuse_rgba,
            // The layout of the texture
            wgpu::ImageDataLayout {
                offset: 0,
                bytes_per_row: std::num::NonZeroU32::new(dimensions.0),
                rows_per_image: std::num::NonZeroU32::new(dimensions.1),
            },
            texture_size,
        );
        return UpdateResult {
            simulation_step_average_time,
            update_time: instant::now() - update_start_time,
        };
    }

    fn render(&mut self, device: &wgpu::Device, queue: &wgpu::Queue, view: &wgpu::TextureView) {
        let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("Render Encoder"),
        });

        {
            // Render pass for the color texture
    {
        let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("Render Pass"),
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view: &view,
                resolve_target: None,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Load,
                    store: true,
                },
            })],
            depth_stencil_attachment: None,
        });

        render_pass.set_pipeline(&self.render_pipeline);
        render_pass.set_bind_group(0, &self.settings_bind_group, &[]);
        render_pass.set_bind_group(1, &self.diffuse_bind_group, &[]);

        render_pass.set_vertex_buffer(0, self.vertex_buffer.slice(..));
        render_pass.set_index_buffer(self.index_buffer.slice(..), wgpu::IndexFormat::Uint16);

        render_pass.draw_indexed(0..self.num_indices, 0, 0..1);
    }

    // Render pass for the bloom texture
    {
        // let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
        //     label: Some("Bloom Render Pass"),
        //     color_attachments: &[Some(wgpu::RenderPassColorAttachment {
        //         view: &bloom_view,
        //         resolve_target: None,
        //         ops: wgpu::Operations {
        //             load: wgpu::LoadOp::Load,
        //             store: true,
        //         },
        //     })],
        //     depth_stencil_attachment: None,
        // });

        // render_pass.set_pipeline(&self.bloom_render_pipeline);
        // render_pass.set_bind_group(0, &self.settings_bind_group, &[]);
        // render_pass.set_bind_group(1, &self.diffuse_bind_group, &[]);

        // render_pass.set_vertex_buffer(0, self.vertex_buffer.slice(..));
        // render_pass.set_index_buffer(self.index_buffer.slice(..), wgpu::IndexFormat::Uint16);

        // render_pass.draw_indexed(0..self.num_indices, 0, 0..1);
    }

    queue.submit(std::iter::once(encoder.finish()));
        }

        queue.submit(std::iter::once(encoder.finish()));
    }
}

#[cfg_attr(target_arch = "wasm32", wasm_bindgen)]
pub async fn run(w: f32, h: f32) {
    let mut fps_meter = FpsMeter::new();

    cfg_if::cfg_if! {
        if #[cfg(target_arch = "wasm32")] {
            std::panic::set_hook(Box::new(console_error_panic_hook::hook));
            console_log::init_with_level(log::Level::Warn).expect("Could't initialize logger");
        } else {
            env_logger::init();
        }
    }

    let event_loop = EventLoop::new();
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

    let instance_desc = InstanceDescriptor {
        ..Default::default()
    };

    fn create_surface(window: &Window, instance: &wgpu::Instance) -> Result<wgpu::Surface, wgpu::SurfaceError> {
        window
            .create_surface(instance)
            .map_err(|e| wgpu::SurfaceError::NativeWindowIntegrationError(Box::new(e)))
    }
    
    // Create a new instance with the given descriptor
    let instance = Instance::new(instance_desc);
    let surface = create_surface(&window, &instance)?;
let size = window.inner_size();
let surface_config = wgpu::SurfaceConfiguration {
    usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
    format: surface.get_preferred_format(&adapter).unwrap(),
    width: size.width,
    height: size.height,
    present_mode: wgpu::PresentMode::Fifo,
};

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
        width: size.width as u32,
        height: size.height as u32,
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

    let mut state = State::new(&device, &queue, &surface_config, &surface).await;

    let mut evolution_app = EvolutionApp::new();

    for a in state.pal_container.pal.iter() {
        if a.id() != 0 {
            evolution_app.options.push(a.name());
        }
    }

    let start_time = instant::now();
    event_loop.run(move |event, _, control_flow| {
        // Pass the winit events to the platform integration.
        platform.handle_event(&event);

        match event {
            RedrawRequested(..) => {
                platform.update_time((instant::now() - start_time) / 1000.0);

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

                let upd_result =
                    state.update(&queue, evolution_app.simulation_steps_per_frame as u8, &mut evolution_app, &window);
                _ = state.render(&device, &queue, &output_view);

                // Begin to draw the UI frame.
                //
                //    //////
                //    /    /
                //    //////
                //

                platform.begin_frame();

                evolution_app.ui(&platform.context(), &mut state, &mut fps_meter, &upd_result);

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
                winit::event::WindowEvent::MouseInput { state, button, .. } => {
                    if button == winit::event::MouseButton::Left {
                        if state == winit::event::ElementState::Pressed {
                            evolution_app.pressed = true;
                        } else {
                            evolution_app.pressed = false;
                        }
                    }
                }
                _ => {}
            },
            _ => (),
        }
    });
}

const MAXIMUM_NUMBER_OF_CELLS_TO_ADD: usize = 10000;
const MAXIMUM_NUMBER_OF_STRUCTURES_TO_ADD: usize = 500;