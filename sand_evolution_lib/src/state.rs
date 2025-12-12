use cgmath::num_traits::clamp;
use std::cell::RefCell;
use std::rc::Rc;
use wgpu::{util::DeviceExt, TextureFormat, TextureView};
use winit::dpi::{LogicalPosition, PhysicalSize};

use crate::shared_state::SharedState;
use crate::{
    cells::{stone::Stone, wood::Wood, CellRegistry, Prng},
    cs::{self, PointType},
    evolution_app::EvolutionApp,
    gbuffer::GBuffer,
    update, Vertex, INDICES, VERTICES,
};

#[repr(C)]
#[derive(Debug, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct WorldSettings {
    time: f32,
    res_x: f32,
    res_y: f32,
    display_mode: f32, // 0.0 = Normal, 1.0 = Temperature, 2.0 = Both
    /// Global temperature offset (degrees). Effective temperature = layer + global_temperature.
    global_temperature: f32,
    // Keep uniform size a multiple of 16 bytes.
    pub(crate) _pad0: f32,
    pub(crate) _pad1: f32,
    pub(crate) _pad2: f32,
    pub(crate) _pad3: f32,
    pub(crate) _pad4: f32,
    pub(crate) _pad5: f32,
}
#[derive(Default)]
pub struct UpdateResult {
    pub simulation_step_average_time: f64,
    pub update_time: f64,
    pub dropping: bool,
}

#[derive(Debug, Clone)]
pub struct DayNightCycle {
    /// Length of a full day in simulation seconds.
    pub day_length_seconds: f32,
    /// Current time of day in [0, day_length_seconds).
    pub time_of_day_seconds: f32,
    /// Multiplier for how fast the cycle advances relative to simulation time.
    pub speed: f32,
    /// Pause only the day/night cycle (simulation can still run).
    pub paused: bool,
    /// Rotation offset so that time=0 matches the initial art direction.
    pub sun_angle_offset_rad: f32,
    /// Shadow strength sent to shader (0..1).
    pub shadow_strength: f32,
    /// Shadow length in raymarch steps (1..64).
    pub shadow_length_steps: f32,
    /// Controls how quickly shadow fades with distance (0 disables distance attenuation).
    pub shadow_distance_falloff: f32,
}

impl DayNightCycle {
    pub fn new(day_length_seconds: f32, initial_dir_xy: (f32, f32)) -> Self {
        let (mut x, mut y) = initial_dir_xy;
        let len2 = x * x + y * y;
        if len2 > 1e-12 {
            let inv = 1.0 / len2.sqrt();
            x *= inv;
            y *= inv;
        } else {
            x = -0.8;
            y = 0.4;
        }

        Self {
            day_length_seconds: day_length_seconds.max(0.1),
            time_of_day_seconds: 0.0,
            speed: 1.0,
            // By default keep time-of-day fixed (can be toggled in UI).
            paused: true,
            sun_angle_offset_rad: y.atan2(x),
            shadow_strength: 0.9,
            shadow_length_steps: 26.0,
            shadow_distance_falloff: 1.0,
        }
    }

    /// Advances the cycle and returns new normalized light direction (x,y).
    pub fn advance(&mut self, dt_sim_seconds: f32) -> (f32, f32) {
        if self.paused {
            return self.current_dir();
        }

        let dt = (dt_sim_seconds.max(0.0)) * self.speed.max(0.0);
        self.time_of_day_seconds =
            (self.time_of_day_seconds + dt).rem_euclid(self.day_length_seconds.max(0.1));
        self.current_dir()
    }

    pub fn current_dir(&self) -> (f32, f32) {
        let phase = (self.time_of_day_seconds / self.day_length_seconds.max(0.1)).clamp(0.0, 1.0);
        let angle = phase * std::f32::consts::TAU + self.sun_angle_offset_rad;
        (angle.cos(), angle.sin())
    }
}

pub struct State {
    render_pipeline: wgpu::RenderPipeline,
    bloom_render_pipeline: wgpu::RenderPipeline,
    gbuffer_collect_pipeline: wgpu::RenderPipeline,
    hblur_render_pipeline: wgpu::RenderPipeline,
    vblur_render_pipeline: wgpu::RenderPipeline,
    vertex_buffer: wgpu::Buffer,
    index_buffer: wgpu::Buffer,
    num_indices: u32,
    pub(crate) world_settings: WorldSettings,
    settings_buffer: wgpu::Buffer,
    settings_bind_group: wgpu::BindGroup,
    float_texture_plus_sampler_bgl: wgpu::BindGroupLayout,
    float_texture_plus_sampler_plus_texture_bgl: wgpu::BindGroupLayout,
    pub start_time: f64,
    type_bind_group: wgpu::BindGroup,
    shadow_props_bind_group: wgpu::BindGroup,
    gbuffer_combine_bind_group: wgpu::BindGroup,
    pub diffuse_rgba: image::ImageBuffer<image::Luma<u8>, Vec<u8>>,
    pub loaded_rgba: image::ImageBuffer<image::Luma<u8>, Vec<u8>>,
    diffuse_texture: wgpu::Texture,
    shadow_props_texture: wgpu::Texture,
    pub flip: cs::PointType,
    pub flop: cs::PointType,
    last_spawn: f32,
    pub pal_container: CellRegistry,
    pub prng: Prng,
    base_texture: wgpu::Texture,
    glow_texture: wgpu::Texture,
    gbuffer: GBuffer,
    surface_format: TextureFormat,
    pub toggled: bool,
    pub tick: i64,
    pub frame: i64,
    /// Simulation time in seconds (advances with ticks, not wall clock).
    pub sim_time_seconds: f64,
    pub day_night: DayNightCycle,
    // Temperature system for each cell (degrees)
    pub cell_temperatures: Vec<f32>,
    /// Global/base temperature (degrees).
    pub global_temperature: f32,
    // Temperature texture for GPU
    temperature_texture: wgpu::Texture,
    temperature_bind_group: wgpu::BindGroup,
}

