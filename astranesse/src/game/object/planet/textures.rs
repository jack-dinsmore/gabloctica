use std::collections::HashSet;

use cgmath::{InnerSpace, Vector3};
use image::{ImageBuffer, Rgba};
use noise::{NoiseFn, Perlin};
use rand::{Rng, RngExt};
use rand_distr::StandardNormal;

use crate::{game::object::{BlockId, BlockProperties, chunk::{PhysicalBlock, PhysicalChunk}}, util::{CHUNK_SIZE, KMeans, Ori, SphericalInterpolator}};

pub struct Structure {
    blocks: Vec<((i32, i32, i32), Ori, BlockId)>, // Pos, ori, block
}

impl Structure {
    pub fn new(blocks: Vec<((i32, i32, i32), Ori, BlockId)>) -> Self {
        Self {
            blocks,
        }
    }

    pub fn new_tree(id: BlockId, rng: &mut impl Rng) -> Self {
        fn get_acceptable(blocks: &HashSet<((i32, i32, i32), Ori, BlockId)>, new_pos: (i32, i32, i32)) -> bool {
            let mut min_dist2 = i32::MAX;
            for (pos, _, _) in blocks {
                if new_pos.2 > pos.2 {continue;}
                let dist2 = ((pos.0 - new_pos.0) as i32).pow(2) + ((pos.1 - new_pos.1) as i32).pow(2) + ((pos.2 - new_pos.2) as i32).pow(2);
                min_dist2 = min_dist2.min(dist2);
            }
            min_dist2 > 2
        }

        let mut blocks = HashSet::new();
        let mut queue = vec![(0, 0, 0)];
        let ori = Ori { data: 0 };
        while let Some(pos) = queue.pop() {
            blocks.insert((pos, ori, id));

            // Rotate
            let rotate_prob = if pos.2 < 4 { 0.1 } else { 0.3 };
            if rng.random::<f32>() < rotate_prob {
                let mut success = false;
                for _ in 0..20 {
                    // Find a rotation direction
                    let new_pos = match rng.random::<u8>() % 4 {
                        0 => (pos.0 + 1, pos.1, pos.2),
                        1 => (pos.0 - 1, pos.1, pos.2),
                        2 => (pos.0, pos.1 + 1, pos.2),
                        3 => (pos.0, pos.1 - 1, pos.2),
                        _ => unreachable!()
                    };
                    if get_acceptable(&blocks, new_pos) {
                        queue.push(new_pos);
                        success = true;
                        break;
                    }
                }
                
                if success {
                    continue;
                }
            }

            // Split
            if rng.random::<f32>() < 0.1 {
                match rng.random::<u8>() % 2 {
                    0 => {
                        queue.push((pos.0 + 1, pos.1, pos.2));
                        queue.push((pos.0 - 1, pos.1, pos.2));
                    },
                    1 => {
                        queue.push((pos.0, pos.1 + 1, pos.2));
                        queue.push((pos.0, pos.1 - 1, pos.2));
                    },
                    _ => unreachable!()
                }
                continue;
            }

            // Stop
            let stop_prob = if pos.2 < 4 { 0. } else { 0.2 };
            if rng.random::<f32>() < stop_prob || pos.2 > 10 {
                continue;
            }

            // Grow
            if get_acceptable(&blocks, (pos.0, pos.1, pos.2 + 1)){
                queue.push((pos.0, pos.1, pos.2 + 1));
            }
        }

        Self {
            blocks: blocks.into_iter().collect()
        }
    }

    pub fn place(&self, chunk: &mut PhysicalChunk, (u, v, w): (i32, i32, i32)) {
        for ((du, dv, dw), ori, id) in &self.blocks {
            if u + du < 0 || u + du >= CHUNK_SIZE as i32 { continue; }
            if v + dv < 0 || v + dv >= CHUNK_SIZE as i32 { continue; }
            if w + dw < 0 || w + dw >= CHUNK_SIZE as i32 { continue; }
            chunk.grid[((u+du) as u32, (v+dv) as u32, (w+dw) as u32)] = PhysicalBlock { ori: *ori, id: *id };
        }
    }
}

pub struct BlockSelector {
    kmeans: KMeans<3>,
    block_ids: Vec<BlockId>,
    structures: Vec<Structure>,
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
            block_ids.push(properties.add_textured_block(image, 1000.));
        }

        let mut structures = Vec::new();
        for _ in 0..4 {
            structures.push(Structure::new_tree(block_ids[0], rng));
        }

        Self {
            kmeans: KMeans::new(&samples, 16, rng),
            block_ids,
            structures,
        }
    }

    pub fn get_block_id(&self, altitude: f32, humidity: f32, temperature: f32) -> BlockId {
        self.block_ids[self.kmeans.evaluate([altitude, humidity, temperature])] as BlockId
    }
    
    pub fn get_structure(&self, altitude: f32, humidity: f32, temperature: f32, rng: &mut impl Rng) -> Option<&Structure> {
        for structure in &self.structures {
            if rng.random::<f32>() < 0.005 {
                return Some(structure)
            }
        }
        None
    }
}

fn to_color(c: f64) -> u8 {
    (c * 255.) as u8
}