use wgpu::Device;
use crate::graphics::{Graphics, Renderer, ResourceType};
use super::DEPTH_FORMAT;
use image::{GenericImageView, ImageBuffer, Rgba};

pub struct Texture {
    pub(in crate::graphics) texture: wgpu::Texture,
    pub(in crate::graphics) view: wgpu::TextureView,
    pub(in crate::graphics) bind_group: wgpu::BindGroup,
    resource_type: ResourceType,
}

impl Texture {
    /// Create a texture with a bind group
    pub fn new_normal(graphics: &Graphics, dimensions: (u32, u32)) -> Self {
        let (texture, view) = Self::unbound(graphics, dimensions);
        let bind_group = Self::get_bind_group(graphics, &view, ResourceType::Texture);
        Self {
            texture,
            bind_group,
            view,
            resource_type: ResourceType::Texture,
        }
    }
    /// Create a depth texture with a bind group
    pub fn new_depth(graphics: &Graphics, dimensions: (u32, u32)) -> Self {
        let (texture, view) = Self::depth(&graphics.device, dimensions);
        let bind_group = Self::get_bind_group(graphics, &view, ResourceType::Shadows);
        Self {
            texture,
            bind_group,
            view,
            resource_type: ResourceType::Shadows,
        }
    }

    /// Create a texture from an image
    pub fn from_bytes(graphics: &Graphics, image_bytes: &[u8]) -> Self {
        let loaded_image = image::load_from_memory(image_bytes).unwrap();
        Self::from_image(graphics, &loaded_image.to_rgba8(), loaded_image.dimensions())
    }

    /// Create a texture from an imagebuffer
    pub fn from_image(graphics: &Graphics, rgba: &ImageBuffer<Rgba<u8>, Vec<u8>>, dimensions: (u32, u32)) -> Self {
        let texture = Self::new_normal(graphics, dimensions);

        let texture_size = wgpu::Extent3d {
            width: dimensions.0,
            height: dimensions.1,
            depth_or_array_layers: 1,
        };

        // Write the texture
        graphics.queue.write_texture(
            wgpu::TexelCopyTextureInfo {
                texture: &texture.texture,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
                aspect: wgpu::TextureAspect::All,
            },
            &rgba,
            wgpu::TexelCopyBufferLayout {
                offset: 0,
                bytes_per_row: Some(4 * dimensions.0),
                rows_per_image: Some(dimensions.1),
            },
            texture_size,
        );

        texture
    }

    pub fn bind(&self, renderer: &mut Renderer) {
        let group = renderer.get_group(self.resource_type);
        renderer.render_pass.as_mut().unwrap().set_bind_group(group, &self.bind_group, &[]);
    }

    /// Create a texture without a bind group
    pub(in crate::graphics) fn unbound(graphics: &Graphics, dimensions: (u32, u32)) -> (wgpu::Texture, wgpu::TextureView) {
        let texture_size = wgpu::Extent3d {
            width: dimensions.0,
            height: dimensions.1,
            // All textures are stored as 3D, we represent our 2D texture
            // by setting depth to 1.
            depth_or_array_layers: 1,
        };
        let desc = wgpu::TextureDescriptor {
            label: Some("Texture"),
            size: texture_size,
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba8UnormSrgb,
            usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
            view_formats: &[],
        };
        let texture = graphics.device.create_texture(&desc);
        let view = texture.create_view(&wgpu::TextureViewDescriptor::default());
        (texture, view)
    }

    /// Create a depth texture
    pub(in crate::graphics) fn depth(device: &Device, dimensions: (u32, u32)) -> (wgpu::Texture, wgpu::TextureView) {
        let size = wgpu::Extent3d {
            width: dimensions.0,
            height: dimensions.1,
            depth_or_array_layers: 1,
        };
        let desc = wgpu::TextureDescriptor {
            label: Some("Depth texture"),
            size,
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: DEPTH_FORMAT,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::TEXTURE_BINDING,
            view_formats: &[],
        };
        let texture = device.create_texture(&desc);
        let view = texture.create_view(&wgpu::TextureViewDescriptor::default());
        (texture, view)
    }

    /// Create a texture with a bind group
    fn get_bind_group(graphics: &Graphics, view: &wgpu::TextureView, resource_type: ResourceType) -> wgpu::BindGroup {
        let layout = &graphics.get_layout(resource_type);
        let sampler = graphics.device.create_sampler(&wgpu::SamplerDescriptor {
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            address_mode_w: wgpu::AddressMode::ClampToEdge,
            mag_filter: wgpu::FilterMode::Nearest,
            min_filter: wgpu::FilterMode::Nearest,
            mipmap_filter: wgpu::FilterMode::Nearest,
            ..Default::default()
        });

        let bind_group = graphics.device.create_bind_group(
            &wgpu::BindGroupDescriptor {
                layout,
                entries: &[
                    wgpu::BindGroupEntry {
                        binding: 0,
                        resource: wgpu::BindingResource::TextureView(view),
                    },
                    wgpu::BindGroupEntry {
                        binding: 1,
                        resource: wgpu::BindingResource::Sampler(&sampler),
                    }
                ],
                label: Some("Bind group"),
            }
        );
        bind_group
    }
}