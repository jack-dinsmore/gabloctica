use image::{ImageBuffer, Rgba};
use rusttype::{Scale, point};

use crate::graphics::{Camera, Graphics, Lighting, Renderer, ResourceType, Shader, StorageBuffer, TextVertex, Texture, resource::{IndexBuffer, VertexBuffer}};

#[include_wgsl_oil::include_wgsl_oil("../shaders/font.wgsl")]
mod font_shader {}

pub struct Font {
    shader: Shader,
    font: rusttype::Font<'static>,
    scale: Scale,
    texture: Texture,
    borders: Vec<f32>,
    width: u32, 
    height: u32,

    storage_buffer_vertex: StorageBuffer,
    storage_buffer_index: StorageBuffer,
    vertex_buffer: VertexBuffer<TextVertex>,
    index_buffer: IndexBuffer,
    vertices: Vec<TextVertex>,
    indices: Vec<u16>,
}
const MAX_SIZE: usize = 1024;

impl Font {
    pub fn new(graphics: &mut Graphics, bytes: &'static [u8]) -> Self {
        let shader = Shader::new::<TextVertex>(graphics, font_shader::SOURCE, vec![
            ResourceType::Texture,
        ]);
        let font = rusttype::Font::try_from_bytes(bytes).unwrap();
        let scale = Scale { x: 32., y: 42. };
        let string: String = (0..255u8).map(|c| c as char).collect();

        let mut full_width = 0;
        let mut full_height = 0;
        let mut starts = Vec::new();
        for g in font.layout(&string, scale, point(0., 0.)) {
            starts.push(full_width);
            if let Some(bbox) = g.pixel_bounding_box() {
                full_width += bbox.width() as u32;
                full_height = full_height.max(bbox.height() as u32);
            }
        };

        let mut image = ImageBuffer::from_pixel(full_width, full_height, Rgba([0, 0, 0, 0]));
        for (g, offset_x) in font.layout(&string, scale, point(0., 0.)).zip(&starts) {
            let g = g.into_unpositioned().positioned(point(0.,0.));
            g.draw(|x,y,v| {
                let vi = (v*255.) as u8;
                image[(offset_x+x, y)] = Rgba([0, 0, 0, vi]);
            });
        }
        let texture = Texture::from_image(graphics, &image, (full_width, full_height));

        let mut borders: Vec<f32> = starts.iter().map(|s| (*s as f32) / (full_width as f32)).collect();
        borders.push(1.);

        let storage_buffer_vertex = StorageBuffer::new(graphics, MAX_SIZE * std::mem::size_of::<TextVertex>());
        let storage_buffer_index = StorageBuffer::new(graphics, MAX_SIZE * std::mem::size_of::<u16>() * 6);
        let vertex_buffer = VertexBuffer::new(graphics, MAX_SIZE*4);
        let index_buffer = IndexBuffer::new(graphics, MAX_SIZE*6);  

        let size = graphics.window.inner_size();
        let width = size.width.max(1);
        let height = size.height.max(1);

        Self {
            shader,
            font,
            scale,
            texture,
            borders,
            vertices: Vec::new(),
            indices: Vec::new(),
            storage_buffer_vertex,
            storage_buffer_index,
            vertex_buffer,
            index_buffer,
            width,
            height
        }
    }

    pub fn copy_buffers(&self, renderer: &mut Renderer) {
        renderer.encoder().copy_buffer_to_buffer(
            &self.storage_buffer_vertex.buffer, 0, &self.vertex_buffer.buffer, 0,
            (self.vertices.len()*std::mem::size_of::<TextVertex>()) as u64);
        renderer.encoder().copy_buffer_to_buffer(
            &self.storage_buffer_index.buffer, 0, &self.index_buffer.buffer, 0,
            (self.indices.len()*std::mem::size_of::<u16>()) as u64);
    }

    pub fn render(&mut self, renderer: &mut Renderer, camera: &Camera, lighting: &Lighting) {
        self.shader.bind(renderer);
        camera.bind(renderer);
        lighting.bind(renderer);
        self.texture.bind(renderer);
        self.vertex_buffer.bind(renderer);
        self.index_buffer.bind(renderer);
        renderer.draw_indices(self.indices.len() as u32);
        self.indices.clear();
        self.vertices.clear();
    }

    pub fn text(&mut self, text: &str, px: f32, py: f32) {
        for (g, c) in self.font.layout(text, self.scale, point(0., 0.)).zip(text.chars()) {
            if let Some(bbox) = g.pixel_bounding_box() {
                let start = self.borders[c as usize];
                let stop = self.borders[c as usize+1];
                let min = (px + 2.*bbox.min.x as f32 / self.width as f32 - 1., -py - 2.*bbox.min.y as f32 / self.height as f32 + 1.);
                let max = (px + 2.*bbox.max.x as f32 / self.width as f32 - 1., -py - 2.*bbox.max.y as f32 / self.height as f32 + 1.);
                let i0 = self.vertices.len() as u16;
                self.vertices.push(TextVertex {
                    x: [min.0, min.1],
                    texpos: [start, 0.],
                });
                self.vertices.push(TextVertex {
                    x: [max.0, min.1],
                    texpos: [stop, 0.],
                });
                self.vertices.push(TextVertex {
                    x: [max.0, max.1],
                    texpos: [stop, 1.],
                });
                self.vertices.push(TextVertex {
                    x: [min.0, max.1],
                    texpos: [start, 1.],
                });
                self.indices.push(i0+0);
                self.indices.push(i0+2);
                self.indices.push(i0+1);
                self.indices.push(i0+0);
                self.indices.push(i0+3);
                self.indices.push(i0+2);
            }
        }
    }

    pub fn get_length(&mut self, text: &str) -> f32 {
        let mut start = f32::INFINITY;
        let mut stop = -f32::INFINITY;
        for g in self.font.layout(text, self.scale, point(0., 0.)) {
            if let Some(bbox) = g.pixel_bounding_box() {
                start = start.min(2.*bbox.min.x as f32 / self.width as f32 - 1.);
                stop = stop.min(2.*bbox.max.x as f32 / self.width as f32 - 1.);
            }
        }
        stop - start
    }

    pub fn get_height(&mut self) -> f32 {
        self.scale.y
    }

    pub fn update(&self, graphics: &Graphics) {
        self.storage_buffer_vertex.write(graphics, &self.vertices);
        self.storage_buffer_index.write(graphics, &self.indices);
    }
}