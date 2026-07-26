use crate::graphics::{Camera, Graphics, Renderer, ResourceType};
use super::DEPTH_FORMAT;
use image::{GenericImageView, ImageBuffer, Rgba};

/// A texture that can be bound
pub struct Texture {
    pub(in crate::graphics) texture: FreeTexture,
    pub(in crate::graphics) bind_group: wgpu::BindGroup,
}

/// A texture which cannot be bound
pub (in crate::graphics) struct FreeTexture {
    pub(in crate::graphics) texture: wgpu::Texture,
    pub(in crate::graphics) view: wgpu::TextureView,
}

#[derive(Clone, Copy, Debug)]
pub enum TextureType {
    Normal,
    Surface,
    Depth
}
impl TextureType {
    fn get_format(&self) -> wgpu::TextureFormat {
        match self {
            TextureType::Normal => wgpu::TextureFormat::Rgba8UnormSrgb,
            TextureType::Surface => wgpu::TextureFormat::Bgra8UnormSrgb,
            TextureType::Depth => DEPTH_FORMAT,
        }
    }
    fn get_usage(&self) -> wgpu::TextureUsages {
        match self {
            TextureType::Normal => wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
            TextureType::Surface | TextureType::Depth => wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::RENDER_ATTACHMENT,
        }
    }
}

impl FreeTexture {
    pub(in crate::graphics) fn new(device: &wgpu::Device, dimensions: (u32, u32), texture_type: TextureType) -> Self {
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
            format: texture_type.get_format(),
            usage: texture_type.get_usage(),
            view_formats: &[],
        };
        let texture = device.create_texture(&desc);
        let view = texture.create_view(&wgpu::TextureViewDescriptor::default());
        Self {
            texture,
            view,
        }
    }
}

impl Texture {
    /// Create a texture with a bind group
    pub fn new(graphics: &Graphics, camera: &Camera, dimensions: (u32, u32), texture_type: TextureType) -> Self {
        let texture = FreeTexture::new(&graphics.device, dimensions, texture_type);
        let bind_group = Self::get_bind_group(graphics, &texture.view, &camera.shadow_texture.view, Some(&camera.shadow_texture_sampler));
        Self {
            texture,
            bind_group,
        }
    }

    /// Create a texture with a bind group
    pub(in crate::graphics) fn new_surface_texture(graphics: &Graphics, dimensions: (u32, u32), depth_texture_view: &wgpu::TextureView) -> Self {
        let texture = FreeTexture::new(&graphics.device, dimensions, TextureType::Surface);
        let bind_group = Self::get_bind_group(graphics, &texture.view, depth_texture_view, None);
        Self {
            texture,
            bind_group,
        }
    }

    /// Create a texture from an image
    pub fn from_bytes(graphics: &Graphics, camera: &Camera, image_bytes: &[u8]) -> Self {
        let loaded_image = image::load_from_memory(image_bytes).unwrap();
        Self::from_image(graphics, camera, &loaded_image.to_rgba8(), loaded_image.dimensions())
    }

    /// Create a texture from an imagebuffer
    pub fn from_image(graphics: &Graphics, camera: &Camera, rgba: &ImageBuffer<Rgba<u8>, Vec<u8>>, dimensions: (u32, u32)) -> Self {
        let texture = Self::new(graphics, camera, dimensions, TextureType::Normal);

        let texture_size = wgpu::Extent3d {
            width: dimensions.0,
            height: dimensions.1,
            depth_or_array_layers: 1,
        };

        // Write the texture
        graphics.queue.write_texture(
            wgpu::TexelCopyTextureInfo {
                texture: &texture.texture.texture,
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
        let group = renderer.get_group(ResourceType::Texture);
        renderer.render_pass.as_mut().unwrap().set_bind_group(group, &self.bind_group, &[]);
    }

    /// Create a texture with a bind group. You should pass the two views. If you do not pass a second sampler, the first sampler will be re-used.
    fn get_bind_group(graphics: &Graphics, view1: &wgpu::TextureView, view2: &wgpu::TextureView, sampler2: Option<&wgpu::Sampler>) -> wgpu::BindGroup {
        let layout = &graphics.get_layout(ResourceType::Texture);
        let sampler = graphics.device.create_sampler(&wgpu::SamplerDescriptor {
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            address_mode_w: wgpu::AddressMode::ClampToEdge,
            mag_filter: wgpu::FilterMode::Nearest,
            min_filter: wgpu::FilterMode::Nearest,
            mipmap_filter: wgpu::FilterMode::Nearest,
            ..Default::default()
        });

        let sampler2_ref = match sampler2 {
            Some(s) => s,
            None => &sampler,
        };

        let bind_group = graphics.device.create_bind_group(
            &wgpu::BindGroupDescriptor {
                layout,
                entries: &[
                    wgpu::BindGroupEntry {
                        binding: 0,
                        resource: wgpu::BindingResource::TextureView(view1),
                    },
                    wgpu::BindGroupEntry {
                        binding: 1,
                        resource: wgpu::BindingResource::Sampler(&sampler),
                    },
                    wgpu::BindGroupEntry {
                        binding: 2,
                        resource: wgpu::BindingResource::TextureView(view2),
                    },
                    wgpu::BindGroupEntry {
                        binding: 3,
                        resource: wgpu::BindingResource::Sampler(&sampler2_ref),
                    }
                ],
                label: Some("Bind group"),
            }
        );
        bind_group
    }
}