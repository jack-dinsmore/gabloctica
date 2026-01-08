use std::marker::PhantomData;
use crate::graphics::Graphics;

mod buffer;
mod texture;

pub use texture::Texture;
pub use buffer::{StorageBuffer, UniformBuffer};

pub(super) const DEPTH_FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::Depth32Float;
pub(super) const TEXTURE_GROUP: u32 = 3;

pub trait Uniform: bytemuck::Zeroable + bytemuck::Pod {
    const GROUP: u32;
}