/// Minimum allowed temperature in the simulation (degrees).
pub const TEMP_MIN: f32 = -100.0;
/// Maximum allowed temperature in the simulation (degrees).
///
/// Note: some cells use ignition thresholds above 100 (e.g. coal at 150),
/// so this needs to be > 150 to make those rules reachable.
pub const TEMP_MAX: f32 = 500.0;

impl State {
    pub(crate) fn update_with_data(&mut self, p0: &[u8]) {
        if p0.is_empty() || p0.len() == 0 {
            self.generate_simple();
        } else {
            let res = image::load_from_memory(p0).expect("Load from memory failed");
            self.loaded_rgba = res.to_luma8();
            self.diffuse_rgba = res.to_luma8();
            // Imported map should not inherit previous temperature field.
            self.reset_temperatures();
            println!("Some image loaded");
        }
    }
}

struct MyState {
    name: String,
    count: usize,
}

impl State {
    pub fn generate_simple(&mut self) {
        let mut buf = [0u8; 4];
        self.diffuse_rgba = image::GrayImage::from_fn(
            cs::SECTOR_SIZE.x as u32,
            cs::SECTOR_SIZE.y as u32,
            |x, y| {
                if x > 1
                    && y > 1
                    && x < cs::SECTOR_SIZE.x as u32 - 2
                    && y < cs::SECTOR_SIZE.y as u32 - 2
                {
                    _ = getrandom::getrandom(&mut buf);
                    return image::Luma([if buf[0] % 7 == 0 && y < cs::SECTOR_SIZE.y as u32 / 2 {
                        buf[1] % 4
                    } else {
                        0
                    }]);
                } else {
                    return image::Luma([Stone::id()]);
                }
            },
        );

        for _ in 0..150 {
            _ = getrandom::getrandom(&mut buf);

            let nx = (((buf[0] as u32) << 8) | buf[1] as u32) % cs::SECTOR_SIZE.x as u32;
            let ny = (((buf[2] as u32) << 8) | buf[3] as u32) % cs::SECTOR_SIZE.y as u32;

            for x in 0..50 {
                self.diffuse_rgba.put_pixel(
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
                    self.diffuse_rgba.put_pixel(
                        clamp(nx + x, 0, cs::SECTOR_SIZE.x as u32 - 1),
                        clamp(ny + y, 0, cs::SECTOR_SIZE.y as u32 - 1),
                        image::Luma([Wood::id()]),
                    );
                }
            }
        }

        for _ in 0..3 {
            for cell in self.pal_container.pal.iter() {
                if cell.id() != 0 {
                    _ = getrandom::getrandom(&mut buf);

                    let nx = (((buf[0] as u32) << 8) | buf[1] as u32) % cs::SECTOR_SIZE.x as u32;
                    let ny = (((buf[2] as u32) << 8) | buf[3] as u32) % cs::SECTOR_SIZE.y as u32;

                    for x in 0..35 {
                        for y in 0..20 {
                            self.diffuse_rgba.put_pixel(
                                clamp(nx + x, 0, cs::SECTOR_SIZE.x as u32 - 1),
                                clamp(ny + y, 0, cs::SECTOR_SIZE.y as u32 - 1),
                                image::Luma([cell.id()]),
                            );
                        }
                    }
                }
            }
        }

        // New random map should start from a clean temperature field.
        self.reset_temperatures();
    }
    pub fn new(
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        config: &wgpu::SurfaceConfiguration,
        _surface: &wgpu::Surface,
        surface_format: wgpu::TextureFormat,
    ) -> Self {
        let diffuse_rgba = image::GrayImage::from_fn(
            cs::SECTOR_SIZE.x as u32,
            cs::SECTOR_SIZE.y as u32,
            |x, y| {
                if x > 1
                    && y > 1
                    && x < cs::SECTOR_SIZE.x as u32 - 2
                    && y < cs::SECTOR_SIZE.y as u32 - 2
                {
                    return image::Luma([0]);
                } else {
                    return image::Luma([Stone::id()]);
                }
            },
        );

        let pal_container = CellRegistry::new();

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

        let dimensions = diffuse_rgba.dimensions();

        let gbuffer = GBuffer::new(device, dimensions.0, dimensions.1, surface_format);

        let texture_size = wgpu::Extent3d {
            width: dimensions.0,
            height: dimensions.1,
            depth_or_array_layers: 1,
        };

        let cell_type_texture = device.create_texture(&wgpu::TextureDescriptor {
            size: texture_size,
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: TextureFormat::R8Uint,
            usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
            label: Some("diffuse_texture"),
        });

        // Per-cell-type shadow properties: RGBA where
        // - RGB is the shadow color multiplier (255 = no darkening),
        // - A is the shadow opacity (0 = does not affect, 255 = fully affects).
        // Stored as a 256x1 RGBA8 texture and indexed by cell id in the shader.
        let shadow_props_texture = device.create_texture(&wgpu::TextureDescriptor {
            size: wgpu::Extent3d {
                width: 256,
                height: 1,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: TextureFormat::Rgba8Unorm,
            usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
            label: Some("shadow_props_texture"),
        });

        let mut shadow_props = [255u8; 256 * 4];
        for (i, cell) in pal_container.pal.iter().enumerate().take(256) {
            let rgba = cell.shadow_rgba();
            let o = i * 4;
            shadow_props[o..o + 4].copy_from_slice(&rgba);
        }

        let viewport_extent = wgpu::Extent3d {
            width: 1024,
            height: 768,
            depth_or_array_layers: 1,
        };

        let base_texture = create_render_target(&device, viewport_extent, surface_format);
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

        queue.write_texture(
            wgpu::ImageCopyTexture {
                texture: &shadow_props_texture,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
                aspect: wgpu::TextureAspect::All,
            },
            &shadow_props,
            wgpu::ImageDataLayout {
                offset: 0,
                bytes_per_row: std::num::NonZeroU32::new(256 * 4),
                rows_per_image: std::num::NonZeroU32::new(1),
            },
            wgpu::Extent3d {
                width: 256,
                height: 1,
                depth_or_array_layers: 1,
            },
        );

        let type_render_and_fullscreen_vertex =
            device.create_shader_module(wgpu::ShaderModuleDescriptor {
                label: Some("Shader"),
                source: wgpu::ShaderSource::Wgsl(include_str!("shader.wgsl").into()),
            });

        let bloom_shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Shader"),
            source: wgpu::ShaderSource::Wgsl(include_str!("bloom.wgsl").into()),
        });

