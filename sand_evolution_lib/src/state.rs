use cgmath::num_traits::clamp;
use wgpu::{util::DeviceExt, Surface, TextureFormat, TextureView};
use winit::{
    dpi::{LogicalPosition, PhysicalSize},
    window::Window,
};

use crate::{
    cells::{self, stone::Stone, wood::Wood, CellRegistry, Prng},
    cs,
    evolution_app::EvolutionApp,
    gbuffer::GBuffer,
    update, Vertex, INDICES, VERTICES,
};

#[repr(C)]
#[derive(Debug, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct WorldSettings {
    time: f32,
    _wasm_padding0: f32,
    _wasm_padding1: f32,
    _wasm_padding2: f32,
}

pub struct UpdateResult {
    pub simulation_step_average_time: f64,
    pub update_time: f64,
}

pub struct State {
    render_pipeline: wgpu::RenderPipeline,
    //bloom_render_pipeline: wgpu::RenderPipeline,
    gbuffer_collect_pipeline: wgpu::RenderPipeline,
    vertex_buffer: wgpu::Buffer,
    index_buffer: wgpu::Buffer,
    num_indices: u32,
    world_settings: WorldSettings,
    settings_buffer: wgpu::Buffer,
    settings_bind_group: wgpu::BindGroup,
    start_time: f64,
    diffuse_bind_group: wgpu::BindGroup,
    gbuffer_combine_bind_group: wgpu::BindGroup,
    pub diffuse_rgba: image::ImageBuffer<image::Luma<u8>, Vec<u8>>,
    diffuse_texture: wgpu::Texture,
    pub a: cs::PointType,
    pub b: cs::PointType,
    last_spawn: f32,
    pub pal_container: cells::CellRegistry,
    pub prng: cells::Prng,
    base_texture: wgpu::Texture,
    glow_texture: wgpu::Texture,
    gbuffer: GBuffer,
}

