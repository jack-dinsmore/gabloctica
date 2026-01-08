use image::{ImageBuffer, Rgba};
use rusttype::{Scale, point};

use crate::graphics::{Graphics, Texture};

struct Font {
    font: rusttype::Font<'static>,
    texture: Texture,
    borders: Vec<f32>,
}

impl Font {
    pub fn new(graphics: &Graphics, bytes: &'static [u8]) -> Self {
        let font = rusttype::Font::try_from_bytes(bytes).unwrap();
        let scale = Scale::uniform(24.);
        let string: String = (0..255u8).map(|c| c as char).collect();

        let mut full_width = 0;
        let mut full_height = 0;
        let mut starts = Vec::new();
        for g in font.layout(&string, scale, point(0., 0.)) {
            let bbox = g.pixel_bounding_box().unwrap();
            starts.push(full_width);
            full_width += bbox.width() as u32;
            full_height = full_height.max(bbox.height() as u32);
        }

        let mut image = ImageBuffer::from_pixel(full_width, full_height, Rgba([255, 255, 255, 1]));
        for (g, offset_x) in font.layout(&string, scale, point(0., 0.)).zip(&starts) {
            let g = g.into_unpositioned().positioned(point(0.,0.));
            g.draw(|x,y,v| {
                image[(offset_x+x, y)] = Rgba([0, 0, 0, (v*255.) as u8])
            });
        }

        let texture = Texture::from_image(graphics, &image, (full_width, full_height));
        let mut borders: Vec<f32> = starts.iter().map(|s| (*s as f32) / (full_width as f32)).collect();
        borders.push(1.);
        
        Self {
            font,
            texture,
            borders,
        }
    }

    // pub fn text(text: &str) -> Text {
    //     for g in font.layout(text, scale, point(0., 0.)) {

    //     }
    // }
}

struct Text {

}

impl Drop for Text {
    fn drop(&mut self) {
        todo!()
    }
}