        let hblur_shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Shader"),
            source: wgpu::ShaderSource::Wgsl(include_str!("hor_blur.wgsl").into()),
        });

        let vblur_shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Shader"),
            source: wgpu::ShaderSource::Wgsl(include_str!("vert_blur.wgsl").into()),
        });

        let gbuffer_collect_shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Shader"),
            source: wgpu::ShaderSource::Wgsl(include_str!("gbuffer_collect_shader.wgsl").into()),
        });

        let start_time = instant::now();

        // Day/night defaults (also used to seed shader light direction).
        // "Almost strictly down" in texel space.
        // Note: in this project, positive Y in texel space corresponds to "up" on screen,
        // so "down" is negative Y.
        let initial_sun_dir = (0.03f32, -1.0f32);
        let day_night = DayNightCycle::new(120.0, initial_sun_dir);
        let (sun_x, sun_y) = day_night.current_dir();

        let world_settings = WorldSettings {
            time: 0.0,
            res_x: dimensions.0 as f32,
            res_y: dimensions.1 as f32,
            display_mode: 0.0, // Normal mode by default
            global_temperature: 21.0,
            // Directional light/shadows (used by shader.wgsl):
            // _pad0/_pad1 = sun direction (texel space)
            // _pad2 = shadow strength [0..2]
            // _pad3 = shadow length in steps [1..64]
            // _pad4 = distance falloff exponent [0..4]
            _pad0: sun_x,
            _pad1: sun_y,
            _pad2: day_night.shadow_strength,
            _pad3: day_night.shadow_length_steps,
            _pad4: day_night.shadow_distance_falloff,
            _pad5: 0.0,
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

        let mut type_texture_view = wgpu::TextureViewDescriptor::default().clone();
        type_texture_view.format = Some(TextureFormat::R8Uint);

        let mut shadow_props_texture_view_desc = wgpu::TextureViewDescriptor::default().clone();
        shadow_props_texture_view_desc.format = Some(TextureFormat::Rgba8Unorm);

        let mut color_texture_view = wgpu::TextureViewDescriptor::default().clone();
        color_texture_view.format = Some(surface_format);

        let cell_type_texture_view = cell_type_texture.create_view(&type_texture_view);
        let shadow_props_texture_view =
            shadow_props_texture.create_view(&shadow_props_texture_view_desc);

        let type_texture_sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            address_mode_w: wgpu::AddressMode::ClampToEdge,
            mag_filter: wgpu::FilterMode::Nearest,
            min_filter: wgpu::FilterMode::Nearest,
            mipmap_filter: wgpu::FilterMode::Nearest,
            ..Default::default()
        });

        let color_texture_sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            address_mode_w: wgpu::AddressMode::ClampToEdge,
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Linear,
            mipmap_filter: wgpu::FilterMode::Linear,
            ..Default::default()
        });

        let uint_texture_plus_sampler_bgl =
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
                label: Some("cell_type_bind_group_layout"),
            });

        let shadow_props_texture_bgl =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                entries: &[
                    wgpu::BindGroupLayoutEntry {
                        binding: 0,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Texture {
                            multisampled: false,
                            view_dimension: wgpu::TextureViewDimension::D2,
                            sample_type: wgpu::TextureSampleType::Float { filterable: false },
                        },
                        count: None,
                    },
                    wgpu::BindGroupLayoutEntry {
                        binding: 1,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::NonFiltering),
                        count: None,
                    },
                ],
                label: Some("shadow_props_texture_bind_group_layout"),
            });

        let float_texture_plus_sampler_bgl =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                entries: &[
                    wgpu::BindGroupLayoutEntry {
                        binding: 0,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Texture {
                            multisampled: false,
                            view_dimension: wgpu::TextureViewDimension::D2,
                            sample_type: wgpu::TextureSampleType::Float { filterable: true },
                        },
                        count: None,
                    },
                    wgpu::BindGroupLayoutEntry {
                        binding: 1,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        // This should match the filterable field of the
                        // corresponding Texture entry above.
                        ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                        count: None,
                    },
                ],
                label: Some("gbuffer_bind_group_layout"),
            });

        let float_texture_plus_sampler_plus_texture_bgl =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                entries: &[
                    wgpu::BindGroupLayoutEntry {
                        binding: 0,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Texture {
                            multisampled: false,
                            view_dimension: wgpu::TextureViewDimension::D2,
                            sample_type: wgpu::TextureSampleType::Float { filterable: true },
                        },
                        count: None,
                    },
                    wgpu::BindGroupLayoutEntry {
                        binding: 1,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        // This should match the filterable field of the
                        // corresponding Texture entry above.
                        ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                        count: None,
                    },
                    wgpu::BindGroupLayoutEntry {
                        binding: 2,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Texture {
                            multisampled: false,
                            view_dimension: wgpu::TextureViewDimension::D2,
                            sample_type: wgpu::TextureSampleType::Float { filterable: true },
                        },
                        count: None,
                    },
                ],
                label: Some("gbuffer_bind_group_layout"),
            });

        let type_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &uint_texture_plus_sampler_bgl,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(&cell_type_texture_view),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::Sampler(&type_texture_sampler),
                },
            ],
            label: Some("type_bind_group"),
        });

        let shadow_props_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &shadow_props_texture_bgl,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(&shadow_props_texture_view),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::Sampler(&type_texture_sampler),
                },
            ],
            label: Some("shadow_props_bind_group"),
        });

        let gbuffer_combine_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &float_texture_plus_sampler_bgl,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(&gbuffer.albedo_view),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::Sampler(&color_texture_sampler),
                },
            ],
            label: Some("gbuffer_combine_bind_group"),
        });

        // Create temperature texture (4x smaller for faster diffusion)
        let temp_texture_size = wgpu::Extent3d {
            width: dimensions.0 / 4,
            height: dimensions.1 / 4,
            depth_or_array_layers: 1,
        };
        let temperature_texture = device.create_texture(&wgpu::TextureDescriptor {
            size: temp_texture_size,
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: TextureFormat::R32Float,
            usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
            label: Some("temperature_texture"),
        });

        let mut temperature_texture_view = wgpu::TextureViewDescriptor::default().clone();
        temperature_texture_view.format = Some(TextureFormat::R32Float);
        let temperature_texture_view = temperature_texture.create_view(&temperature_texture_view);

        let temperature_texture_sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            address_mode_w: wgpu::AddressMode::ClampToEdge,
            mag_filter: wgpu::FilterMode::Nearest,
            min_filter: wgpu::FilterMode::Nearest,
            mipmap_filter: wgpu::FilterMode::Nearest,
            ..Default::default()
        });

        // Create bind group layout for temperature texture
        // R32Float doesn't support filtering, so we use NonFiltering
        let temperature_texture_bgl =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                entries: &[
                    wgpu::BindGroupLayoutEntry {
                        binding: 0,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Texture {
                            multisampled: false,
                            view_dimension: wgpu::TextureViewDimension::D2,
                            sample_type: wgpu::TextureSampleType::Float { filterable: false },
                        },
                        count: None,
                    },
                    wgpu::BindGroupLayoutEntry {
                        binding: 1,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::NonFiltering),
                        count: None,
                    },
                ],
                label: Some("temperature_texture_bind_group_layout"),
            });

        let temperature_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &temperature_texture_bgl,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(&temperature_texture_view),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::Sampler(&temperature_texture_sampler),
                },
            ],
            label: Some("temperature_bind_group"),
        });

        //-------------------------------

        let type_render_pipeline_layout =
            device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("Render Pipeline Layout"),
                bind_group_layouts: &[
                    &settings_bind_group_layout,
                    &uint_texture_plus_sampler_bgl,
                    &temperature_texture_bgl,
                    &shadow_props_texture_bgl,
                ],
                push_constant_ranges: &[],
            });

        let settings_plus_texture_sampler_pl =
            device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("Gbuffer Render Pipeline Layout"),
                bind_group_layouts: &[&settings_bind_group_layout, &float_texture_plus_sampler_bgl],
                push_constant_ranges: &[],
            });

        let settings_plus_texture_sampler_plus_texture_pl =
            device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("Gbuffer Render Pipeline Layout"),
                bind_group_layouts: &[
                    &settings_bind_group_layout,
                    &float_texture_plus_sampler_plus_texture_bgl,
                ],
                push_constant_ranges: &[],
            });

        let bloom_render_pipeline =
            device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
                label: Some("Bloom Render Pipeline"),
                layout: Some(&settings_plus_texture_sampler_plus_texture_pl),
                vertex: wgpu::VertexState {
                    module: &bloom_shader,
                    entry_point: "vs_main",
                    buffers: &[Vertex::desc()],
                },
                fragment: Some(wgpu::FragmentState {
                    module: &bloom_shader,
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

        let hblur_render_pipeline =
            device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
                label: Some("Bloom Render Pipeline"),
                layout: Some(&settings_plus_texture_sampler_pl),
                vertex: wgpu::VertexState {
                    module: &hblur_shader,
                    entry_point: "vs_main",
                    buffers: &[Vertex::desc()],
                },
                fragment: Some(wgpu::FragmentState {
                    module: &hblur_shader,
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
                    polygon_mode: wgpu::PolygonMode::Fill,
                    unclipped_depth: false,
                    conservative: false,
                },
                depth_stencil: None,
                multisample: wgpu::MultisampleState {
                    count: 1,
                    mask: !0,
                    alpha_to_coverage_enabled: false,
                },
                multiview: None,
            });

        let vblur_render_pipeline =
            device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
                label: Some("Bloom Render Pipeline"),
                layout: Some(&settings_plus_texture_sampler_pl),
                vertex: wgpu::VertexState {
                    module: &vblur_shader,
                    entry_point: "vs_main",
                    buffers: &[Vertex::desc()],
                },
                fragment: Some(wgpu::FragmentState {
                    module: &vblur_shader,
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
                    polygon_mode: wgpu::PolygonMode::Fill,
                    unclipped_depth: false,
                    conservative: false,
                },
                depth_stencil: None,
                multisample: wgpu::MultisampleState {
                    count: 1,
                    mask: !0,
                    alpha_to_coverage_enabled: false,
                },
                multiview: None,
            });

        let gbuffer_collect_pipeline =
            device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
                label: Some("Gbuffer Render Pipeline"),
                layout: Some(&settings_plus_texture_sampler_pl),
                vertex: wgpu::VertexState {
                    module: &type_render_and_fullscreen_vertex,
                    entry_point: "vs_main",
                    buffers: &[Vertex::desc()],
                },
                fragment: Some(wgpu::FragmentState {
                    module: &gbuffer_collect_shader,
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

        let type_render_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Render Pipeline"),
            layout: Some(&type_render_pipeline_layout),
            vertex: wgpu::VertexState {
                module: &type_render_and_fullscreen_vertex,
                entry_point: "vs_main",
                buffers: &[Vertex::desc()],
            },
            fragment: Some(wgpu::FragmentState {
                module: &type_render_and_fullscreen_vertex,
                entry_point: "fs_main",
                targets: &[
                    Some(wgpu::ColorTargetState {
                        format: config.format,
                        blend: Some(wgpu::BlendState {
                            color: wgpu::BlendComponent::OVER,
                            alpha: wgpu::BlendComponent::OVER,
                        }),
                        write_mask: wgpu::ColorWrites::ALL,
                    }),
                    Some(wgpu::ColorTargetState {
                        format: config.format,
                        blend: Some(wgpu::BlendState {
                            color: wgpu::BlendComponent::OVER,
                            alpha: wgpu::BlendComponent::OVER,
                        }),
                        write_mask: wgpu::ColorWrites::ALL,
                    }),
                ],
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
        let prng = Prng::new();

        // Initialize cell temperatures (initial temperature = 0.0)
        // Use a reduced grid (4x smaller) for optimization
        let temp_width = (cs::SECTOR_SIZE.x / 4) as usize;
        let temp_height = (cs::SECTOR_SIZE.y / 4) as usize;
        let total_cells = temp_width * temp_height;
        let cell_temperatures = vec![0.0; total_cells];

        Self {
            render_pipeline: type_render_pipeline,
            bloom_render_pipeline,
            gbuffer_collect_pipeline,
            vertex_buffer,
            hblur_render_pipeline,
            vblur_render_pipeline,
            index_buffer,
            num_indices,
            world_settings,
            settings_buffer,
            settings_bind_group,
            start_time,
            type_bind_group,
            shadow_props_bind_group,
            loaded_rgba: diffuse_rgba.clone(),
            diffuse_rgba,
            gbuffer_combine_bind_group,
            diffuse_texture: cell_type_texture,
            shadow_props_texture,
            float_texture_plus_sampler_bgl,
            float_texture_plus_sampler_plus_texture_bgl,
            flip: a,
            flop: b,
            last_spawn,
            pal_container,
            prng,
            base_texture,
            glow_texture,
            gbuffer,
            surface_format,
            toggled: true,
            tick: 0,
            frame: 0,
            sim_time_seconds: 0.0,
            day_night,
            cell_temperatures,
            global_temperature: 21.0,
            temperature_texture,
            temperature_bind_group,
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
    //     falsex
    // }

    fn set_cell(&mut self, x: i32, y: i32, t: u8) {
        self.diffuse_rgba
            .put_pixel(x as u32, y as u32, image::Luma([t]));
    }

    // Convert full grid coordinates to reduced grid coordinates
    fn temp_coords_to_index(&self, i: PointType, j: PointType) -> usize {
        let temp_x = (i / 4) as usize;
        let temp_y = (j / 4) as usize;
        let temp_width = (cs::SECTOR_SIZE.x / 4) as usize;
        temp_y * temp_width + temp_x
    }

    // Get cell temperature by index (in reduced grid)
    pub fn get_cell_temperature(&self, index: usize) -> f32 {
        if index < self.cell_temperatures.len() {
            (self.cell_temperatures[index] + self.global_temperature)
                .max(TEMP_MIN)
                .min(TEMP_MAX)
        } else {
            0.0
        }
    }

    // Get cell temperature by coordinates (full grid)
    pub fn get_temperature(&self, i: PointType, j: PointType) -> f32 {
        let idx = self.temp_coords_to_index(i, j);
        self.get_cell_temperature(idx)
    }

    // Set cell temperature by index (in reduced grid)
    pub fn set_cell_temperature(&mut self, index: usize, temp: f32) {
        if index < self.cell_temperatures.len() {
            let clamped = temp.max(TEMP_MIN).min(TEMP_MAX);
            self.cell_temperatures[index] = clamped - self.global_temperature;
        }
    }

    // Set cell temperature by coordinates (full grid)
    pub fn set_temperature(&mut self, i: PointType, j: PointType, temp: f32) {
        let idx = self.temp_coords_to_index(i, j);
        self.set_cell_temperature(idx, temp);
    }

    // Add temperature to cell (for heat generation)
    pub fn add_temperature(&mut self, i: PointType, j: PointType, delta: f32) {
        let idx = self.temp_coords_to_index(i, j);
        if idx < self.cell_temperatures.len() {
            self.cell_temperatures[idx] += delta;
            // Clamp effective temperature to reasonable limits
            let eff = self.cell_temperatures[idx] + self.global_temperature;
            let clamped = eff.max(TEMP_MIN).min(TEMP_MAX);
            self.cell_temperatures[idx] = clamped - self.global_temperature;
        }
    }

    // Add temperature to cell by index (in reduced grid)
    pub fn add_cell_temperature(&mut self, index: usize, delta: f32) {
        if index < self.cell_temperatures.len() {
            self.cell_temperatures[index] += delta;
            // Clamp effective temperature to reasonable limits
            let eff = self.cell_temperatures[index] + self.global_temperature;
            let clamped = eff.max(TEMP_MIN).min(TEMP_MAX);
            self.cell_temperatures[index] = clamped - self.global_temperature;
        }
    }

    /// Reset per-cell temperature field (local delta, reduced grid) to zero.
    /// Note: global temperature offset is preserved.
    pub fn reset_temperatures(&mut self) {
        self.cell_temperatures.as_mut_slice().fill(0.0);
    }

    // Fast temperature diffusion - processes all cells of the reduced grid each frame
    pub fn diffuse_temperature_fast(&mut self) {
        // Tuned to avoid rapid global heat "flooding" from local sources (fire/wood).
        let diffusion_rate = 0.10;
        let cooling_rate = 0.998;
        
        // Work directly with the reduced grid
        let width = (cs::SECTOR_SIZE.x / 4) as usize;
        let height = (cs::SECTOR_SIZE.y / 4) as usize;
        
        // Create temporary buffer for new temperatures
        let mut new_temps = vec![0.0f32; width * height];
        
        // Process all cells of the reduced grid
        for ty in 1..(height - 1) {
            for tx in 1..(width - 1) {
                let idx = ty * width + tx;
                let current = self.cell_temperatures.get(idx).copied().unwrap_or(0.0);
                
                // Get temperatures of neighboring cells in the reduced grid
                let top_idx = (ty + 1) * width + tx;
                let bot_idx = (ty - 1) * width + tx;
                let left_idx = ty * width + (tx - 1);
                let right_idx = ty * width + (tx + 1);
                
                let top_temp = self.cell_temperatures.get(top_idx).copied().unwrap_or(current);
                let bot_temp = self.cell_temperatures.get(bot_idx).copied().unwrap_or(current);
                let left_temp = self.cell_temperatures.get(left_idx).copied().unwrap_or(current);
                let right_temp = self.cell_temperatures.get(right_idx).copied().unwrap_or(current);
                
                // Simple diffusion: average with neighbors
                let avg = (top_temp + bot_temp + left_temp + right_temp) / 4.0;
                let diffused = current + (avg - current) * diffusion_rate;
                // Temperatures are stored as local delta; clamp effective temperature.
                let new_local = diffused * cooling_rate;
                let eff = new_local + self.global_temperature;
                let clamped = eff.max(TEMP_MIN).min(TEMP_MAX);
                let new_temp = clamped - self.global_temperature;
                
                new_temps[idx] = new_temp;
            }
        }
        
        // Apply new temperatures
        for ty in 1..(height - 1) {
            for tx in 1..(width - 1) {
                let idx = ty * width + tx;
                self.cell_temperatures[idx] = new_temps[idx];
            }
        }
    }

    fn spawn(
        &mut self,
        evolution_app: &mut EvolutionApp,
        size: PhysicalSize<u32>,
        scale_factor: f64,
    ) {
        if let Some(position) = evolution_app.cursor_position {
            let scale_factor = scale_factor;
            let logical_position: LogicalPosition<f64> =
                LogicalPosition::from_physical(position, scale_factor);
            let window_size = size;
            let scaled_window_size = PhysicalSize::new(
                window_size.width as f64 / scale_factor,
                window_size.height as f64 / scale_factor,
            );
            let percentage_position: (f64, f64) = (
                logical_position.x / scaled_window_size.width as f64,
                1.0 - logical_position.y / scaled_window_size.height as f64,
            );

            for _i in 0..evolution_app.number_of_cells_to_add as usize {
                let mut px = percentage_position.0 * cs::SECTOR_SIZE.x as f64
                    + (self.prng.next() as f64 - 128.0) / 25.0;
                let mut py = percentage_position.1 * cs::SECTOR_SIZE.y as f64
                    + (self.prng.next() as f64 - 128.0) / 25.0;

                px = clamp(px, 0.0, cs::SECTOR_SIZE.x as f64 - 1.0);
                py = clamp(py, 0.0, cs::SECTOR_SIZE.y as f64 - 1.0);

                self.diffuse_rgba.put_pixel(
                    px as u32,
                    py as u32,
                    image::Luma([self.pal_container.dict[&evolution_app.selected_option]]),
                );
            }
        }
    }

    pub fn update(
        &mut self,
        queue: &wgpu::Queue,
        mut sim_steps: i32,
        evolution_app: &mut EvolutionApp,
        world: &mut specs::World,
        shared_state: &Rc<RefCell<SharedState>>,
        size: PhysicalSize<u32>,
        scale_factor: f64,
    ) -> UpdateResult
    {
        let update_start_time = instant::now();
        // Shader time is simulation-time based (deterministic, starts at 0).
        // This makes time "fixed at start" and independent from wall clock.
        self.world_settings.time = self.sim_time_seconds as f32;
        
        // Update display mode from evolution_app
        self.world_settings.display_mode = match evolution_app.display_mode {
            crate::evolution_app::DisplayMode::Normal => 0.0,
            crate::evolution_app::DisplayMode::Temperature => 1.0,
            crate::evolution_app::DisplayMode::Both => 2.0,
        };
        self.world_settings.global_temperature = self.global_temperature;

        let sim_upd_start_time = instant::now();

        let dimensions = self.diffuse_rgba.dimensions();

        // Update hover info for UI (cell under cursor + temperature)
        evolution_app.hover_info = None;
        if let Some(position) = evolution_app.cursor_position {
            // If pointer is outside the window bounds, don't show stale hover info.
            if position.x >= 0.0
                && position.y >= 0.0
                && position.x <= size.width as f64
                && position.y <= size.height as f64
            {
                let logical_position: LogicalPosition<f64> =
                    LogicalPosition::from_physical(position, scale_factor);
                let scaled_window_size = PhysicalSize::new(
                    size.width as f64 / scale_factor,
                    size.height as f64 / scale_factor,
                );

                if scaled_window_size.width > 0.0 && scaled_window_size.height > 0.0 {
                    let percentage_position: (f64, f64) = (
                        logical_position.x / scaled_window_size.width as f64,
                        1.0 - logical_position.y / scaled_window_size.height as f64,
                    );

                    let px = clamp(
                        percentage_position.0 * cs::SECTOR_SIZE.x as f64,
                        0.0,
                        cs::SECTOR_SIZE.x as f64 - 1.0,
                    );
                    let py = clamp(
                        percentage_position.1 * cs::SECTOR_SIZE.y as f64,
                        0.0,
                        cs::SECTOR_SIZE.y as f64 - 1.0,
                    );

                    let x = px as PointType;
                    let y = py as PointType;
                    let cell_id = self.diffuse_rgba.get_pixel(x as u32, y as u32).0[0];
                    let temperature = self.get_temperature(x, y);

                    evolution_app.hover_info = Some(crate::evolution_app::HoverInfo {
                        x,
                        y,
                        cell_id,
                        temperature,
                    });
                }
            }
        }

        if evolution_app.pressed && !evolution_app.hovered {
            self.spawn(evolution_app, size, scale_factor);
        }

        let mut dropping = false;
        if sim_steps > 10 {
            sim_steps = 1;
            dropping = true;
        }

        if sim_steps > 0 {
            update::update_tick(
                self,
                sim_steps,
                dimensions,
                evolution_app,
                world,
                shared_state,
                update_start_time,
            );
        }

        // Upload settings AFTER update_tick so GPU sees current light direction.
        queue.write_buffer(
            &self.settings_buffer,
            0,
            bytemuck::cast_slice(&[self.world_settings]),
        );

        let simulation_step_average_time = if sim_steps > 0 {
            (instant::now() - sim_upd_start_time) / sim_steps as f64
        } else {
            0.0
        };

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
        
        // Upload temperature data to GPU - write directly from the reduced grid
        let temp_width = dimensions.0 / 4;
        let temp_height = dimensions.1 / 4;
        
        let temp_texture_size = wgpu::Extent3d {
            width: temp_width,
            height: temp_height,
            depth_or_array_layers: 1,
        };
        
        queue.write_texture(
            wgpu::ImageCopyTexture {
                texture: &self.temperature_texture,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
                aspect: wgpu::TextureAspect::All,
            },
            bytemuck::cast_slice(&self.cell_temperatures),
            wgpu::ImageDataLayout {
                offset: 0,
                bytes_per_row: std::num::NonZeroU32::new(temp_width * 4), // 4 bytes per f32
                rows_per_image: std::num::NonZeroU32::new(temp_height),
            },
            temp_texture_size,
        );
        
        UpdateResult {
            simulation_step_average_time,
            update_time: instant::now() - update_start_time,
            dropping,
        }
    }

    pub fn render(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        output_view: &TextureView,
    ) {
        let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("Render Encoder"),
        });

        {
            // Render pass for the color texture
            {
                let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                    label: Some("Render Pass"),
                    color_attachments: &[
                        Some(wgpu::RenderPassColorAttachment {
                            view: &self.gbuffer.albedo_view,
                            resolve_target: None,
                            ops: wgpu::Operations {
                                load: wgpu::LoadOp::Clear(wgpu::Color::BLACK),
                                store: true,
                            },
                        }),
                        Some(wgpu::RenderPassColorAttachment {
                            view: &self.gbuffer.blur_2_texture_view,
                            resolve_target: None,
                            ops: wgpu::Operations {
                                load: wgpu::LoadOp::Clear(wgpu::Color::BLACK),
                                store: true,
                            },
                        }),
                    ],
                    depth_stencil_attachment: None,
                });

                render_pass.set_pipeline(&self.render_pipeline);
                render_pass.set_bind_group(0, &self.settings_bind_group, &[]);
                render_pass.set_bind_group(1, &self.type_bind_group, &[]);
                render_pass.set_bind_group(2, &self.temperature_bind_group, &[]);
                render_pass.set_bind_group(3, &self.shadow_props_bind_group, &[]);

                render_pass.set_vertex_buffer(0, self.vertex_buffer.slice(..));
                render_pass
                    .set_index_buffer(self.index_buffer.slice(..), wgpu::IndexFormat::Uint16);

                render_pass.draw_indexed(0..self.num_indices, 0, 0..1);
            }

            // Render pass for the bloom texture
            {
                let mut color_texture_view = wgpu::TextureViewDescriptor::default().clone();
                color_texture_view.format = Some(self.surface_format);

                let color_texture_sampler = device.create_sampler(&wgpu::SamplerDescriptor {
                    address_mode_u: wgpu::AddressMode::ClampToEdge,
                    address_mode_v: wgpu::AddressMode::ClampToEdge,
                    address_mode_w: wgpu::AddressMode::ClampToEdge,
                    mag_filter: wgpu::FilterMode::Linear,
                    min_filter: wgpu::FilterMode::Linear,
                    mipmap_filter: wgpu::FilterMode::Linear,
                    ..Default::default()
                });

                let to_hor_blur_bg = device.create_bind_group(&wgpu::BindGroupDescriptor {
                    layout: &self.float_texture_plus_sampler_bgl,
                    entries: &[
                        wgpu::BindGroupEntry {
                            binding: 0,
                            resource: wgpu::BindingResource::TextureView(
                                &self.gbuffer.blur_2_texture_view,
                            ),
                        },
                        wgpu::BindGroupEntry {
                            binding: 1,
                            resource: wgpu::BindingResource::Sampler(&color_texture_sampler),
                        },
                    ],
                    label: Some("to_hor_blur_bg"),
                });

                let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                    label: Some("Hor to Vert"),
                    color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                        view: &self.gbuffer.blur_1_texture_view,
                        resolve_target: None,
                        ops: wgpu::Operations {
                            load: wgpu::LoadOp::Load,
                            store: true,
                        },
                    })],
                    depth_stencil_attachment: None,
                });

                render_pass.set_pipeline(&self.hblur_render_pipeline);
                render_pass.set_bind_group(0, &self.settings_bind_group, &[]);
                render_pass.set_bind_group(1, &to_hor_blur_bg, &[]);

                render_pass.set_vertex_buffer(0, self.vertex_buffer.slice(..));
                render_pass
                    .set_index_buffer(self.index_buffer.slice(..), wgpu::IndexFormat::Uint16);

                render_pass.draw_indexed(0..self.num_indices, 0, 0..1);
            }

            {
                let mut color_texture_view = wgpu::TextureViewDescriptor::default().clone();
                color_texture_view.format = Some(self.surface_format);

                let color_texture_sampler = device.create_sampler(&wgpu::SamplerDescriptor {
                    address_mode_u: wgpu::AddressMode::ClampToEdge,
                    address_mode_v: wgpu::AddressMode::ClampToEdge,
                    address_mode_w: wgpu::AddressMode::ClampToEdge,
                    mag_filter: wgpu::FilterMode::Linear,
                    min_filter: wgpu::FilterMode::Linear,
                    mipmap_filter: wgpu::FilterMode::Linear,
                    ..Default::default()
                });

                let to_vert_blur_bg = device.create_bind_group(&wgpu::BindGroupDescriptor {
                    layout: &self.float_texture_plus_sampler_bgl,
                    entries: &[
                        wgpu::BindGroupEntry {
                            binding: 0,
                            resource: wgpu::BindingResource::TextureView(
                                &self.gbuffer.blur_1_texture_view,
                            ),
                        },
                        wgpu::BindGroupEntry {
                            binding: 1,
                            resource: wgpu::BindingResource::Sampler(&color_texture_sampler),
                        },
                    ],
                    label: Some("to_vert_blur_bg"),
                });

                let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                    label: Some("Vert to output"),
                    color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                        view: &self.gbuffer.blur_2_texture_view,
                        resolve_target: None,
                        ops: wgpu::Operations {
                            load: wgpu::LoadOp::Load,
                            store: true,
                        },
                    })],
                    depth_stencil_attachment: None,
                });

                render_pass.set_pipeline(&self.vblur_render_pipeline);
                render_pass.set_bind_group(0, &self.settings_bind_group, &[]);
                render_pass.set_bind_group(1, &to_vert_blur_bg, &[]);

                render_pass.set_vertex_buffer(0, self.vertex_buffer.slice(..));
                render_pass
                    .set_index_buffer(self.index_buffer.slice(..), wgpu::IndexFormat::Uint16);

                render_pass.draw_indexed(0..self.num_indices, 0, 0..1);
            }

            {
                let mut color_texture_view = wgpu::TextureViewDescriptor::default().clone();
                color_texture_view.format = Some(self.surface_format);

                let color_texture_sampler = device.create_sampler(&wgpu::SamplerDescriptor {
                    address_mode_u: wgpu::AddressMode::ClampToEdge,
                    address_mode_v: wgpu::AddressMode::ClampToEdge,
                    address_mode_w: wgpu::AddressMode::ClampToEdge,
                    mag_filter: wgpu::FilterMode::Nearest,
                    min_filter: wgpu::FilterMode::Nearest,
                    mipmap_filter: wgpu::FilterMode::Nearest,
                    ..Default::default()
                });

                let combine_bg = device.create_bind_group(&wgpu::BindGroupDescriptor {
                    layout: &self.float_texture_plus_sampler_plus_texture_bgl,
                    entries: &[
                        wgpu::BindGroupEntry {
                            binding: 0,
                            resource: wgpu::BindingResource::TextureView(&self.gbuffer.albedo_view),
                        },
                        wgpu::BindGroupEntry {
                            binding: 1,
                            resource: wgpu::BindingResource::Sampler(&color_texture_sampler),
                        },
                        wgpu::BindGroupEntry {
                            binding: 2,
                            resource: wgpu::BindingResource::TextureView(
                                &self.gbuffer.blur_2_texture_view,
                            ),
                        },
                    ],
                    label: Some("combine_bg"),
                });

                let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                    label: Some("Vert to output"),
                    color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                        view: &output_view,
                        resolve_target: None,
                        ops: wgpu::Operations {
                            load: wgpu::LoadOp::Load,
                            store: true,
                        },
                    })],
                    depth_stencil_attachment: None,
                });

                render_pass.set_pipeline(&self.bloom_render_pipeline);
                render_pass.set_bind_group(0, &self.settings_bind_group, &[]);
                render_pass.set_bind_group(1, &combine_bg, &[]);

                render_pass.set_vertex_buffer(0, self.vertex_buffer.slice(..));
                render_pass
                    .set_index_buffer(self.index_buffer.slice(..), wgpu::IndexFormat::Uint16);

                render_pass.draw_indexed(0..self.num_indices, 0, 0..1);
            }
        }

        queue.submit(std::iter::once(encoder.finish()));
    }
}