impl State {
    pub async fn new(
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

        let gbuffer = GBuffer::new(device, dimensions.0, dimensions.1);

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

        let type_render_and_fullscreen_vertex = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Shader"),
            source: wgpu::ShaderSource::Wgsl(include_str!("shader.wgsl").into()),
        });

        let bloom_shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Shader"),
            source: wgpu::ShaderSource::Wgsl(include_str!("shader.wgsl").into()),
        });

        let gbuffer_collect_shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Shader"),
            source: wgpu::ShaderSource::Wgsl(include_str!("gbuffer_collect_shader.wgsl").into()),
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

        let cell_type_texture_view = cell_type_texture.create_view(&tv);

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

        let cell_type_bind_group_layout =
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

        let gbuffer_bind_group_layout =
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

        let type_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &cell_type_bind_group_layout,
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
            label: Some("diffuse_bind_group"),
        });

        let gbuffer_combine_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &gbuffer_bind_group_layout,
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

        //-------------------------------

        let type_render_pipeline_layout =
            device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("Render Pipeline Layout"),
                bind_group_layouts: &[&settings_bind_group_layout, &cell_type_bind_group_layout],
                push_constant_ranges: &[],
            });

        let gbuffer_combine_pipeline_layout =
            device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("Gbuffer Render Pipeline Layout"),
                bind_group_layouts: &[&settings_bind_group_layout, &gbuffer_bind_group_layout],
                push_constant_ranges: &[],
            });

        // let bloom_render_pipeline =
        //     device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
        //         label: Some("Bloom Render Pipeline"),
        //         layout: Some(&type_render_pipeline_layout),
        //         vertex: wgpu::VertexState {
        //             module: &vertex_full_screen_shader,
        //             entry_point: "vs_main",
        //             buffers: &[Vertex::desc()],
        //         },
        //         fragment: Some(wgpu::FragmentState {
        //             module: &bloom_shader,
        //             entry_point: "fs_main",
        //             targets: &[Some(wgpu::ColorTargetState {
        //                 format: config.format,
        //                 blend: Some(wgpu::BlendState {
        //                     color: wgpu::BlendComponent::OVER,
        //                     alpha: wgpu::BlendComponent::OVER,
        //                 }),
        //                 write_mask: wgpu::ColorWrites::ALL,
        //             })],
        //         }),
        //         primitive: wgpu::PrimitiveState {
        //             topology: wgpu::PrimitiveTopology::TriangleList,
        //             strip_index_format: None,
        //             front_face: wgpu::FrontFace::Ccw,
        //             cull_mode: Some(wgpu::Face::Back),
        //             // Setting this to anything other than Fill requires Features::POLYGON_MODE_LINE
        //             // or Features::POLYGON_MODE_POINT
        //             polygon_mode: wgpu::PolygonMode::Fill,
        //             // Requires Features::DEPTH_CLIP_CONTROL
        //             unclipped_depth: false,
        //             // Requires Features::CONSERVATIVE_RASTERIZATION
        //             conservative: false,
        //         },
        //         depth_stencil: None,
        //         multisample: wgpu::MultisampleState {
        //             count: 1,
        //             mask: !0,
        //             alpha_to_coverage_enabled: false,
        //         },
        //         // If the pipeline will be used with a multiview render pass, this
        //         // indicates how many array layers the attachments will have.
        //         multiview: None,
        //     });

        let gbuffer_collect_pipeline =
            device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
                label: Some("Gbuffer Render Pipeline"),
                layout: Some(&gbuffer_combine_pipeline_layout),
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
            render_pipeline: type_render_pipeline,
            //bloom_render_pipeline,
            gbuffer_collect_pipeline,
            vertex_buffer,
            index_buffer,
            num_indices,
            world_settings,
            settings_buffer,
            settings_bind_group,
            start_time,
            diffuse_bind_group: type_bind_group,
            diffuse_rgba,
            gbuffer_combine_bind_group,
            diffuse_texture: cell_type_texture,
            a,
            b,
            last_spawn,
            pal_container,
            prng,
            base_texture,
            glow_texture,
            gbuffer,
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

            let mut buf = [0u8; 10000 + 1];
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

    pub fn update(
        &mut self,
        queue: &wgpu::Queue,
        sim_steps: u8,
        evolution_app: &mut EvolutionApp,
        window: &Window,
    ) -> UpdateResult {
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

        update::update_dim(self, sim_steps, dimensions);

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
                    color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                        view: &output_view,
                        resolve_target: None,
                        ops: wgpu::Operations {
                            load: wgpu::LoadOp::Clear(wgpu::Color::BLACK),
                            store: true,
                        },
                    })],
                    depth_stencil_attachment: None,
                });

                render_pass.set_pipeline(&self.render_pipeline);
                render_pass.set_bind_group(0, &self.settings_bind_group, &[]);
                render_pass.set_bind_group(1, &self.diffuse_bind_group, &[]);

                render_pass.set_vertex_buffer(0, self.vertex_buffer.slice(..));
                render_pass
                    .set_index_buffer(self.index_buffer.slice(..), wgpu::IndexFormat::Uint16);

                render_pass.draw_indexed(0..self.num_indices, 0, 0..1);
            }

            // Render pass for the bloom texture
            {
                // let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                //     label: Some("Bloom Render Pass"),
                //     color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                //         view: &self.gbuffer.bloom_view,
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

            // let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            //     label: Some("Render Pass"),
            //     color_attachments: &[Some(wgpu::RenderPassColorAttachment {
            //         view: &output_view,
            //         resolve_target: None,
            //         ops: wgpu::Operations {
            //             load: wgpu::LoadOp::Clear(wgpu::Color::BLACK),
            //             store: true,
            //         },
            //     })],
            //     depth_stencil_attachment: None,
            // });

            // render_pass.set_pipeline(&self.gbuffer_collect_pipeline);
            // render_pass.set_bind_group(0, &self.settings_bind_group, &[]);
            // render_pass.set_bind_group(1, &self.gbuffer_combine_bind_group, &[]);

            // render_pass.set_vertex_buffer(0, self.vertex_buffer.slice(..));
            // render_pass.set_index_buffer(self.index_buffer.slice(..), wgpu::IndexFormat::Uint16);

            // render_pass.draw_indexed(0..self.num_indices, 0, 0..1);

            // drop(render_pass);
        }

        queue.submit(std::iter::once(encoder.finish()));
    }
}
