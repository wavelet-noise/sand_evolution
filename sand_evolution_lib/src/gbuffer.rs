use wgpu::{Texture, TextureView};

pub struct GBuffer {
    pub albedo: Texture,
    pub normal: Texture,
    pub depth: Texture,
    pub bloom: Texture,

    pub albedo_view: TextureView,
    pub blur_1_texture_view: TextureView,
    pub depth_view: TextureView,
    pub blur_2_texture_view: TextureView,
    // Add more texture fields as needed
}

impl GBuffer {
    pub fn new(
        device: &wgpu::Device,
        width: u32,
        height: u32,
        surface_format: wgpu::TextureFormat,
    ) -> GBuffer {
        let albedo = create_texture(
            device,
            width,
            height,
            surface_format,
            wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::TEXTURE_BINDING,
        );
        let normal = create_texture(
            device,
            width,
            height,
            surface_format,
            wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::TEXTURE_BINDING,
        );
        let depth = create_texture(
            device,
            width,
            height,
            wgpu::TextureFormat::Depth32Float,
            wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::TEXTURE_BINDING,
        );

        let bloom = create_texture(
            device,
            width,
            height,
            surface_format,
            wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::TEXTURE_BINDING,
        );

        let descr = &wgpu::TextureViewDescriptor {
            label: Some("color_view"),
            format: Some(surface_format),
            dimension: Some(wgpu::TextureViewDimension::D2),
            aspect: wgpu::TextureAspect::All,
            base_mip_level: 0,
            base_array_layer: 0,
            array_layer_count: None,
            mip_level_count: None,
        };

        let albedo_view = albedo.create_view(descr);
        let normal_view = normal.create_view(descr);
        let depth_view = depth.create_view(&wgpu::TextureViewDescriptor::default());
        let bloom_view = bloom.create_view(descr);

        GBuffer {
            albedo,
            normal,
            depth,
            bloom,

            albedo_view,
            blur_1_texture_view: normal_view,
            depth_view,
            blur_2_texture_view: bloom_view,
        }
    }
}

fn create_texture(
    device: &wgpu::Device,
    width: u32,
    height: u32,
    format: wgpu::TextureFormat,
    usage: wgpu::TextureUsages,
) -> Texture {
    device.create_texture(&wgpu::TextureDescriptor {
        size: wgpu::Extent3d {
            width,
            height,
            depth_or_array_layers: 1,
        },
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format,
        usage,
        label: None,
    })
}
