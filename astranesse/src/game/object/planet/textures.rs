use cgmath::{InnerSpace, Vector3};
use image::{ImageBuffer, Rgba};
use noise::{NoiseFn, Perlin};
use rand::{Rng, RngExt};
use rand_distr::StandardNormal;

use crate::{game::object::{BlockId, BlockProperties}, util::{KMeans, SphericalInterpolator}};

pub struct BlockSelector {
    kmeans: KMeans<3>,
    block_ids: Vec<BlockId>,
}

impl BlockSelector {
    pub fn new(properties: &mut BlockProperties, alt_interp: &SphericalInterpolator<f32>, humid_interp: &SphericalInterpolator<f32>, temp_interp: &SphericalInterpolator<f32>, rng: &mut impl Rng) -> Self {
        let mut block_ids = Vec::new();
        const N_BLOCKS: u32 = 16;

        let mut samples = Vec::with_capacity(128);
        for _ in 0..128 {
            let mut pos = Vector3::new(rng.sample(StandardNormal), rng.sample(StandardNormal), rng.sample(StandardNormal));
            pos = pos.normalize();

            samples.push([alt_interp.get(pos), humid_interp.get(pos), temp_interp.get(pos)]);
        }

        for cube_index in 0..N_BLOCKS {
            let mut image: ImageBuffer<Rgba<u8>, _> = ImageBuffer::new(16 * 6, 16);
            for face in 0..6 {
                let noise_1 = Perlin::new(rng.random());
                let noise_2 = Perlin::new(rng.random());
                const SCALE_1: f64 = 0.25;
                const SCALE_2: f64 = 0.5;
                let color_1 = [rng.random::<u8>(), rng.random::<u8>(), rng.random::<u8>()];
                let color_2 = [rng.random::<u8>(), rng.random::<u8>(), rng.random::<u8>()];

                for dx in 0..16 {
                    for dy in 0..16 {
                        let n1 = noise_1.get([dx as f64 * SCALE_1, dy as f64 * SCALE_1]) as f32 * 0.5;
                        let n2 = noise_2.get([dx as f64 * SCALE_2, dy as f64 * SCALE_2]) as f32 * 0.5;
                        let pixel = image.get_pixel_mut(face * 16 + dx, dy);
                        pixel[0] = ((color_1[0] as f32 * (1. + n1) + color_2[0] as f32 * (1. + n2)) / 2.) as u8;
                        pixel[1] = ((color_1[1] as f32 * (1. + n1) + color_2[1] as f32 * (1. + n2)) / 2.) as u8;
                        pixel[2] = ((color_1[2] as f32 * (1. + n1) + color_2[2] as f32 * (1. + n2)) / 2.) as u8;
                        pixel[3] = 255;
                    }
                }
            }
            block_ids.push(properties.add_textured_block(image));
        }

        Self {
            kmeans: KMeans::new(&samples, 16, rng),
            block_ids,
        }
    }

    pub fn get_block_id(&self, altitude: f32, humidity: f32, temperature: f32) -> BlockId {
        self.block_ids[self.kmeans.evaluate([altitude, humidity, temperature])] as BlockId
    }
}

fn to_color(c: f64) -> u8 {
    (c * 255.) as u8
}